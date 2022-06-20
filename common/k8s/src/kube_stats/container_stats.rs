use chrono::{Local};
use k8s_openapi::{api::{core::v1::{Container, ContainerStatus, ContainerState}}};
use serde::{Serialize, Deserialize};

use super::helpers::{convert_memory_usage_to_bytes, convert_cpu_usage_to_milli};

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
    pub fn new(c: &Container, c_status: &ContainerStatus, c_state: &ContainerState, raw_cpu_usage: &str, raw_memory_usage: &str) -> ContainerStats {
        let container = c.name.clone();

        let memory_usage = convert_memory_usage_to_bytes(&raw_memory_usage);
        let cpu_usage = convert_cpu_usage_to_milli(&raw_cpu_usage);

        let mut image = String::new();
        let mut image_tag = String::new();
        let mut state = String::new();
        let mut last_state = String::new();
        let mut last_reason = String::new();
        let mut last_started = String::new();
        let mut last_finished = String::new();

        let mut cpu_limit = -1;
        let mut cpu_request = -1;
        let mut memory_limit = -1;
        let mut memory_request = -1;

        let mut container_age: i64 = 0;

        let restarts =  c_status.restart_count;
        let ready = c_status.ready;
        let started = c_status.started.unwrap_or_else(|| false);

        if c.image.is_some()
        {
            let container_image = c.image.clone().unwrap();

            let split: Vec<&str> = container_image.split(":").collect();

            if split.len() == 2 {
                image = split[0].to_string();
                image_tag = split[1].to_string();
            }
        }

        let running = c_state.running.as_ref();
        let terminated = c_state.terminated.as_ref();

        if running.is_some() {
            state = "Running".to_string();
            let started_at = c_state.running.as_ref().unwrap().started_at.as_ref().unwrap().0;
            container_age = Local::now().signed_duration_since(started_at).num_milliseconds();
        }
        else if terminated.is_some() {
            state = "Terminated".to_string()
        }
        else {
            state = "Waiting".to_string()
        }

        let last_status_state = c_status.last_state.as_ref();

        if last_status_state.is_some() {

            let last_running = c_state.running.as_ref();
            let last_terminated = c_state.terminated.as_ref();    

            if last_running.is_some() {
                last_state = "Running".to_string();
                last_started = last_running.as_ref().unwrap().started_at.as_ref().unwrap().0.to_string();
            }
            else if last_terminated.is_some() {
                last_state = "Terminated".to_string();
                last_reason = last_terminated.as_ref().unwrap().reason.as_ref().unwrap().to_string();
                last_finished =  last_terminated.as_ref().unwrap().finished_at.as_ref().unwrap().0.to_string();
                last_started = last_terminated.as_ref().unwrap().started_at.as_ref().unwrap().0.to_string();
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

        let resources = c.resources.as_ref();

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

fn skip_serializing_int32(n: &i32) -> bool {
    n.is_negative()
}

fn skip_serializing_int64(n: &i64) -> bool {
    n.is_negative()
}

#[derive(Debug)]
pub struct NodeContainerStats {
    pub containers_waiting: i32,
    pub containers_total: i32,
    pub containers_terminated: i32,
    pub containers_running: i32,
    pub containers_ready: i32,
    pub containers_init: i32
}

impl NodeContainerStats {
    pub fn new() -> Self {

        NodeContainerStats { 
            containers_waiting: 0, 
            containers_total: 0, 
            containers_terminated: 0, 
            containers_running: 0, 
            containers_ready: 0,
            containers_init: 0
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