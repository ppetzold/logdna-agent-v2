use k8s_openapi::api::core::v1::Pod;

pub struct ControllerStats {
    containers_ready: String,
    containers_total: String,
    pods_ready: String,
    pods_total: String
}

impl ControllerStats {

    pub fn build(p: Pod) -> ControllerStats {
        ControllerStats {
            containers_ready: todo!(),
            containers_total: todo!(),
            pods_ready: todo!(),
            pods_total: todo!(),
        }
    }
}