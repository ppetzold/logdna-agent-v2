use chrono::Local;
use k8s_openapi::api::core::v1::{Container, ContainerState, ContainerStatus};
use serde::{Deserialize, Serialize};

use super::helpers::{convert_cpu_usage_to_milli, convert_memory_usage_to_bytes};

#[derive(Serialize, Deserialize)]
pub struct ContainerStats {
    pub container_age: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub container: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "::serde_with::rust::unwrap_or_skip")]
    pub cpu_limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "::serde_with::rust::unwrap_or_skip")]
    pub cpu_request: Option<i32>,
    pub cpu_usage: i32,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub image_tag: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "::serde_with::rust::unwrap_or_skip")]
    pub last_finished: Option<i64>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub last_reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "::serde_with::rust::unwrap_or_skip")]
    pub last_started: Option<i64>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub last_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "::serde_with::rust::unwrap_or_skip")]
    pub memory_limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "::serde_with::rust::unwrap_or_skip")]
    pub memory_request: Option<i64>,
    pub memory_usage: i64,
    pub ready: bool,
    pub restarts: i32,
    pub started: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub state: String,
}

impl ContainerStats {
    pub fn builder<'a>(
        c: &'a Container,
        c_status: &'a ContainerStatus,
        c_state: &'a ContainerState,
        raw_cpu_usage: &'a str,
        raw_memory_usage: &'a str,
    ) -> ContainerStatsBuilder<'a> {
        ContainerStatsBuilder {
            c,
            c_status,
            c_state,
            raw_cpu_usage,
            raw_memory_usage,
        }
    }
}

pub struct ContainerStatsBuilder<'a> {
    c: &'a Container,
    c_status: &'a ContainerStatus,
    c_state: &'a ContainerState,
    raw_cpu_usage: &'a str,
    raw_memory_usage: &'a str,
}

impl ContainerStatsBuilder<'_> {
    pub fn new<'a>(
        c: &'a Container,
        c_status: &'a ContainerStatus,
        c_state: &'a ContainerState,
        raw_cpu_usage: &'a str,
        raw_memory_usage: &'a str,
    ) -> ContainerStatsBuilder<'a> {
        ContainerStatsBuilder {
            c,
            c_status,
            c_state,
            raw_cpu_usage,
            raw_memory_usage,
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

        let mut cpu_limit = None;
        let mut cpu_request = None;
        let mut memory_limit = None;
        let mut memory_request = None;
        let mut last_started = None;
        let mut last_finished = None;

        let mut container_age: i64 = 0;
        let mut started: i64 = 0;

        let restarts = self.c_status.restart_count;
        let ready = self.c_status.ready;

        if self.c.image.is_some() {
            let container_image = self.c.image.clone().unwrap();

            let split = container_image.split_once(":");

            if split.is_some() {
                image = split.unwrap().0.to_string();
                image_tag = split.unwrap().1.to_string();
            }
        }

        let running = self.c_state.running.as_ref();
        let terminated = self.c_state.terminated.as_ref();

        if running.is_some() {
            state = "Running".to_string();

            let started_at = running.unwrap().started_at.as_ref().and_then(|s| Some(s.0));

            if started_at.is_some() {
                container_age = Local::now()
                    .signed_duration_since(started_at.unwrap())
                    .num_milliseconds();

                started = started_at.unwrap().timestamp_millis();
            }
        } else if terminated.is_some() {
            state = "Terminated".to_string()
        } else {
            state = "Waiting".to_string()
        }

        let last_status_state = self.c_status.last_state.as_ref();

        if last_status_state.is_some() {
            let last_running = last_status_state.unwrap().running.as_ref();
            let last_terminated = last_status_state.unwrap().terminated.as_ref();
            let last_waiting = last_status_state.unwrap().waiting.as_ref();

            if last_waiting.is_some() {
                last_state = String::from("Waiting");
            }

            last_running.and_then(|l| {
                last_state = "Running".to_string();
                l.started_at.as_ref().and_then(|s| {
                    Some({
                        last_started = Some(s.0.timestamp_millis());
                    })
                })
            });

            last_terminated.and_then(|l| {
                last_state = "Terminated".to_string();
                l.started_at
                    .as_ref()
                    .and_then(|s| Some(last_started = Some(s.0.timestamp_millis())));
                l.finished_at
                    .as_ref()
                    .and_then(|f| Some(last_finished = Some(f.0.timestamp_millis())));
                l.reason
                    .as_ref()
                    .and_then(|r| Some(last_reason = r.to_string()))
            });
        }

        if last_state.eq(&state) || last_state.eq("") {
            last_state = String::from("");
            last_reason = String::from("");
            last_finished = None;
            last_started = None;
        }

        let resources = self.c.resources.as_ref();

        if resources.is_some() {
            let limits = resources.unwrap().limits.as_ref();

            if limits.is_some() {
                let cpu = limits.unwrap().get("cpu");
                let memory = limits.unwrap().get("memory");

                cpu_limit = cpu
                    .as_deref()
                    .map(|cpu| Some(convert_cpu_usage_to_milli(cpu.0.as_str())))
                    .unwrap_or(None);

                memory_limit = memory
                    .as_deref()
                    .map(|memory| Some(convert_memory_usage_to_bytes(memory.0.as_str())))
                    .unwrap_or(None);
            }

            let requests = resources.unwrap().requests.as_ref();

            if requests.is_some() {
                let cpu = requests.unwrap().get("cpu");
                let memory = requests.unwrap().get("memory");

                cpu_request = cpu
                    .as_deref()
                    .map(|cpu| Some(convert_cpu_usage_to_milli(cpu.0.as_str())))
                    .unwrap_or(None);

                memory_request = memory
                    .as_deref()
                    .map(|memory| Some(convert_memory_usage_to_bytes(memory.0.as_str())))
                    .unwrap_or(None);
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
            state,
        }
    }
}
