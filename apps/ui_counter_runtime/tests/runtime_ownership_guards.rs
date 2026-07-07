use std::fs;
use std::path::PathBuf;

fn crate_file(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"))
}

#[test]
fn counter_product_does_not_build_prepared_frames_or_render_reports() {
    let source = crate_file("src/lib.rs");

    for forbidden in [
        "UiRuntimePreparedFrameRecord::new",
        "UiRenderPrimitiveReport::from_runtime_view_report",
        "UiRenderPrimitiveReport::from_runtime_view_report_with_host_data",
        "UiRuntimeView::from_artifact_report",
        "index.saturating_mul(3)",
    ] {
        assert!(
            !source.contains(forbidden),
            "apps/ui_counter_runtime must not own generic runtime/render plumbing: found {forbidden}"
        );
    }
}

#[test]
fn counter_product_does_not_build_ui_packets_or_action_events() {
    let source = crate_file("src/lib.rs");

    for forbidden in ["UiEventPacket::new", "UiActionEvent::new"] {
        assert!(
            !source.contains(forbidden),
            "apps/ui_counter_runtime must not manually build UI packets/events: found {forbidden}"
        );
    }
}

#[test]
fn counter_text_helpers_are_real_labels_not_fake_buttons() {
    let source = crate_file("src/lib.rs");
    let label_start = source
        .find("fn label_control")
        .expect("label_control helper should exist");
    let action_start = source
        .find("fn counter_action_control")
        .expect("counter_action_control helper should exist");
    let label_helpers = &source[label_start..action_start];

    assert!(
        label_helpers.contains("LABEL_CONTROL_KIND_ID"),
        "label helpers must render labels as label controls"
    );
    assert!(
        !label_helpers.contains("BUTTON_CONTROL_KIND_ID"),
        "label helpers must not fake text as button controls"
    );
}

#[test]
fn counter_product_uses_engine_owned_runtime_helpers() {
    let source = crate_file("src/lib.rs");

    for required in [
        "evaluate_and_prepare_mounted_ui_screen",
        "UiRuntimeHitTargetResource",
        "UiPointerActivationResource",
        "dispatch_ui_runtime_action_request",
    ] {
        assert!(
            source.contains(required),
            "counter app should consume engine-owned UI runtime helper {required}"
        );
    }
}
