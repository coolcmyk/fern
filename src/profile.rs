use std::collections::BTreeMap;

use crate::provider::{ComputeKind, DeploymentSpec};

pub const DRONE_SIM_UPSTREAM_SHA: &str = "edc89943ca5a498d05d81b4dd65b2fa3e47073e2";
pub const DRONE_SIM_IMAGE: &str =
    "ghcr.io/coolcmyk/fern-drone-sim-lane-a:edc89943ca5a498d05d81b4dd65b2fa3e47073e2-fern.1";

pub fn drone_sim_lane_a(image: Option<String>, duration_seconds: u32) -> DeploymentSpec {
    DeploymentSpec {
        name: "fern-drone-sim-lane-a".into(),
        image: image.unwrap_or_else(|| DRONE_SIM_IMAGE.into()),
        compute: ComputeKind::Cpu,
        container_disk_gb: 40,
        volume_gb: 20,
        volume_mount_path: "/workspace".into(),
        ports: vec![],
        env: BTreeMap::from([
            ("DURATION".into(), duration_seconds.to_string()),
            ("FASTDDS_BUILTIN_TRANSPORTS".into(), "UDPv4".into()),
            ("FERN_PROFILE".into(), "drone-sim-lane-a".into()),
            ("OUTDIR".into(), "/workspace/fern/drone-sim".into()),
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
    fn lane_a_is_cpu_only_and_uses_udp_transport() {
        let spec = drone_sim_lane_a(None, 60);

        assert_eq!(spec.compute, ComputeKind::Cpu);
        assert_eq!(spec.vcpu_count, Some(8));
        assert_eq!(spec.gpu_count, None);
        assert_eq!(spec.env["FASTDDS_BUILTIN_TRANSPORTS"], "UDPv4");
        assert_eq!(spec.env["DURATION"], "60");
        assert!(spec.ports.is_empty());
        assert_eq!(spec.image, DRONE_SIM_IMAGE);
    }

    #[test]
    fn lane_a_accepts_an_image_override() {
        let spec = drone_sim_lane_a(Some("registry.example/drone-sim:test".into()), 300);
        assert_eq!(spec.image, "registry.example/drone-sim:test");
    }
}
