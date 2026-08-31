#[path = "gpu_direct_wgpu_cost_native/common.rs"]
mod common;
#[path = "gpu_direct_wgpu_cost_native/offscreen_draw.rs"]
mod offscreen_draw;
#[path = "gpu_direct_wgpu_cost_native/prefix_scan.rs"]
mod prefix_scan;
// The retained workload module also contains the surface-present proof path, which this
// offscreen-only comparison intentionally does not execute.
#[allow(dead_code)]
#[path = "gpu_direct_wgpu_cost_native/reaction_diffusion.rs"]
mod reaction_diffusion;

fn retain_measurement_profile(evidence: &mut serde_json::Value) {
    assert!(
        !cfg!(debug_assertions),
        "G6-P01 performance evidence must run from an optimized Cargo test profile"
    );
    let profile = std::env::var("RUNEN_GPU_PROOF_BUILD_PROFILE")
        .expect("G6-P01 conformance must declare its Cargo build profile");
    assert_eq!(
        profile, "release",
        "G6-P01 retained evidence currently requires the Cargo release profile"
    );
    evidence["measurement_environment"] = serde_json::json!({
        "cargo_test_profile": profile,
        "debug_assertions": cfg!(debug_assertions),
        "purpose": "production-representative host-cost characterization; correctness remains profile-independent",
    });
}

#[test]
#[ignore = "requires a real Vulkan fallback adapter; executed by focused G6-P01 CI while the comparison portfolio is built"]
fn known_pattern_direct_wgpu_boundary_cost_is_measurable_and_correct() {
    let mut evidence = offscreen_draw::compare();
    retain_measurement_profile(&mut evidence);
    assert_eq!(evidence["workload"], "G6-C01-known-pattern-offscreen-draw");
    assert!(
        evidence["runengpu"]["count"].as_u64().unwrap() > 0,
        "RunenGPU measured samples must be retained"
    );
    assert!(
        evidence["direct_wgpu"]["count"].as_u64().unwrap() > 0,
        "direct-WGPU measured samples must be retained"
    );
    println!("{}", serde_json::to_string_pretty(&evidence).unwrap());
}

#[test]
#[ignore = "requires a real Vulkan fallback adapter; executed by focused G6-P01 CI while the comparison portfolio is built"]
fn prefix_scan_direct_wgpu_boundary_cost_is_measurable_and_correct() {
    let mut evidence = prefix_scan::compare();
    retain_measurement_profile(&mut evidence);
    assert_eq!(evidence["workload"], "G5-C01-4097-u32-prefix-scan");
    assert_eq!(
        evidence["runengpu"]["count"].as_u64().unwrap(),
        u64::try_from(common::MEASURED_SAMPLES).unwrap()
    );
    assert_eq!(
        evidence["direct_wgpu"]["count"].as_u64().unwrap(),
        u64::try_from(common::MEASURED_SAMPLES).unwrap()
    );
    println!("{}", serde_json::to_string_pretty(&evidence).unwrap());
}

#[test]
#[ignore = "requires a real Vulkan fallback adapter; executed by focused G6-P01 CI while the comparison portfolio is built"]
fn reaction_diffusion_direct_wgpu_boundary_cost_is_measurable_and_correct() {
    let mut evidence = reaction_diffusion::compare();
    retain_measurement_profile(&mut evidence);
    assert_eq!(evidence["workload"], "G6-I01-reaction-diffusion");
    assert_eq!(
        evidence["runengpu"]["count"].as_u64().unwrap(),
        u64::try_from(common::MEASURED_SAMPLES).unwrap()
    );
    assert_eq!(
        evidence["direct_wgpu"]["count"].as_u64().unwrap(),
        u64::try_from(common::MEASURED_SAMPLES).unwrap()
    );
    println!("{}", serde_json::to_string_pretty(&evidence).unwrap());
}
