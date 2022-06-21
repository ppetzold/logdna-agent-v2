use k8s_openapi::api::core::v1::Pod;

pub struct ClusterStats {
    containers_init: i16,
    containers_ready: i16,
    containers_running: i16,
    containers_terminated: i16,
    containers_total: i16,
    containers_waiting: i16,
    cpu_allocatable: i16,
    cpu_capacity: i16,
    cpu_usage: i16,
    memory_allocatable: i16,
    memory_capacity: i16,
    memory_usage: i16,
    nodes_notready: i16,
    nodes_ready: i16,
    nodes_total: i16,
    nodes_unschedulable: i16,
    pods_allocatable: i16,
    pods_capacity: i16,
    pods_failed: i16,
    pods_pending: i16,
    pods_running: i16,
    pods_succeeded: i16,
    pods_unknown: i16,
    pods_total: i16,
}

impl ClusterStats {
    pub fn build(p: Pod) -> ClusterStats {
        ClusterStats {
            containers_init: todo!(),
            containers_ready: todo!(),
            containers_running: todo!(),
            containers_terminated: todo!(),
            containers_total: todo!(),
            containers_waiting: todo!(),
            cpu_allocatable: todo!(),
            cpu_capacity: todo!(),
            cpu_usage: todo!(),
            memory_allocatable: todo!(),
            memory_capacity: todo!(),
            memory_usage: todo!(),
            nodes_notready: todo!(),
            nodes_ready: todo!(),
            nodes_total: todo!(),
            nodes_unschedulable: todo!(),
            pods_allocatable: todo!(),
            pods_capacity: todo!(),
            pods_failed: todo!(),
            pods_pending: todo!(),
            pods_running: todo!(),
            pods_succeeded: todo!(),
            pods_unknown: todo!(),
            pods_total: todo!(),
        }
    }
}
