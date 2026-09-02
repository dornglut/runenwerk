use super::{common, offscreen_draw, prefix_scan, reaction_diffusion, retain_measurement_profile};
use serde_json::{Value, json};
use std::path::PathBuf;

const REPORT_SCHEMA_VERSION: u32 = 1;
const REQUIRED_WORKLOADS: usize = 3;

fn has_correctness_outcome(value: &Value) -> bool {
    value.as_str().is_some_and(|text| !text.is_empty())
        || value.as_object().is_some_and(|outcomes| {
            !outcomes.is_empty()
                && outcomes
                    .values()
                    .all(|outcome| outcome.as_str().is_some_and(|text| !text.is_empty()))
        })
}

fn assert_workload_evidence(evidence: &Value, expected_workload: &str) {
    assert_eq!(evidence["workload"], expected_workload);
    assert_eq!(
        evidence["runengpu"]["count"].as_u64().unwrap(),
        u64::try_from(common::MEASURED_SAMPLES).unwrap()
    );
    assert_eq!(
        evidence["direct_wgpu"]["count"].as_u64().unwrap(),
        u64::try_from(common::MEASURED_SAMPLES).unwrap()
    );
    assert_eq!(evidence["timestamp_evidence"]["status"], "measured");
    assert_eq!(
        evidence["measurement_environment"]["cargo_test_profile"],
        "release"
    );
    assert_eq!(
        evidence["measurement_environment"]["debug_assertions"],
        false
    );
    assert!(
        has_correctness_outcome(&evidence["correctness"]),
        "retained workload evidence must state a non-empty correctness outcome"
    );
    assert!(evidence["comparison_envelope"].is_object());
    assert!(evidence["adapter_equivalence"].is_object());
    assert!(evidence["cold"].is_object());
    assert!(evidence["warm_lifecycle"].is_object());
    assert!(evidence["runengpu_over_direct_ratio"].is_object());
}

fn workload_evidence() -> Vec<Value> {
    let mut known_pattern = offscreen_draw::compare();
    retain_measurement_profile(&mut known_pattern);
    assert_workload_evidence(&known_pattern, "G6-C01-known-pattern-offscreen-draw");

    let mut prefix_scan_evidence = prefix_scan::compare();
    prefix_scan_evidence["timestamp_evidence"] =
        prefix_scan::timestamp::evidence(&prefix_scan_evidence);
    retain_measurement_profile(&mut prefix_scan_evidence);
    assert_workload_evidence(&prefix_scan_evidence, "G5-C01-4097-u32-prefix-scan");

    let mut reaction_diffusion_evidence = reaction_diffusion::compare();
    reaction_diffusion_evidence["timestamp_evidence"] =
        reaction_diffusion::timestamp::evidence(&reaction_diffusion_evidence);
    reaction_diffusion_evidence["host_characterization"] =
        reaction_diffusion::host_characterization::evidence();
    retain_measurement_profile(&mut reaction_diffusion_evidence);
    assert_workload_evidence(&reaction_diffusion_evidence, "G6-I01-reaction-diffusion");
    assert!(reaction_diffusion_evidence["host_characterization"].is_object());

    vec![
        known_pattern,
        prefix_scan_evidence,
        reaction_diffusion_evidence,
    ]
}

fn artifact_path() -> PathBuf {
    std::env::var_os("RUNEN_GPU_PROOF_ARTIFACT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/runengpu-proof-artifacts"))
        .join("direct-wgpu-cost")
        .join("report.json")
}

#[test]
#[ignore = "retained optimized G6-P01 characterization; executed by RunenGPU Conformance CI"]
fn direct_wgpu_cost_portfolio_retains_report() {
    let revision = std::env::var("RUNEN_GPU_PROOF_REVISION")
        .expect("retained G6-P01 conformance must declare the exact repository revision");
    assert!(!revision.trim().is_empty());

    let workloads = workload_evidence();
    assert_eq!(workloads.len(), REQUIRED_WORKLOADS);

    let report = json!({
        "schema_version": REPORT_SCHEMA_VERSION,
        "requirement": "G6-P01",
        "subject": "RunenGPU direct-WGPU boundary-cost characterization",
        "repository_revision": revision,
        "runner": {
            "os": std::env::var("RUNNER_OS").ok(),
            "arch": std::env::var("RUNNER_ARCH").ok(),
        },
        "measurement_policy": {
            "cargo_test_profile": "release",
            "warmup_samples_per_path": common::WARMUP_SAMPLES,
            "measured_samples_per_path": common::MEASURED_SAMPLES,
            "hosted_ci_timing_is_characterization_only": true,
            "performance_pass_fail_threshold": null,
            "regression_disposition_location": "issue/PR closeout after review of this retained report",
        },
        "comparison_scope": {
            "workload_count": REQUIRED_WORKLOADS,
            "workloads": [
                "G5-C01 fixed 4,097-element inclusive/exclusive u32 prefix scan",
                "G6-C01 deterministic known-pattern indexed offscreen draw",
                "G6-I01 representative retained reaction-diffusion envelopes",
            ],
            "direct_wgpu_role": "test-owned comparison baseline only",
        },
        "unavailable_metrics": [
            {
                "metric": "backend allocation/high-water bytes",
                "status": "unavailable",
                "reason": "the accepted public/test-visible boundaries expose no allocator high-water metric and G6-P01 does not add production allocator instrumentation",
            },
            {
                "metric": "RunenGPU timestamp period in nanoseconds",
                "status": "unavailable",
                "reason": "public RunenGPU device facts do not expose the timestamp period; symmetric raw timestamp ticks are retained without fabricating a conversion",
            },
        ],
        "workloads": workloads,
    });

    let path = artifact_path();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    assert!(path.metadata().unwrap().len() > 0);
}
