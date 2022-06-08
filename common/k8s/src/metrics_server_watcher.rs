use anyhow:: {Result, Ok };

use futures::{stream, StreamExt};
use k8s_openapi::{api::core::v1::{Pod, Node}};
use kube::{
    api::{Api, DynamicObject, GroupVersionKind, ListParams, ObjectList},
    discovery::{self},
    Client
};

use std:: time::{Duration};
use tokio::time::{self as tokio_time, sleep};

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
            sleep(Duration::from_millis(30000)).await;
            gather_reporter_info(self.client.clone()).await.unwrap();
        }

        /* 
        let forever = stream::unfold(interval, |mut interval| async {
            interval.tick().await;
            gather_reporter_info(self.client.clone()).await.unwrap();
            Some(((), interval))
        });

        */

        //forever.for_each(|_| async {}).await;
    }
}

async fn gather_reporter_info(client: Client) -> anyhow::Result<()> {

    let pod_metrics = self::call_metric_api(&"PodMetrics", client.clone()).await?;
    let node_metrics = self::call_metric_api(&"NodeMetrics", client.clone()).await?;

    let pods_info = self::get_all_pods(client.clone()).await?;
    let node_info = self::get_all_nodes(client.clone()).await?;

    for node in node_info {
        info!("Pod status {:?}", node.status);
        info!("Pod Spec {:?}", node.spec);
        info!("Pod metadata {:?}", node.metadata);
    }
   
    for pod in pods_info {
        info!("Pod status {:?}", pod.status);
        info!("Pod Spec {:?}", pod.spec);
        info!("Pod metadata {:?}", pod.metadata);
    }

    Ok(())
}

async fn call_metric_api(kind: &str, client: Client) -> Result<ObjectList<DynamicObject>, kube::Error> {
    info!("Get Metrics");
    let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", kind);
    let (ar, _caps) = discovery::pinned_kind(&client, &gvk).await?;
    let api = Api::<DynamicObject>::all_with(client, &ar);

    let items = api.list(&ListParams::default()).await;

    items
}

async fn get_all_nodes(client: Client) -> Result<ObjectList<Node>, kube::Error> {
    info!("In Node Stats");
    let api: Api<Node> = Api::all(client);
    let nodes = api.list(&ListParams::default()).await;

    nodes
}

async fn get_all_pods(client: Client) -> Result<ObjectList<Pod>, kube::Error>{
    info!("In Pod Stats");
    let api: Api<Pod> = Api::all(client);
    let pods = api.list(&ListParams::default()).await;

    pods
}

