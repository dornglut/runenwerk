#[path = "gpu_direct_wgpu_cost_native/common.rs"]
mod common;
#[path = "gpu_direct_wgpu_cost_native/offscreen_draw.rs"]
mod offscreen_draw;
#[path = "gpu_direct_wgpu_cost_native/prefix_scan.rs"]
mod prefix_scan;

#[test]
#[ignore = "requires a real Vulkan fallback adapter; executed by focused G6-P01 CI while the comparison portfolio is built"]
fn known_pattern_direct_wgpu_boundary_cost_is_measurable_and_correct() {
    let evidence = offscreen_draw::compare();
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
    let evidence = prefix_scan::compare();
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
