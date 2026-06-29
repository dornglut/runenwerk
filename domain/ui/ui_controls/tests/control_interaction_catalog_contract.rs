use ui_controls::{
    BASE_CONTROL_TARGET_KIND_IDS, BUTTON_CONTROL_KIND_ID, BaseControlsPlugin,
    ControlInspectionSection,
};

#[test]
fn compiled_base_control_catalog_exposes_interaction_facts_read_only() {
    let compiled = BaseControlsPlugin::new().compile();

    for control_kind_id in BASE_CONTROL_TARGET_KIND_IDS {
        let entry = compiled
            .catalog
            .entry(control_kind_id)
            .expect("base control should be catalog-visible");

        assert!(!entry.interaction_states.is_empty());
        assert!(!entry.interaction_triggers.is_empty());
        assert!(!entry.interaction_has_runtime_behavior);
        assert!(!entry.interaction_executes_host_commands);
        assert!(!entry.interaction_mutates_product_state);
    }

    let entry = compiled
        .catalog
        .entry(BUTTON_CONTROL_KIND_ID)
        .expect("button should be catalog-visible");
    assert!(
        entry
            .interaction_triggers
            .contains(&"pointer-press".to_owned())
    );
    assert!(
        entry
            .interaction_outcomes
            .contains(&"activation-requested".to_owned())
    );
}

#[test]
fn compiled_base_control_inspection_exposes_interaction_facts_read_only() {
    let compiled = BaseControlsPlugin::new().compile();
    let inspection = compiled
        .inspection
        .descriptor(BUTTON_CONTROL_KIND_ID)
        .expect("button inspection should exist");

    assert_eq!(
        inspection.fact(
            ControlInspectionSection::Interaction,
            "executes_host_commands"
        ),
        Some("false")
    );
    assert_eq!(
        inspection.fact(
            ControlInspectionSection::Interaction,
            "mutates_product_state"
        ),
        Some("false")
    );
    assert_eq!(
        inspection.fact(
            ControlInspectionSection::Interaction,
            "has_runtime_behavior"
        ),
        Some("false")
    );
    assert!(
        inspection
            .fact(ControlInspectionSection::Interaction, "outcomes")
            .is_some_and(|value| value.contains("activation-requested"))
    );
}
