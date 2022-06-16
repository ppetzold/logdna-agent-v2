use chrono::{Local};
use k8s_openapi::api::{core::v1::{Container, ContainerStatus, ContainerState}, batch::v1::CronJobSpec};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ContainerStats {
    pub container_age: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub container: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub cpu_limit: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub cpu_request: String,
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
    #[serde(skip_serializing_if = "String::is_empty")]
    pub memory_limit: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub memory_request: String,
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

        let memory_usage = self::convert_memory_usage_to_bytes(&raw_memory_usage);
        let cpu_usage = self::convert_cpu_usage_to_milli(&raw_cpu_usage);

        let mut image = String::new();
        let mut image_tag = String::new();
        let mut state = String::new();
        let mut last_state = String::new();
        let mut last_reason = String::new();
        let mut last_started = String::new();
        let mut last_finished = String::new();
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

        // If same - we don't want to print 
        if last_state.eq(&state){
            last_state = "".to_string();
            last_reason = "".to_string();
            last_finished =  "".to_string();
            last_started = "".to_string();

        }

        ContainerStats {
            container_age,
            container,
            cpu_limit: "".to_string(),
            cpu_request: "".to_string(),
            cpu_usage,
            image_tag,
            image,
            last_finished,
            last_reason,
            last_started,
            last_state,
            memory_limit: "".to_string(),
            memory_request: "".to_string(),
            memory_usage,
            ready,
            restarts,
            started,
            state
        }
    }
}

fn convert_cpu_usage_to_milli(cpu: &str) -> i32{
    if cpu.is_empty()
    {
        return 0;
    }

    let value: String = cpu.chars().filter(|c| c.is_digit(10)).collect();
    let unit: String = cpu.chars().filter(|c| c.is_alphabetic()).collect();

    if value.is_empty() {
        return 0;
    }

    let parsed_value: f64 = value.parse().unwrap_or_else(|_| 0f64);

    let mut denominator= 1000000.0;

    if parsed_value < 1.0 || unit.is_empty() {
        return (parsed_value * 1000.0).ceil() as i32;
    }

    match unit.as_str() {
        "m" => {
            denominator = 1.0;
        }
        "u" => {
            denominator = 1000.0;
        }
        "n" => {}

        &_ => { error!("Unknown CPU unit") }
    }

    let result = (parsed_value/denominator).ceil() as i32;

    result
}

fn convert_memory_usage_to_bytes(memory: &str) -> i64{
    if memory.is_empty()
    {
        return 0;
    }

    let value: String = memory.chars().filter(|c| c.is_digit(10)).collect();
    let mut unit: String = memory.chars().filter(|c| c.is_alphabetic()).collect();
    unit = unit.to_lowercase();

    if value.is_empty() {
        return 0;
    }

    let parsed_value: i64 = value.parse().unwrap_or_else(|_| 0i64);
    let mut multiplier: i64= 1024;

    match unit.as_str() {
        "" => {
            multiplier = 1;
        }
        "ki" => {}
        "mi" => {
            multiplier = multiplier.pow(2);
        }
        "gi" => {
            multiplier = multiplier.pow(3);
        }
        "ti" => {
            multiplier = multiplier.pow(4);
        }
        "k" => {
            multiplier = 1000;
        }
        "m" => {
            multiplier = 1000000;
        }

        &_ => {}
    }

    return parsed_value * multiplier;
}