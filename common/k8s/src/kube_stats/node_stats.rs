use chrono::Local;
use k8s_openapi::api::core::v1::Node;
use serde::{Deserialize, Serialize};

use super::helpers::{
    convert_cpu_usage_to_milli, convert_memory_usage_to_bytes, skip_serializing_int64,
};

const CPU_MULTIPLIER: i64 = 1000;

#[derive(Serialize, Deserialize)]
pub struct NodeStats {
    pub resource: String,
    pub r#type: String,
    pub age: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub container_runtime_version: String,
    pub containers_init: i32,
    pub containers_ready: i32,
    pub containers_running: i32,
    pub containers_terminated: i32,
    pub containers_total: i32,
    pub containers_waiting: i32,
    #[serde(skip_serializing_if = "skip_serializing_int64")]
    pub cpu_allocatable: i64,
    #[serde(skip_serializing_if = "skip_serializing_int64")]
    pub cpu_capacity: i64,
    pub cpu_usage: i32,
    pub created: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ip_external: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ip: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub kernel_version: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub kubelet_version: String,
    #[serde(skip_serializing_if = "skip_serializing_int64")]
    pub memory_allocatable: i64,
    #[serde(skip_serializing_if = "skip_serializing_int64")]
    pub memory_capacity: i64,
    #[serde(skip_serializing_if = "skip_serializing_int64")]
    pub pods_allocatable: i64,
    #[serde(skip_serializing_if = "skip_serializing_int64")]
    pub pods_capacity: i64,
    pub memory_usage: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub node: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub os_image: String,
    pub pods_failed: i32,
    pub pods_pending: i32,
    pub pods_running: i32,
    pub pods_succeeded: i32,
    pub pods_total: i32,
    pub pods_unknown: i32,
    #[serde(skip_serializing_if = "skip_serializing_int64")]
    pub ready_heartbeat_age: i64,
    #[serde(skip_serializing_if = "skip_serializing_int64")]
    pub ready_heartbeat_time: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ready_message: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ready_status: String,
    #[serde(skip_serializing_if = "skip_serializing_int64")]
    pub ready_transition_age: i64,
    #[serde(skip_serializing_if = "skip_serializing_int64")]
    pub ready_transition_time: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "::serde_with::rust::unwrap_or_skip")]
    pub ready: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "::serde_with::rust::unwrap_or_skip")]
    pub unschedulable: Option<bool>,
}

impl NodeStats {
    pub fn builder<'a>(
        n: &'a Node,
        n_pods: &'a NodePodStats,
        n_containers: &'a NodeContainerStats,
        raw_cpu_usage: &'a str,
        raw_memory_usage: &'a str,
    ) -> NodeStatsBuilder<'a> {
        NodeStatsBuilder {
            n,
            n_pods,
            n_containers,
            raw_cpu_usage,
            raw_memory_usage,
        }
    }
}

pub struct NodeStatsBuilder<'a> {
    n: &'a Node,
    n_pods: &'a NodePodStats,
    n_containers: &'a NodeContainerStats,
    raw_cpu_usage: &'a str,
    raw_memory_usage: &'a str,
}

impl NodeStatsBuilder<'_> {

    pub fn new<'a>(
        n: &'a Node,
        n_pods: &'a NodePodStats,
        n_containers: &'a NodeContainerStats,
        raw_cpu_usage: &'a str,
        raw_memory_usage: &'a str,
    ) -> NodeStatsBuilder<'a> {
        NodeStatsBuilder {
            n,
            n_pods,
            n_containers,
            raw_cpu_usage,
            raw_memory_usage,
        }
    }

    pub fn build(self) -> NodeStats {
        let mut age = 0;

        let memory_usage = convert_memory_usage_to_bytes(self.raw_memory_usage);
        let cpu_usage = convert_cpu_usage_to_milli(self.raw_cpu_usage);

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
        let mut pods_allocatable = -1;
        let mut pods_capacity = -1;
        let mut created: i64 = 0;
        let mut ready_heartbeat_age: i64 = 0;
        let mut ready_heartbeat_time: i64 = 0;
        let mut ready_transition_age: i64 = 0;
        let mut ready_transition_time: i64 = 0;

        let mut ready: Option<bool> = None;
        let mut unschedulable: Option<bool> = None;

        let status = &self.n.status;
        let spec = &self.n.spec;

        let containers_init = 0;
        let containers_ready = self.n_containers.containers_ready;
        let containers_running = self.n_containers.containers_running;
        let containers_terminated = self.n_containers.containers_terminated;
        let containers_total = self.n_containers.containers_total;
        let containers_waiting = self.n_containers.containers_waiting;

        let pods_failed = self.n_pods.pods_failed;
        let pods_pending = self.n_pods.pods_pending;
        let pods_running = self.n_pods.pods_running;
        let pods_succeeded = self.n_pods.pods_succeeded;
        let pods_total = self.n_pods.pods_total;
        let pods_unknown = self.n_pods.pods_unknown;

        match spec {
            Some(spec) => {
                if spec.unschedulable.is_some() {
                    unschedulable = Some(spec.unschedulable.as_ref().unwrap().clone());
                }
            }
            None => {}
        }

        match status {
            Some(status) => {
                if status.node_info.is_some() {
                    container_runtime_version = status
                        .node_info
                        .as_ref()
                        .unwrap()
                        .container_runtime_version
                        .clone();
                    kernel_version = status.node_info.as_ref().unwrap().kernel_version.clone();
                    kubelet_version = status.node_info.as_ref().unwrap().kubelet_version.clone();
                    os_image = status.node_info.as_ref().unwrap().os_image.clone();
                }

                if status.allocatable.is_some() {
                    let allocatable = status.allocatable.as_ref().unwrap();
                    let cpu_quantity = allocatable.get("cpu");
                    let memory_quantity = allocatable.get("memory");
                    let pods_quantity = allocatable.get("pods");

                    if cpu_quantity.is_some() {
                        cpu_allocatable = cpu_quantity
                            .as_ref()
                            .unwrap()
                            .0
                            .parse()
                            .unwrap_or_else(|_| -1)
                            * CPU_MULTIPLIER;
                    }

                    if memory_quantity.is_some() {
                        let memory_allocatable_str = memory_quantity.as_ref().unwrap().0.as_str();
                        memory_allocatable = convert_memory_usage_to_bytes(memory_allocatable_str);
                    }

                    if pods_quantity.is_some() {
                        pods_allocatable = pods_quantity
                            .as_ref()
                            .unwrap()
                            .0
                            .parse()
                            .unwrap_or_else(|_| -1);
                    }
                }

                if status.capacity.is_some() {
                    let capacity = status.capacity.as_ref().unwrap();
                    let cpu_quantity = capacity.get("cpu");
                    let memory_quantity = capacity.get("memory");
                    let pods_quantity = capacity.get("pods");

                    if cpu_quantity.is_some() {
                        cpu_capacity = cpu_quantity
                            .as_ref()
                            .unwrap()
                            .0
                            .parse()
                            .unwrap_or_else(|_| -1)
                            * CPU_MULTIPLIER;
                    }

                    if memory_quantity.is_some() {
                        let memory_capacity_str = memory_quantity.as_ref().unwrap().0.as_str();
                        memory_capacity = convert_memory_usage_to_bytes(memory_capacity_str);
                    }

                    if pods_quantity.is_some() {
                        pods_capacity = pods_quantity
                            .as_ref()
                            .unwrap()
                            .0
                            .parse()
                            .unwrap_or_else(|_| -1);
                    }
                }

                if status.addresses.is_some() {
                    let addresses = status.addresses.as_ref().unwrap();

                    for address in addresses {
                        if address.type_.to_lowercase() == "internalip" {
                            ip = address.address.clone();
                        } else if address.type_.to_lowercase() == "externalip" {
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
                                ready_heartbeat_age = Local::now()
                                    .signed_duration_since(heartbeat.0)
                                    .num_milliseconds();
                                ready_heartbeat_time = heartbeat.0.timestamp();
                                ready_message = condition
                                    .message
                                    .as_ref()
                                    .unwrap_or(&"".to_string())
                                    .clone();
                                ready_status = condition.status.clone();
                            }
                            if condition.last_transition_time.is_some() {
                                let transition = condition.last_transition_time.clone().unwrap();
                                ready_transition_age = Local::now()
                                    .signed_duration_since(transition.0)
                                    .num_milliseconds();
                                ready_transition_time = transition.0.timestamp();
                            }

                            ready = Some(condition.status.to_lowercase() == "true");
                        }
                    }
                }
            }
            None => {}
        }

        if self.n.metadata.creation_timestamp.is_some() {
            let node_created = self.n.metadata.creation_timestamp.clone().unwrap();
            age = Local::now()
                .signed_duration_since(node_created.0)
                .num_milliseconds();
            created = node_created.0.timestamp();
        }

        if self.n.metadata.name.is_some() {
            node = self.n.metadata.name.as_ref().unwrap().clone();
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
            cpu_usage,
            created,
            ip_external,
            ip,
            kernel_version,
            kubelet_version,
            memory_allocatable,
            memory_capacity,
            memory_usage,
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
            pods_allocatable,
            pods_capacity,
            resource: "node".to_string(),
            r#type: "metric".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct NodeContainerStats {
    pub containers_waiting: i32,
    pub containers_total: i32,
    pub containers_terminated: i32,
    pub containers_running: i32,
    pub containers_ready: i32,
    pub containers_init: i32,
}

impl NodeContainerStats {
    pub fn new() -> Self {
        NodeContainerStats {
            containers_waiting: 0,
            containers_total: 0,
            containers_terminated: 0,
            containers_running: 0,
            containers_ready: 0,
            containers_init: 0,
        }
    }

    pub fn inc(&mut self, state: &str, ready: bool, init: bool) {
        if init {
            self.containers_init += 1;
        }

        match state.to_lowercase().as_str() {
            "waiting" => {
                self.containers_waiting += 1;
                self.containers_total += 1;
            }
            "terminated" => {
                self.containers_terminated += 1;
                self.containers_total += 1;
            }
            "running" => {
                self.containers_running += 1;
                self.containers_total += 1;

                if ready {
                    self.containers_ready += 1;
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct NodePodStats {
    pub pods_failed: i32,
    pub pods_pending: i32,
    pub pods_running: i32,
    pub pods_succeeded: i32,
    pub pods_unknown: i32,
    pub pods_total: i32,
}

impl NodePodStats {
    pub fn new() -> Self {
        NodePodStats {
            pods_failed: 0,
            pods_pending: 0,
            pods_running: 0,
            pods_succeeded: 0,
            pods_unknown: 0,
            pods_total: 0,
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
