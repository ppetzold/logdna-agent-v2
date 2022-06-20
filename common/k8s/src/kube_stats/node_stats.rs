use chrono::Local;
use k8s_openapi::api::core::v1::Node;

pub struct NodeStats {
    pub resource: String,
    pub r#type: String,
    pub age: i64,
    pub container_runtime_version: String,
    pub containers_init: i16,
    pub containers_ready: i16,
    pub containers_running: i16,
    pub containers_terminated: i16,
    pub containers_total: i16,
    pub containers_waiting: i16, 
    pub cpu_allocatable: i32,
    pub cpu_capacity: i32,
    pub cpu_usage: String,
    pub created: i64,
    pub ip_external: String,
    pub ip: String,
    pub kernel_version: String,
    pub kubelet_version: String,
    pub memory_allocatable: i32,
    pub memory_capacity: i32,
    pub memory_usage: String,
    pub node: String,
    pub os_image: String,
    pub pods_failed: i16,
    pub pods_pending: i16,
    pub pods_running: i16,
    pub pods_succeeded: i16,
    pub pods_total: i16,
    pub pods_unknown: i16,
    pub ready_heartbeat_age: i64,
    pub ready_heartbeat_time: i64,
    pub ready_message: String,
    pub ready_status: String,
    pub ready_transition_age: i64,
    pub ready_transition_time: i64,
    pub ready: bool,
    pub unschedulable: bool
}

impl NodeStats {

    pub fn new(n: &Node) -> NodeStats {

        let mut age = 0;

        let mut container_runtime_version = String::new();
        let mut ip = String::new();
        let mut ip_external = String::new();
        let mut kernel_version = String::new();
        let mut kubelet_version = String::new();
        let mut node = String::new();
        let mut os_image = String::new();
        let mut ready_message = String::new();
        let mut ready_status = String::new();

        let mut cpu_allocatable = -1;
        let mut cpu_capacity = -1;
        let mut memory_allocatable = -1;
        let mut memory_capacity = -1;
        let mut created: i64 = 0;
        let mut ready_heartbeat_age: i64 = 0;
        let mut ready_heartbeat_time: i64 = 0;
        let mut ready_transition_age: i64 = 0;
        let mut ready_transition_time: i64 = 0;

        let mut ready: bool = false;
        let mut unschedulable: bool = false;

        let status = &n.status;
        let spec = &n.spec;

        let mut containers_init = 0;
        let mut containers_ready = 0;
        let mut containers_running = 0;
        let mut containers_terminated = 0;
        let mut containers_total = 0;
        let mut containers_waiting = 0;

        let mut pods_failed = 0;
        let mut pods_pending = 0;
        let mut pods_running = 0;
        let mut pods_succeeded = 0;
        let mut pods_total = 0;
        let mut pods_unknown = 0;


        match spec {
            
            Some(spec) => {
                if spec.unschedulable.is_some() {
                    unschedulable = spec.unschedulable.as_ref().unwrap().clone();
                }

            }
            None => {},
        }

        match status {
            Some(status) => {
                
                if status.node_info.is_some() {
                    container_runtime_version = status.node_info.as_ref().unwrap().container_runtime_version.clone();
                    kernel_version = status.node_info.as_ref().unwrap().kernel_version.clone();
                    kubelet_version = status.node_info.as_ref().unwrap().kubelet_version.clone();
                    os_image = status.node_info.as_ref().unwrap().os_image.clone();
                }

                if status.allocatable.is_some() {
                    let allocatable = status.allocatable.as_ref().unwrap();
                    let cpu_quantity = allocatable.get("cpu");
                    let memory_quantity = allocatable.get("memory");

                    if cpu_quantity.is_some() {
                        cpu_allocatable = cpu_quantity.as_ref().unwrap().0.parse().unwrap_or_else(|_| -1);
                    }

                    if memory_quantity.is_some() {
                        memory_allocatable = memory_quantity.as_ref().unwrap().0.parse().unwrap_or_else(|_| -1);
                    }
                }

                if status.capacity.is_some() {
                    let allocatable = status.allocatable.as_ref().unwrap();
                    let cpu_quantity = allocatable.get("cpu");
                    let memory_quantity = allocatable.get("memory");

                    if cpu_quantity.is_some() {
                        cpu_capacity = cpu_quantity.as_ref().unwrap().0.parse().unwrap_or_else(|_| -1);
                    }

                    if memory_quantity.is_some() {
                        memory_capacity = memory_quantity.as_ref().unwrap().0.parse().unwrap_or_else(|_| -1);
                    }
                }

                if status.addresses.is_some() {
                    let addresses = status.addresses.as_ref().unwrap();

                    for address in addresses {
                        if address.type_.to_lowercase() == "internalip" {
                            ip = address.address.clone();
                        }
                        else if address.type_.to_lowercase() == "externalip" {
                            ip_external = address.address.clone();
                        }
                    }

                }

                if status.conditions.is_some() {
                    let conditions = status.conditions.as_ref().unwrap();

                    for condition in conditions {
                        if condition.type_.to_lowercase() == "ready" {
                            if condition.last_heartbeat_time.is_some() {
                                let heartbeat = condition.last_heartbeat_time.clone().unwrap();
                                ready_heartbeat_age = Local::now().signed_duration_since(heartbeat.0).num_milliseconds();
                                ready_heartbeat_time = heartbeat.0.timestamp();
                                ready_message = condition.message.as_ref().unwrap_or(&"".to_string()).clone();
                                ready_status = condition.status.clone();
                            }
                            if condition.last_transition_time.is_some() {
                                let transition = condition.last_transition_time.clone().unwrap();
                                ready_transition_age = Local::now().signed_duration_since(transition.0).num_milliseconds();
                                ready_transition_time = transition.0.timestamp();
                            }

                            ready = condition.status.to_lowercase() == "true";
                
                        }
                    }
                }
            },
            None => {},
        } 

        if n.metadata.creation_timestamp.is_some() {
            let node_created = n.metadata.creation_timestamp.clone().unwrap();
            age = Local::now().signed_duration_since(node_created.0).num_milliseconds();
            created = node_created.0.timestamp();
        }

        if n.metadata.name.is_some() {
            node = n.metadata.name.as_ref().unwrap().clone();
        }

        NodeStats {
            age,
            container_runtime_version,
            containers_init,
            containers_ready,
            containers_running,
            containers_terminated,
            containers_total,
            containers_waiting,
            cpu_allocatable,
            cpu_capacity,
            cpu_usage: String::new(),
            created,
            ip_external,
            ip,
            kernel_version,
            kubelet_version,
            memory_allocatable,
            memory_capacity,
            memory_usage: String::new(),
            node,
            os_image,
            pods_failed,
            pods_pending,
            pods_running,
            pods_succeeded,
            pods_total,
            pods_unknown,
            ready_heartbeat_age,
            ready_heartbeat_time,
            ready_message,
            ready_status,
            ready_transition_age,
            ready_transition_time,
            ready,
            unschedulable,
            resource: "node".to_string(),
            r#type: "metric".to_string()
        }
    }
}

fn skip_serializing(n: &i32) -> bool {
    n.is_negative()
}