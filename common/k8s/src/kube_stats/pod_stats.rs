use chrono::{Utc, DateTime, NaiveDate, Local};
use k8s_openapi::{api::core::v1::{Pod,}, apimachinery::pkg::apis::meta::v1::{OwnerReference, Time}};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct PodStats {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub controller: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub controller_type: String,
    pub created: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ip: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub namespace: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub node: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub phase: String,
    pub pod_age: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub pod: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub priority_class: String,
    pub priority: i32,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub qos_class: String,
    pub resource: String,
    pub r#type: String,
}


impl PodStats {
    pub fn new(p: &Pod) -> Self {
        let details = get_controller_details(&p.metadata.owner_references);

        let controller = details.0;
        let controller_type = details.1;

        let spec = &p.spec;
        let status = &p.status;

        let mut created: i64 = 0;
        let mut priority_class = String::new();
        let mut node = String::new();
        let mut priority = -1;

        let mut ip = String::new();
        let mut phase = String::new();
        let mut qos_class = String::new();
        let mut pod_age = 0;

        let namespace = p.metadata.namespace.clone().unwrap_or_else(|| "".to_string());
        let pod = p.metadata.name.clone().unwrap_or("".to_string());

        match spec {
            Some(spec) => {
                if spec.priority.is_some() {
                    priority = spec.priority.unwrap();
                }

                if spec.priority_class_name.is_some() {
                    priority_class = spec.priority_class_name.clone().unwrap();
                }

                if spec.node_name.is_some() {
                    node = spec.node_name.clone().unwrap();
                }
            },
            None => {}
        }

        match status {
            Some(status) => {
                if status.start_time.is_some() {
                    let pod_created = status.start_time.clone().unwrap();
                    pod_age = Local::now().signed_duration_since(pod_created.0).num_milliseconds();
                    created = pod_created.0.timestamp();
                }

                if status.pod_ip.is_some() {
                    ip = status.pod_ip.clone().unwrap();
                }

                if status.phase.is_some() {
                    phase = status.phase.clone().unwrap();
                }

                if status.qos_class.is_some() {
                    qos_class = status.qos_class.clone().unwrap();
                }
            },
            None => {},
        }

        PodStats {
            controller,
            controller_type,
            created,
            ip,
            namespace,
            node,
            phase,
            pod_age,
            pod,
            priority_class,
            priority,
            qos_class,
            resource: "container".to_string(),
            r#type: "metric".to_string()
        }

    }
}


fn get_controller_details(owners: &Option<Vec<OwnerReference>>) -> (String, String)
{
    if owners.is_some() {
        for owner in owners.as_ref().unwrap() {

            if owner.controller == Some(true) {
                return (owner.name.clone(), owner.kind.clone());
            }
        }
    }

    return ("".to_string(), "".to_string());
}

#[derive(Debug)]
pub struct NodePodStats {
    pub pods_failed: i32,
    pub pods_pending: i32,
    pub pods_running: i32,
    pub pods_succeeded: i32,
    pub pods_unknown: i32,
    pub pods_total: i32
}

impl NodePodStats {
    pub fn new() -> Self {

        NodePodStats { 
            pods_failed: 0, 
            pods_pending: 0, 
            pods_running: 0, 
            pods_succeeded: 0, 
            pods_unknown: 0,
            pods_total: 0
        }
    }

    pub fn inc(&mut self, phase: &str) {

        self.pods_total += 1;

        match phase.to_lowercase().as_str() {
            "failed" => {
                self.pods_failed += 1;
            }
            "pending" => {
                self.pods_pending += 1;
            }
            "running" => {
                self.pods_running += 1;
            }
            "succeeded" => {
                self.pods_succeeded += 1;
            }
            "unknown" => {
                self.pods_unknown += 1;
            }
            _ => {
                self.pods_unknown += 1;
            }
        }

    }

}