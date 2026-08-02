use std::{collections::BTreeMap, time::Duration};

use async_trait::async_trait;
use reqwest::{
    Client, StatusCode, Url,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};

use super::{ComputeKind, Deployment, DeploymentSpec, DeploymentStatus, Provider};
use crate::{Error, Result};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub struct RunpodClient {
    http: Client,
    api_base: Url,
}

impl RunpodClient {
    pub fn new(api_base: Url, api_key: &str) -> Result<Self> {
        let mut authorization = HeaderValue::from_str(&format!("Bearer {api_key}"))?;
        authorization.set_sensitive(true);

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, authorization);

        let http = Client::builder()
            .default_headers(headers)
            .timeout(REQUEST_TIMEOUT)
            .build()?;

        Ok(Self { http, api_base })
    }

    fn endpoint(&self, path: &str) -> Result<Url> {
        Ok(self.api_base.join(path.trim_start_matches('/'))?)
    }

    async fn parse_response<T: for<'de> Deserialize<'de>>(
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();
        if status.is_success() {
            return Ok(response.json().await?);
        }

        let body = response.text().await.unwrap_or_default();
        Err(Error::Api {
            status,
            message: extract_error_message(&body, status),
        })
    }
}

#[async_trait]
impl Provider for RunpodClient {
    async fn list(&self, compute: Option<ComputeKind>) -> Result<Vec<Deployment>> {
        let mut request = self.http.get(self.endpoint("pods")?);
        if let Some(compute) = compute {
            request = request.query(&[("computeType", compute.as_runpod_str())]);
        }

        let pods: Vec<RunpodPod> = Self::parse_response(request.send().await?).await?;
        Ok(pods.into_iter().map(Deployment::from).collect())
    }

    async fn get(&self, id: &str) -> Result<Deployment> {
        let pod: RunpodPod = Self::parse_response(
            self.http
                .get(self.endpoint(&format!("pods/{id}"))?)
                .send()
                .await?,
        )
        .await?;
        Ok(pod.into())
    }

    async fn create(&self, spec: DeploymentSpec) -> Result<Deployment> {
        let pod: RunpodPod = Self::parse_response(
            self.http
                .post(self.endpoint("pods")?)
                .json(&CreatePodRequest::from(spec))
                .send()
                .await?,
        )
        .await?;
        Ok(pod.into())
    }

    async fn stop(&self, id: &str) -> Result<Deployment> {
        let pod: RunpodPod = Self::parse_response(
            self.http
                .post(self.endpoint(&format!("pods/{id}/stop"))?)
                .send()
                .await?,
        )
        .await?;
        Ok(pod.into())
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreatePodRequest {
    pub name: String,
    pub image_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_registry_auth_id: Option<String>,
    pub compute_type: ComputeKind,
    pub container_disk_in_gb: u32,
    pub volume_in_gb: u32,
    pub volume_mount_path: String,
    pub ports: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub cloud_type: String,
    pub support_public_ip: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcpu_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_count: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gpu_type_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_type_priority: Option<String>,
}

impl From<DeploymentSpec> for CreatePodRequest {
    fn from(spec: DeploymentSpec) -> Self {
        Self {
            name: spec.name,
            image_name: spec.image,
            container_registry_auth_id: spec.container_registry_auth_id,
            compute_type: spec.compute,
            container_disk_in_gb: spec.container_disk_gb,
            volume_in_gb: spec.volume_gb,
            volume_mount_path: spec.volume_mount_path,
            ports: spec.ports,
            env: spec.env,
            cloud_type: spec.cloud_type,
            support_public_ip: true,
            vcpu_count: spec.vcpu_count,
            gpu_count: spec.gpu_count,
            gpu_type_priority: (!spec.gpu_type_ids.is_empty()).then(|| "custom".into()),
            gpu_type_ids: spec.gpu_type_ids,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunpodPod {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default, alias = "image")]
    image_name: String,
    #[serde(default)]
    desired_status: String,
    #[serde(default)]
    compute_type: Option<ComputeKind>,
    #[serde(default)]
    public_ip: Option<String>,
    #[serde(default, alias = "costPerHr")]
    adjusted_cost_per_hr: Option<f64>,
    #[serde(default)]
    port_mappings: BTreeMap<String, serde_json::Value>,
}

impl From<RunpodPod> for Deployment {
    fn from(pod: RunpodPod) -> Self {
        Self {
            id: pod.id,
            name: pod.name,
            image: pod.image_name,
            status: normalize_status(&pod.desired_status),
            compute: pod.compute_type,
            public_ip: pod.public_ip,
            cost_per_hour: pod.adjusted_cost_per_hr,
            port_mappings: pod
                .port_mappings
                .into_iter()
                .map(|(internal, external)| (internal, json_value_to_string(external)))
                .collect(),
        }
    }
}

fn normalize_status(status: &str) -> DeploymentStatus {
    match status.to_ascii_uppercase().as_str() {
        "RUNNING" => DeploymentStatus::Running,
        "EXITED" => DeploymentStatus::Exited,
        "TERMINATED" => DeploymentStatus::Terminated,
        "CREATED" | "PENDING" => DeploymentStatus::Pending,
        _ => DeploymentStatus::Unknown,
    }
}

fn json_value_to_string(value: serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value,
        other => other.to_string(),
    }
}

fn extract_error_message(body: &str, status: StatusCode) -> String {
    if let Ok(payload) = serde_json::from_str::<serde_json::Value>(body) {
        for key in ["message", "error", "detail"] {
            if let Some(message) = payload.get(key).and_then(|value| value.as_str())
                && !message.trim().is_empty()
            {
                return message.trim().to_owned();
            }
        }
    }

    let body = body.trim();
    if body.is_empty() {
        status
            .canonical_reason()
            .unwrap_or("unknown provider error")
            .to_owned()
    } else {
        body.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_cpu_create_payload_with_runpod_field_names() {
        let request = CreatePodRequest::from(DeploymentSpec {
            name: "fern-lane-a".into(),
            image: "ghcr.io/example/drone-sim:sha-123".into(),
            container_registry_auth_id: Some("registry-auth-123".into()),
            compute: ComputeKind::Cpu,
            container_disk_gb: 40,
            volume_gb: 20,
            volume_mount_path: "/workspace".into(),
            ports: vec!["22/tcp".into(), "8080/http".into()],
            env: BTreeMap::from([("FERN_PROFILE".into(), "lane-a".into())]),
            cloud_type: "COMMUNITY".into(),
            vcpu_count: Some(8),
            gpu_count: None,
            gpu_type_ids: vec![],
        });

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(value["imageName"], "ghcr.io/example/drone-sim:sha-123");
        assert_eq!(value["containerRegistryAuthId"], "registry-auth-123");
        assert_eq!(value["computeType"], "CPU");
        assert_eq!(value["containerDiskInGb"], 40);
        assert_eq!(value["vcpuCount"], 8);
        assert!(value.get("gpuCount").is_none());
        assert!(value.get("gpuTypeIds").is_none());
        assert!(value.get("gpuTypePriority").is_none());
    }

    #[test]
    fn serializes_ordered_gpu_fallback() {
        let request = CreatePodRequest::from(DeploymentSpec {
            name: "fern-stack".into(),
            image: "ghcr.io/example/drone-sim@sha256:123".into(),
            container_registry_auth_id: None,
            compute: ComputeKind::Gpu,
            container_disk_gb: 40,
            volume_gb: 20,
            volume_mount_path: "/workspace".into(),
            ports: vec![],
            env: BTreeMap::new(),
            cloud_type: "SECURE".into(),
            vcpu_count: None,
            gpu_count: Some(1),
            gpu_type_ids: vec![
                "NVIDIA RTX 2000 Ada Generation".into(),
                "NVIDIA GeForce RTX 4090".into(),
            ],
        });

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(value["gpuTypePriority"], "custom");
        assert_eq!(
            value["gpuTypeIds"],
            serde_json::json!(["NVIDIA RTX 2000 Ada Generation", "NVIDIA GeForce RTX 4090"])
        );
    }

    #[test]
    fn parses_current_and_legacy_response_fields() {
        let pod: RunpodPod = serde_json::from_value(serde_json::json!({
            "id": "pod-123",
            "name": "fern-lane-a",
            "image": "drone-sim:test",
            "desiredStatus": "RUNNING",
            "computeType": "CPU",
            "publicIp": "192.0.2.10",
            "costPerHr": 0.12,
            "portMappings": {"22": 32001}
        }))
        .unwrap();

        let deployment = Deployment::from(pod);
        assert_eq!(deployment.status, DeploymentStatus::Running);
        assert_eq!(deployment.image, "drone-sim:test");
        assert_eq!(deployment.compute, Some(ComputeKind::Cpu));
        assert_eq!(deployment.port_mappings["22"], "32001");
    }

    #[test]
    fn extracts_structured_provider_errors() {
        assert_eq!(
            extract_error_message(
                r#"{"message":"There are no instances currently available"}"#,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            "There are no instances currently available"
        );
    }
}
