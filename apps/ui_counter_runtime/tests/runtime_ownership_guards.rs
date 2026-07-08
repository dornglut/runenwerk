use std::fs;
use std::path::PathBuf;

fn crate_file(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).expect("source file should be readable")
}

#[test]
fn counter_product_keeps_generic_render_preparation_out_of_the_app() {
    let source = crate_file("src/lib.rs");
    assert!(!source.contains("UiRuntimePreparedFrameRecord::new"));
    assert!(!source.contains("UiRenderPrimitiveReport::from_runtime_view_report"));
    assert!(!source.contains("UiRenderPrimitiveReport::from_runtime_view_report_with_host_data"));
    assert!(!source.contains("UiRuntimeView::from_artifact_report"));
    assert!(!source.contains("index.saturating_mul(3)"));
}

#[test]
fn counter_product_keeps_packet_and_event_construction_out_of_the_app() {
    let source = crate_file("src/lib.rs");
    assert!(!source.contains("UiEventPacket::new"));
    assert!(!source.contains("UiActionEvent::new"));
}

#[test]
fn counter_text_helpers_are_real_labels_not_buttons() {
    let source = crate_file("src/lib.rs");
    let label_start = source
        .find("fn label_control")
        .expect("label helper should exist");
    let action_start = source
        .find("fn counter_action_control")
        .expect("action helper should exist");
    let label_helpers = &source[label_start..action_start];
    assert!(label_helpers.contains("LABEL_CONTROL_KIND_ID"));
    assert!(!label_helpers.contains("BUTTON_CONTROL_KIND_ID"));
}

#[test]
fn counter_product_consumes_engine_owned_runtime_helpers() {
    let source = crate_file("src/lib.rs");
    assert!(source.contains("evaluate_and_prepare_mounted_ui_screen"));
    assert!(source.contains("UiRuntimeHitTargetResource"));
    assert!(source.contains("UiPointerActivationResource"));
    assert!(source.contains("dispatch_ui_runtime_action_request"));
}
