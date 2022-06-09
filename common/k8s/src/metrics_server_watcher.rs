use anyhow:: {Result, Ok };

use k8s_openapi::{api::{core::v1::{Pod, Node}}};
use kube::{
    api::{Api, DynamicObject, GroupVersionKind, ListParams, ObjectList},
    discovery::{self},
    Client
};

use std:: time::{Duration};
use tokio::time::{sleep};
use std::collections::HashMap;


use crate::stat_models::{pod_stats::PodStats, controller_stats::ControllerStats};


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

    for pod in pods {
        let translated_pod = PodStats::build(&pod);

        let controller_key = format!("{}.{}.{}", translated_pod.namespace, translated_pod.controller_type, translated_pod.controller);

        if controller_map.contains_key(&controller_key) {
            

            for condition in pod.status.unwrap().conditions.unwrap() {
                if condition.status.to_lowercase() == "true" && condition.type_.to_lowercase() == "ready" {
                    controller_map.get_mut(&controller_key).unwrap().inc_pods_ready();
                }
            }    
        }
        else {
            controller_map.insert(controller_key.clone(), ControllerStats::new());
        }

        controller_map.get_mut(&controller_key).unwrap().inc_pods_total();
    }

    //

    //let pod_stats = PodStats::build(pod.clone());

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

