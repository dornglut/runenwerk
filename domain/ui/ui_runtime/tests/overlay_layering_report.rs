use ui_runtime::{
    base_controls_overlay_layering_fixture, base_controls_overlay_layering_negative_scripts,
    base_controls_overlay_layering_positive_script, replay_overlay_layering,
};

#[test]
fn overlay_layering_report_proves_open_stack_dismissal_and_no_bypass() {
    let fixture = base_controls_overlay_layering_fixture();
    let script = base_controls_overlay_layering_positive_script();
    let report = replay_overlay_layering(&fixture, &script);

    assert!(report.declarations.len() >= 8);
    assert!(report.open_intents.iter().any(|row| row.overlay_kind == "popup"));
    assert!(report.open_intents.iter().any(|row| row.overlay_kind == "menu"));
    assert!(report.open_intents.iter().any(|row| row.overlay_kind == "dropdown"));
    assert!(report.open_intents.iter().any(|row| row.overlay_kind == "tooltip"));
    assert!(report.open_intents.iter().any(|row| row.overlay_kind == "picker-popup"));
    assert!(report.open_intents.iter().any(|row| row.overlay_kind == "focus-containing-overlay"));
    assert!(report.stack_entries.iter().any(|row| row.parent_request_id.is_some()));
    assert!(report.dismissal_evidence.iter().any(|row| row.reason == "escape.topmost_dismissed"));
    assert!(report.dismissal_evidence.iter().any(|row| row.reason == "outside-pointer.topmost_dismissed"));
    assert!(report.pointer_capture_evidence.iter().any(|row| row.outside_dismissal_blocked));
    assert!(!report.keyboard_navigation_evidence.is_empty());
    assert!(report.boundary_assertions.placement_recomputed_after_scroll > 0);
    assert!(report.boundary_assertions.placement_recomputed_after_viewport_resize > 0);
    assert!(report.boundary_assertions.anchor_invalidation_suppressed > 0);
    assert!(report.boundary_assertions.no_bypass_evidence());
}

#[test]
fn overlay_layering_negative_report_proves_disabled_suppression() {
    let fixture = base_controls_overlay_layering_fixture();
    let scripts = base_controls_overlay_layering_negative_scripts();
    let report = replay_overlay_layering(&fixture, &scripts[0]);

    assert!(report.open_intents.is_empty());
    assert!(report.suppression_evidence.iter().any(|row| row.reason == "anchor.disabled"));
    assert_eq!(report.boundary_assertions.host_commands_executed, 0);
    assert_eq!(report.boundary_assertions.product_mutations, 0);
    assert_eq!(report.boundary_assertions.text_edit_transactions, 0);
}
