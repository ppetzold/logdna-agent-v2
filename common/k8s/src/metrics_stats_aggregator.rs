use anyhow::{Result};
use core::result::Result::Ok;

use k8s_openapi::api::core::v1::{Node, Pod};
use kube::{
    api::{Api, DynamicObject, GroupVersionKind, ListParams, ObjectList},
    discovery::{self},
    Client,
};
use serde_json::Value;

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

use crate::kube_stats::{
    container_stats::ContainerStats,
    controller_stats::ControllerStats,
    extended_pod_stats::{ExtendedPodStats},
    pod_stats::PodStats, node_stats::NodeStats,
};

pub struct MetricsServerAggregator {
    pub client: Client,
}

impl MetricsServerAggregator {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn start_metrics_call_task(self) {
        loop {
            let result = self::gather_reporter_info(self.client.clone()).await;

            match result {
                Ok(_) => {},
                Err(_) => {error!("Failed To Gather Metrics Server Info")},
            }

            sleep(Duration::from_millis(30000)).await;
        }
    }
}

async fn gather_reporter_info(client: Client) -> anyhow::Result<()> {
    let pods = self::get_all_pods(client.clone()).await?;
    let nodes = self::get_all_nodes(client.clone()).await?;

    let pod_metrics = self::call_metric_api(&"PodMetrics", client.clone()).await?;
    let node_metrics = self::call_metric_api(&"NodeMetrics", client.clone()).await?;

    let mut controller_map: HashMap<String, ControllerStats> = HashMap::new();
    let mut pod_usage_map: HashMap<String, Value> = HashMap::new();
    let mut node_usage_map: HashMap<String, Value> = HashMap::new();

    let mut extended_pod_stats: Vec<ExtendedPodStats> = Vec::new();

    build_pod_metric_map(pod_metrics, &mut pod_usage_map);
    process_pods(pods, &mut controller_map, pod_usage_map, &mut extended_pod_stats);
    print_pods(extended_pod_stats, controller_map);

    build_node_metric_map(node_metrics, &mut node_usage_map);
    process_nodes(nodes, node_usage_map);
    //print_pods(extended_pod_stats, controller_map);

    Ok(())
}

fn build_pod_metric_map(pod_metrics: ObjectList<DynamicObject>, pod_usage_map: &mut HashMap<String, Value>) {
    for pod_metric in pod_metrics {
        let containers = pod_metric.data["containers"].as_array();

        if containers.is_some()
        {
            for container in containers.unwrap() {

                let container_name = container["name"].as_str();

                if container_name.is_none() {
                    continue;
                }

                pod_usage_map.insert(container_name.unwrap().to_string(), container["usage"].clone());
            }
        }
    }
}

fn build_node_metric_map(node_metrics: ObjectList<DynamicObject>, node_usage_map: &mut HashMap<String, Value>) {
    for node_metric in node_metrics {
        let node_name = node_metric.metadata.name.unwrap_or_else(|| "NONE".to_string());
        let usage = &node_metric.data["usage"];

        node_usage_map.insert(node_name, usage.clone());
    }
}

fn print_pods(extended_pod_stats: Vec<ExtendedPodStats>, controller_map: HashMap<String, ControllerStats>) {
    for mut translated_pod_container in extended_pod_stats {
        let controller_key = format!(
            "{}.{}.{}",
            translated_pod_container.pod_stats.namespace.clone(),
            translated_pod_container.pod_stats.controller_type.clone(),
            translated_pod_container.pod_stats.controller.clone()
        );

        let controller_stats = controller_map.get(&controller_key);

        //unwrap gaurd
        translated_pod_container.controller_stats.copy_stats(controller_stats.unwrap());

        info!(r#"{{"kube":{}}}"#, serde_json::to_string(&translated_pod_container).unwrap()); // wrap in kube {}
    }
}

fn process_pods(
    pods: ObjectList<Pod>,
    controller_map: &mut HashMap<String, ControllerStats>,
    pod_usage_map: HashMap<String, Value>,
    extended_pod_stats: &mut Vec<ExtendedPodStats>,
) {
    for pod in pods {
        if pod.spec.is_none() || pod.status.is_none() {
            continue;
        }

        let status = pod.status.as_ref().unwrap();
        let spec = pod.spec.as_ref().unwrap();

        if status.conditions.is_none() || status.container_statuses.is_none() {
            continue;
        }

        let translated_pod = PodStats::new(&pod);

        let controller_key = format!(
            "{}.{}.{}",
            translated_pod.namespace.clone(),
            translated_pod.controller_type.clone(),
            translated_pod.controller.clone()
        );

        let controller = controller_map
            .entry(controller_key.clone())
            .or_insert(ControllerStats::new());

        let conditions = status.conditions.as_ref().unwrap();
        if conditions
            .iter()
            .any(|c| c.status.to_lowercase() == "true" && c.type_.to_lowercase() == "ready")
        {
            controller.inc_pods_ready();
        }

        controller.inc_pods_total();

        let mut container_status_map = HashMap::new();

        let default_status_vec = Vec::new();
        for status in status
            .container_statuses
            .as_ref()
            .unwrap_or_else(|| &&default_status_vec)
            .into_iter()
            .chain(
                status
                    .init_container_statuses
                    .as_ref()
                    .unwrap_or_else(|| &default_status_vec)
                    .into_iter(),
            )
        {
            container_status_map.insert(status.name.clone(), status.clone());

            let controller = controller_map
                .entry(controller_key.clone())
                .or_insert(ControllerStats::new());

            controller.inc_containers_total();

            if status.ready {
                controller.inc_containers_ready();
            }
        }

        let default_container_vec = Vec::new();
        for container in spec.containers.iter().chain(
            (&spec)
                .init_containers
                .as_ref()
                .unwrap_or_else(|| &default_container_vec)
                .iter(),
        ) {
            if container.name.is_empty() || container.image.is_none() || container.resources.is_none()
            {
                continue;
            }

            let container_status = container_status_map.get(&container.name);

            if container_status.is_none() {
                continue;
            }

            let usage = pod_usage_map.get(&container.name);

            if usage.is_some() {
                let translated_container = ContainerStats::new(
                    &container,
                    container_status.as_ref().unwrap(),
                    container_status.unwrap().state.as_ref().unwrap(),
                    usage.unwrap()["cpu"].as_str().unwrap_or_else(|| ""),
                    usage.unwrap()["memory"].as_str().unwrap_or_else(|| "")
                );

                extended_pod_stats.push(ExtendedPodStats::new(translated_pod.clone(), translated_container));
            }
        }
    }
}

fn process_nodes(nodes: ObjectList<Node>, node_usage_map: HashMap<String, Value>) {
    for node in nodes {
    
        if node.spec.is_none() || node.status.is_none() || node.metadata.name.is_none() {
            continue;
        }

        let translated_node = NodeStats::new(&node);
    }

}

async fn call_metric_api(
    kind: &str,
    client: Client,
) -> Result<ObjectList<DynamicObject>, kube::Error> {
    let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", kind);
    let (ar, _caps) = discovery::pinned_kind(&client, &gvk).await?;
    let api = Api::<DynamicObject>::all_with(client, &ar);

    let items = api.list(&ListParams::default()).await;

    items
}

async fn get_all_nodes(client: Client) -> Result<ObjectList<Node>, kube::Error>  {
    let api: Api<Node> = Api::all(client);
    let nodes = api.list(&ListParams::default()).await;

    nodes
}

async fn get_all_pods(client: Client) -> Result<ObjectList<Pod>, kube::Error> {
    let api: Api<Pod> = Api::all(client);
    let pods = api.list(&ListParams::default()).await;

    pods
}
