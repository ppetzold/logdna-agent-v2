use serde::{Serialize, Deserialize};

use super::{pod_stats::PodStats, container_stats::ContainerStats, controller_stats::ControllerStats};

#[derive(Serialize, Deserialize)]
pub struct ExtendedPodStats{
    #[serde(flatten)]
    pub pod_stats: PodStats, 

    #[serde(flatten)]
    pub container_stats: ContainerStats,

    #[serde(flatten)]
    pub controller_stats: ControllerStats
}

impl ExtendedPodStats {
    pub fn new(p_s: PodStats, c_s: ContainerStats) -> Self {
        Self { 
            pod_stats: p_s, 
            container_stats: c_s, 
            controller_stats: ControllerStats::new()
        }
    }
}