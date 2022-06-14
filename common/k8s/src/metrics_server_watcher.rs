use anyhow:: {Result, Ok };

use k8s_openapi::{api::{core::v1::{Pod, Node}}};
use kube::{
    api::{Api, DynamicObject, GroupVersionKind, ListParams, ObjectList},
    discovery::{self},
    Client
};
use serde_json::Value;

use std::{ time::{Duration}};
use tokio::time::{sleep};
use std::collections::HashMap;

use crate::stat_models::{pod_stats::{PodStats, PodAndContainerStats}, container_stats::ContainerStats, controller_stats::ControllerStats};

pub struct MetricsServerWatcher {
    pub client: Client,
}

impl MetricsServerWatcher {
    pub fn new(client: Client) -> Self {
        Self {
            client
        }
    }

    pub async fn start_metrics_call_task(self) {

        loop {
            self::gather_reporter_info(self.client.clone()).await.unwrap(); // TODO probably shouldn't clone here
            sleep(Duration::from_millis(30000)).await;
        }
    }    
}

async fn gather_reporter_info(client: Client) -> anyhow::Result<()> {

    let pods = self::get_all_pods(client.clone()).await?;
    let node_info = self::get_all_nodes(client.clone()).await;

    let pod_metrics = self::call_metric_api(&"PodMetrics", client.clone()).await?;
    let node_metrics = self::call_metric_api(&"NodeMetrics", client.clone()).await?;

    let mut controller_map: HashMap<String, ControllerStats> = HashMap::new();
    let mut pod_usage_map: HashMap<String, Value> = HashMap::new();

    let mut translated_pods_containers: Vec<PodAndContainerStats> = Vec::new();

    for pod_metric in pod_metrics {

        let containers = pod_metric.data["containers"].as_array();

        for container in containers.unwrap() {
            pod_usage_map.insert(container["name"].as_str().unwrap().to_string(), container["usage"].clone());
        }
    }

    for pod in pods {

        if pod.spec.is_none() || pod.status.is_none() {
            continue;
        }

        let status = pod.status.as_ref().unwrap();
        let spec = pod.spec.as_ref().unwrap();

        let translated_pod = PodStats::new(&pod);
        let controller_key = format!("{}.{}.{}", translated_pod.namespace.clone(), translated_pod.controller_type.clone(), translated_pod.controller.clone());
        
        let controller = controller_map.entry(controller_key.clone()).or_insert(ControllerStats::new()); 
        let conditions = status.conditions.as_ref().unwrap();

        if conditions.iter().any(
            |c| c.status.to_lowercase() == "true" && c.type_.to_lowercase() == "ready") {
                controller.inc_pods_ready();
        }

        controller.inc_pods_total();


        let mut container_status_map = HashMap::new();



        // TODO check option 
        for status in status.container_statuses.as_ref().unwrap() {
            container_status_map.insert(status.name.clone(), status.clone());

            let controller = controller_map.entry(controller_key.clone()).or_insert(ControllerStats::new()); 

            controller.inc_containers_total();

            if status.ready {
                controller.inc_containers_ready();
            }


        }

        if status.init_container_statuses.is_some() {
            for status in status.init_container_statuses.as_ref().unwrap() {
                container_status_map.insert(status.name.clone(), status.clone());

                let controller = controller_map.entry(controller_key.clone()).or_insert(ControllerStats::new()); 

                controller.inc_containers_total();
    
                if status.ready {
                    controller.inc_containers_ready();
                }
            }
        }

        //let mut container_map: HashMap<String, ContainerStats> = HashMap::new();
        for container in &spec.containers {

            if container.name.is_empty() || container.image.is_none() || container.resources.is_none() {
                continue;
            }

            let container_status = container_status_map.get(&container.name);

            if container_status.is_none() {
                continue;
            }

            let usage = pod_usage_map.get(&container.name);

            if usage.is_some() {
                let translated_container = 
                ContainerStats::build(&container, container_status.as_ref().unwrap(), usage.unwrap()["cpu"].to_string(), usage.unwrap()["memory"].to_string());
                translated_pods_containers.push(PodAndContainerStats::new(translated_pod.clone(), translated_container));
                
            }

        }

        if (&spec).init_containers.is_some() {

            for container in (&spec).init_containers.as_ref().unwrap() {

                if container.name.is_empty() || container.image.is_none() || container.resources.is_none() {
                    continue;
                }

                let container_status = container_status_map.get(&container.name);

                if container_status.is_none() {
                    continue;
                }

                let usage = pod_usage_map.get(&container.name);

                if usage.is_some() {
                    let translated_container = 
                    ContainerStats::build(&container, container_status.as_ref().unwrap(), usage.unwrap()["cpu"].to_string(), usage.unwrap()["memory"].to_string());
                    translated_pods_containers.push(PodAndContainerStats::new(translated_pod.clone(), translated_container));
                    
                }
            }  
        }         

    }

    for mut translated_pod_container in translated_pods_containers {

        let controller_key = format!("{}.{}.{}", translated_pod_container.pod_stats.namespace.clone(), translated_pod_container.pod_stats.controller_type.clone(), translated_pod_container.pod_stats.controller.clone());
        
        let controller_stats = controller_map.get(&controller_key);

        translated_pod_container.controller_stats.copy_stats(controller_stats.unwrap());

        info!("{}", serde_json::to_string(&translated_pod_container)?); // wrap in kube {}
    }

    Ok(())
}


async fn call_metric_api(kind: &str, client: Client) -> Result<ObjectList<DynamicObject>, kube::Error> {
    let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", kind);
    let (ar, _caps) = discovery::pinned_kind(&client, &gvk).await?;
    let api = Api::<DynamicObject>::all_with(client, &ar);

    let items = api.list(&ListParams::default()).await;

    items
}

async fn get_all_nodes(client: Client) -> ObjectList<Node> {
    let api: Api<Node> = Api::all(client);
    let nodes = api.list(&ListParams::default()).await.unwrap();

    nodes
}

async fn get_all_pods(client: Client) -> Result<ObjectList<Pod>, kube::Error>{
    let api: Api<Pod> = Api::all(client);
    let pods = api.list(&ListParams::default()).await;

    pods
}

