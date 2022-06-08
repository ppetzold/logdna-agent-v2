use k8s_openapi::api::core::v1::Pod;

pub struct ContainerStats {
    container_age: String,
    container: String,
    cpu_limit: i16,
    cpu_request: i16,
    cpu_usage: i16,
    image_tag: String,
    image: String,
    last_finished: String,
    last_reason: String,
    last_started: String,
    last_state: String,
    memory_limit: i32,
    memory_request: i32,
    memory_usage: i32,
    ready: String,
    restarts: i16,
    started: String,
    state: String
}

impl ContainerStats {

    pub fn build(p: Pod) -> ContainerStats {
        ContainerStats {
            container_age: todo!(),
            container: todo!(),
            cpu_limit: todo!(),
            cpu_request: todo!(),
            cpu_usage: todo!(),
            image_tag: todo!(),
            image: todo!(),
            last_finished: todo!(),
            last_reason: todo!(),
            last_started: todo!(),
            last_state: todo!(),
            memory_limit: todo!(),
            memory_request: todo!(),
            memory_usage: todo!(),
            ready: todo!(),
            restarts: todo!(),
            started: todo!(),
            state: todo!(),
        }
    }
}