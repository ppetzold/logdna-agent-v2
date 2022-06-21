use chrono::{Local};
use k8s_openapi::{api::{core::v1::{Container, ContainerStatus, ContainerState}}};
use serde::{Serialize, Deserialize};

use super::helpers::{
    convert_memory_usage_to_bytes, convert_cpu_usage_to_milli, skip_serializing_int64, skip_serializing_int32
};

#[derive(Serialize, Deserialize)]
pub struct ContainerStats {
    pub container_age: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub container: String,
    #[serde(skip_serializing_if = "skip_serializing_int32")]
    pub cpu_limit: i32,
    #[serde(skip_serializing_if = "skip_serializing_int32")]
    pub cpu_request: i32,
    pub cpu_usage: i32,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub image_tag: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub image: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub last_finished: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub last_reason: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub last_started: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub last_state: String,
    #[serde(skip_serializing_if = "skip_serializing_int64")]
    pub memory_limit: i64,
    #[serde(skip_serializing_if = "skip_serializing_int64")]
    pub memory_request: i64,
    pub memory_usage: i64,
    pub ready: bool,
    pub restarts: i32,
    pub started: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub state: String
}

impl ContainerStats {
    pub fn builder<'a>(
        c: &'a Container, 
        c_status: &'a ContainerStatus, 
        c_state: &'a ContainerState, 
        raw_cpu_usage: &'a str, 
        raw_memory_usage: &'a str
    ) -> ContainerStatsBuilder<'a> {
        ContainerStatsBuilder {
            c,
            c_status,
            c_state,
            raw_cpu_usage,
            raw_memory_usage
        }
    }
}

pub struct ContainerStatsBuilder<'a> {
    c: &'a Container, 
    c_status: &'a ContainerStatus, 
    c_state: &'a ContainerState, 
    raw_cpu_usage: &'a str, 
    raw_memory_usage: &'a str
}

impl ContainerStatsBuilder<'_> {
    pub fn new<'a>(
        c: &'a Container, 
        c_status: &'a ContainerStatus, 
        c_state: &'a ContainerState, 
        raw_cpu_usage: &'a str, 
        raw_memory_usage: &'a str
    ) -> ContainerStatsBuilder<'a> {
        ContainerStatsBuilder {
            c,
            c_status,
            c_state,
            raw_cpu_usage,
            raw_memory_usage
        }
    }

    pub fn build(self) -> ContainerStats {
        let container = self.c.name.clone();

        let memory_usage = convert_memory_usage_to_bytes(&self.raw_memory_usage);
        let cpu_usage = convert_cpu_usage_to_milli(&self.raw_cpu_usage);

        let mut image = String::new();
        let mut image_tag = String::new();
        let state;
        let mut last_state = String::new();
        let mut last_reason = String::new();
        let mut last_started = String::new();
        let mut last_finished = String::new();

        let mut cpu_limit = -1;
        let mut cpu_request = -1;
        let mut memory_limit = -1;
        let mut memory_request = -1;

        let mut container_age: i64 = 0;

        let restarts =  self.c_status.restart_count;
        let ready = self.c_status.ready;
        let started = self.c_status.started.unwrap_or_else(|| false);

        if self.c.image.is_some()
        {
            let container_image = self.c.image.clone().unwrap();

            let split: Vec<&str> = container_image.split(":").collect();

            if split.len() == 2 {
                image = split[0].to_string();
                image_tag = split[1].to_string();
            }
        }

        let running = self.c_state.running.as_ref();
        let terminated = self.c_state.terminated.as_ref();

        if running.is_some() {
            state = "Running".to_string();
            let started_at = self.c_state.running.as_ref().unwrap().started_at.as_ref().unwrap().0; // TODO need a default 
            container_age = Local::now().signed_duration_since(started_at).num_milliseconds();
        }
        else if terminated.is_some() {
            state = "Terminated".to_string()
        }
        else {
            state = "Waiting".to_string()
        }

        let last_status_state = self.c_status.last_state.as_ref();

        if last_status_state.is_some() {

            let last_running = self.c_state.running.as_ref();
            let last_terminated = self.c_state.terminated.as_ref();    

            if last_running.is_some() {
                last_state = "Running".to_string();
                let last_running_ref = last_running.as_ref().unwrap();

                if last_running_ref.started_at.is_some() {
                    last_started = last_running_ref.started_at.as_ref().unwrap().0.to_string(); 
                } 
            }
            else if last_terminated.is_some() {
                last_state = "Terminated".to_string();
                 
                let last_terminated_ref = last_terminated.as_ref().unwrap();

                if last_terminated_ref.reason.is_some() {
                    last_reason = last_terminated_ref.reason.as_ref().unwrap().to_string();
                }

                if last_terminated_ref.finished_at.is_some() {
                    last_finished = last_terminated_ref.finished_at.as_ref().unwrap().0.to_string();
                }

                if last_terminated_ref.started_at.is_some() {
                    last_started = last_terminated_ref.started_at.as_ref().unwrap().0.to_string(); 
                }                
            } 
            else {
                last_state = "Waiting".to_string()
            }
        }

        // If same as previous state - we don't want to print 
        if last_state.eq(&state){
            last_state = "".to_string();
            last_reason = "".to_string();
            last_finished =  "".to_string();
            last_started = "".to_string();

        }

        let resources = self.c.resources.as_ref();

        if resources.is_some() {
            let limits = resources.unwrap().limits.as_ref();

            if limits.is_some() {
                let cpu = limits.unwrap().get("cpu");
                let memory = limits.unwrap().get("memory");

                if cpu.is_some() {
                    cpu_limit = convert_cpu_usage_to_milli(cpu.unwrap().0.as_str());
                }

                if memory.is_some() {
                    memory_limit = convert_memory_usage_to_bytes(memory.unwrap().0.as_str());
                }
            }

            let requests = resources.unwrap().requests.as_ref();

            if requests.is_some() {
                let cpu = requests.unwrap().get("cpu");
                let memory = requests.unwrap().get("memory");

                if cpu.is_some() {
                    cpu_request= convert_cpu_usage_to_milli(cpu.unwrap().0.as_str());
                }

                if memory.is_some() {
                    memory_request = convert_memory_usage_to_bytes(memory.unwrap().0.as_str());
                }
            }
        }

        ContainerStats {
            container_age,
            container,
            cpu_limit,
            cpu_request,
            cpu_usage,
            image_tag,
            image,
            last_finished,
            last_reason,
            last_started,
            last_state,
            memory_limit,
            memory_request,
            memory_usage,
            ready,
            restarts,
            started,
            state
        }
    }
}