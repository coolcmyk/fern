use std::collections::BTreeMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::Result;

pub mod runpod;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ComputeKind {
    Cpu,
    Gpu,
}

impl ComputeKind {
    pub fn as_runpod_str(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Gpu => "GPU",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    Pending,
    Running,
    Exited,
    Terminated,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: DeploymentStatus,
    pub compute: Option<ComputeKind>,
    pub public_ip: Option<String>,
    pub cost_per_hour: Option<f64>,
    pub port_mappings: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DeploymentSpec {
    pub name: String,
    pub image: String,
    pub compute: ComputeKind,
    pub container_disk_gb: u32,
    pub volume_gb: u32,
    pub volume_mount_path: String,
    pub ports: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub cloud_type: String,
    pub vcpu_count: Option<u32>,
    pub gpu_count: Option<u32>,
    pub gpu_type_ids: Vec<String>,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn list(&self, compute: Option<ComputeKind>) -> Result<Vec<Deployment>>;
    async fn get(&self, id: &str) -> Result<Deployment>;
    async fn create(&self, spec: DeploymentSpec) -> Result<Deployment>;
    async fn stop(&self, id: &str) -> Result<Deployment>;
}
