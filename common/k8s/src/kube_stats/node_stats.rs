use chrono::Local;
use k8s_openapi::api::core::v1::Node;

pub struct NodeStats {
    resource: String,
    r#type: String,
    age: i64,
    container_runtime_version: String,
    containers_init: i16,
    containers_ready: i16,
    containers_running: i16,
    containers_terminated: i16,
    containers_total: i16,
    containers_waiting: i16, 
    cpu_allocatable: String,
    cpu_capacity: String,
    cpu_usage: String,
    created: String,
    ip_external: String,
    ip: String,
    kernel_version: String,
    kubelet_version: String,
    memory_allocatable: String,
    memory_capacity: String,
    memory_usage: String,
    node: String,
    os_image: String,
    pods_failed: String,
    pods_pending: i16,
    pods_running: i16,
    pods_succeeded: i16,
    pods_total: i16,
    pods_unknown: i16,
    ready_heartbeat_age: String,
    ready_heartbeat_time: String,
    ready_message: String,
    ready_status: String,
    ready_transition_age: String,
    ready_transition_time: String,
    ready: String,
    unschedulable: String
}

impl NodeStats {

    pub fn new(n: &Node) -> NodeStats {

        let mut age = 0;

        let mut container_runtime_version = String::new();

        let spec = &n.spec;
        let status = &n.status;

        match spec {
            Some(spec) => {

            },
            None => {}
        }

        match status {
            Some(status) => {
                
                if status.node_info.is_some() {
                    container_runtime_version = status.node_info.as_ref().unwrap().container_runtime_version.clone();
                }
            },
            None => {},
        }

        if n.metadata.creation_timestamp.is_some() {
            let node_created = n.metadata.creation_timestamp.clone().unwrap();
            age = Local::now().signed_duration_since(node_created.0).num_milliseconds();
        }

        NodeStats {
            age,
            container_runtime_version,
            containers_init: todo!(),
            containers_ready: todo!(),
            containers_running: todo!(),
            containers_terminated: todo!(),
            containers_total: todo!(),
            containers_waiting: todo!(),
            cpu_allocatable: todo!(),
            cpu_capacity: todo!(),
            cpu_usage: todo!(),
            created: todo!(),
            ip_external: todo!(),
            ip: todo!(),
            kernel_version: todo!(),
            kubelet_version: todo!(),
            memory_allocatable: todo!(),
            memory_capacity: todo!(),
            memory_usage: todo!(),
            node: todo!(),
            os_image: todo!(),
            pods_failed: todo!(),
            pods_pending: todo!(),
            pods_running: todo!(),
            pods_succeeded: todo!(),
            pods_total: todo!(),
            pods_unknown: todo!(),
            ready_heartbeat_age: todo!(),
            ready_heartbeat_time: todo!(),
            ready_message: todo!(),
            ready_status: todo!(),
            ready_transition_age: todo!(),
            ready_transition_time: todo!(),
            ready: todo!(),
            unschedulable: todo!(),
            resource: "node".to_string(),
            r#type: "metric".to_string()
        }
    }
}