use std::collections::BTreeMap;

use crate::provider::{ComputeKind, DeploymentSpec};

pub const DRONE_SIM_STACK_IMAGE: &str = "ghcr.io/teapotlaboratories/drone-sim:stack-main";

pub fn drone_sim_stack(
    image: Option<String>,
    duration_seconds: u32,
    container_registry_auth_id: Option<String>,
) -> DeploymentSpec {
    DeploymentSpec {
        name: "fern-drone-sim-stack".into(),
        image: image.unwrap_or_else(|| DRONE_SIM_STACK_IMAGE.into()),
        container_registry_auth_id,
        compute: ComputeKind::Cpu,
        container_disk_gb: 40,
        volume_gb: 20,
        volume_mount_path: "/workspace".into(),
        ports: vec![],
        env: BTreeMap::from([
            ("DURATION".into(), duration_seconds.to_string()),
            ("FASTDDS_BUILTIN_TRANSPORTS".into(), "UDPv4".into()),
            ("FERN_PROFILE".into(), "drone-sim-stack".into()),
            ("FERN_WORKSPACE".into(), "/workspace".into()),
        ]),
        cloud_type: "SECURE".into(),
        vcpu_count: Some(8),
        gpu_count: None,
        gpu_type_ids: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_is_cpu_only_and_uses_udp_transport() {
        let spec = drone_sim_stack(None, 60, Some("auth-id".into()));

        assert_eq!(spec.compute, ComputeKind::Cpu);
        assert_eq!(spec.vcpu_count, Some(8));
        assert_eq!(spec.gpu_count, None);
        assert_eq!(spec.env["FASTDDS_BUILTIN_TRANSPORTS"], "UDPv4");
        assert_eq!(spec.env["DURATION"], "60");
        assert_eq!(spec.env["FERN_PROFILE"], "drone-sim-stack");
        assert_eq!(spec.env["FERN_WORKSPACE"], "/workspace");
        assert!(!spec.env.contains_key("OUTDIR"));
        assert!(spec.ports.is_empty());
        assert_eq!(spec.image, DRONE_SIM_STACK_IMAGE);
        assert_eq!(spec.container_registry_auth_id.as_deref(), Some("auth-id"));
    }

    #[test]
    fn stack_accepts_an_image_override() {
        let spec = drone_sim_stack(Some("registry.example/drone-sim:test".into()), 300, None);
        assert_eq!(spec.image, "registry.example/drone-sim:test");
        assert!(spec.container_registry_auth_id.is_none());
    }
}
