use chrono::{Utc, DateTime, NaiveDate};
use k8s_openapi::{api::core::v1::{Pod}, apimachinery::pkg::apis::meta::v1::{OwnerReference, Time}};

#[derive(Debug)]
pub struct PodStats {
    controller: String,
    controller_type: String,
    created: Time,
    ip: String,
    namespace: String,
    node: String,
    phase: String,
    pod_age: String,
    pod: String,
    priority_class: String,
    priority: String,
    qos_class: String,
    resource: String,
    r#type: String,
}

impl PodStats {

    pub fn build(p: Pod) -> PodStats {

        let default_naive_date = NaiveDate::from_ymd(0001, 1, 1).and_hms(0, 0, 0);
        let default_date_time = Time(DateTime::<Utc>::from_utc(default_naive_date, Utc));

        let details = get_controller_details(p.metadata.owner_references);

        let spec = p.spec;
        let status = p.status;

        let mut priority: i32 = -1;
        let mut priority_class_name: String = String::new();
        let mut node_name: String = String::new();

        let mut created: Time = default_date_time;
        let mut ip: String = String::new();
        let mut phase: String = String::new();
        let mut qos_class: String = String::new();

        match spec {
            Some(spec) => {

                if spec.priority.is_some() {
                    priority = spec.priority.unwrap();
                }

                if spec.priority_class_name.is_some() {
                    priority_class_name = spec.priority_class_name.unwrap();
                }

                if spec.node_name.is_some() {
                    node_name = spec.node_name.unwrap();
                }
            },
            None => {}
        }

        match status {
            Some(status) => {

                if status.start_time.is_some() {
                    created = status.start_time.unwrap();
                }

                if status.pod_ip.is_some() {
                    ip = status.pod_ip.unwrap();
                }

                if status.phase.is_some() {
                    phase = status.phase.unwrap();
                }

                if status.qos_class.is_some() {
                    qos_class = status.qos_class.unwrap();
                }
            },
            None => {},
        }

        PodStats {
            controller: details.0,
            controller_type: details.1,
            created,
            ip,
            namespace: p.metadata.namespace.unwrap_or("".to_string()),
            node: node_name,
            phase,
            pod_age: "".to_string(), //TODO
            pod: p.metadata.name.unwrap_or("".to_string()),
            priority_class: priority_class_name,
            priority: priority.to_string(),
            qos_class,
            resource: "container".to_string(),
            r#type: "metric".to_string()
        }

    }

    
}

fn get_controller_details(owners: Option<Vec<OwnerReference>>) -> (String, String)
{
    if !owners.is_some() {
        return ("".to_string(), "".to_string());
    }

    for owner in owners.unwrap() {

        if owner.controller == Some(true) {
            return (owner.name, owner.kind);
        }
    }

    return ("".to_string(), "".to_string());
}