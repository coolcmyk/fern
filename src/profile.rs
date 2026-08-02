use std::collections::BTreeMap;

use crate::provider::{ComputeKind, DeploymentSpec};

pub const DRONE_SIM_STACK_IMAGE: &str = "ghcr.io/teapotlaboratories/drone-sim:stack-main";
pub const DRONE_SIM_GPU_TYPE_ID: &str = "NVIDIA RTX 2000 Ada Generation";
pub const DRONE_SIM_GPU_FALLBACK_IDS: &[&str] = &[
    "NVIDIA RTX 4000 Ada Generation",
    "NVIDIA GeForce RTX 4090",
    "NVIDIA L4",
    "NVIDIA RTX 5000 Ada Generation",
    "NVIDIA RTX 6000 Ada Generation",
    "NVIDIA L40S",
];

pub fn drone_sim_stack(
    image: Option<String>,
    duration_seconds: u32,
    container_registry_auth_id: Option<String>,
    gpu_type_id: Option<String>,
) -> DeploymentSpec {
    let gpu_type_ids = match gpu_type_id {
        Some(gpu_type_id) => vec![gpu_type_id],
        None => std::iter::once(DRONE_SIM_GPU_TYPE_ID.to_owned())
            .chain(
                DRONE_SIM_GPU_FALLBACK_IDS
                    .iter()
                    .map(|gpu_type_id| (*gpu_type_id).to_owned()),
            )
            .collect(),
    };

    DeploymentSpec {
        name: "fern-drone-sim-stack".into(),
        image: image.unwrap_or_else(|| DRONE_SIM_STACK_IMAGE.into()),
        container_registry_auth_id,
        compute: ComputeKind::Gpu,
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
        vcpu_count: None,
        gpu_count: Some(1),
        gpu_type_ids,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_uses_one_gpu_and_udp_transport() {
        let spec = drone_sim_stack(None, 60, Some("auth-id".into()), None);

        assert_eq!(spec.compute, ComputeKind::Gpu);
        assert_eq!(spec.vcpu_count, None);
        assert_eq!(spec.gpu_count, Some(1));
        assert_eq!(spec.gpu_type_ids[0], DRONE_SIM_GPU_TYPE_ID);
        assert_eq!(&spec.gpu_type_ids[1..], DRONE_SIM_GPU_FALLBACK_IDS);
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
        let spec = drone_sim_stack(
            Some("registry.example/drone-sim:test".into()),
            300,
            None,
            Some("NVIDIA GeForce RTX 4090".into()),
        );
        assert_eq!(spec.image, "registry.example/drone-sim:test");
        assert_eq!(spec.gpu_type_ids, ["NVIDIA GeForce RTX 4090"]);
        assert!(spec.container_registry_auth_id.is_none());
    }
}
