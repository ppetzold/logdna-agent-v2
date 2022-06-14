use k8s_openapi::api::core::v1::{Container, ContainerStatus};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ContainerStats {
    pub container_age: String,
    pub container: String,
    pub cpu_limit: i16,
    pub cpu_request: i16,
    pub cpu_usage: String,
    pub image_tag: String,
    pub image: String,
    pub last_finished: String,
    pub last_reason: String,
    pub last_started: String,
    pub last_state: String,
    pub memory_limit: i32,
    pub memory_request: i32,
    pub memory_usage: String,
    pub ready: String,
    pub restarts: i16,
    pub started: String,
    pub state: String
}

impl ContainerStats {

    pub fn build(c: &Container, s: &ContainerStatus, cpu_usage: String, memory_usage: String) -> ContainerStats {

        let container = c.name.clone();
        let mut image = String::new();
        let mut image_tag = String::new();
        let cpu_request = 0; // TODO 
        let cpu_limit = 0;
        let memory_request = 0;
        let memory_limit = 0;

        if c.image.is_some()
        {
            image = c.image.clone().unwrap();

            let split_image: Vec<&str> = c.image.as_ref().unwrap().split(":").collect();
            image_tag = split_image[0].to_string();
        }


        ContainerStats {
            container_age: String::new(),
            container: String::new(),
            cpu_limit: -1,
            cpu_request,
            cpu_usage : cpu_usage,
            image_tag,
            image: image,
            last_finished: String::new(),
            last_reason: String::new(),
            last_started: String::new(),
            last_state: String::new(),
            memory_limit,
            memory_request,
            memory_usage: memory_usage,
            ready: String::new(),
            restarts: 0,
            started: String::new(),
            state: String::new(),
        }
    }
}