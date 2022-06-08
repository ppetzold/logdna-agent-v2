use k8s_openapi::api::core::v1::Pod;

pub struct PodStats{
    controller: String,
    controller_type: String,
    created: String,
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
    r#type: String
}

impl PodStats {

    pub fn populate_pod_stats(p: Pod) -> PodStats{

        p.metadata
    }
}