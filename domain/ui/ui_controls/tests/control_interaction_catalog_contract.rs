use ui_controls::{
    BUTTON_CONTROL_KIND_ID, BaseControlsPlugin, ControlCatalogEntryDescriptor,
    ControlInspectionSection, ControlKindId, runenwerk_control_package,
};

#[test]
fn control_interaction_summary_attaches_to_catalog_entry_read_only() {
    let package = runenwerk_control_package();
    let button_id = ControlKindId::new(BUTTON_CONTROL_KIND_ID);
    let kind = package
        .control_kind(&button_id)
        .expect("button control kind should exist");
    let compiled = BaseControlsPlugin::new().compile();
    let control = compiled
        .controls
        .iter()
        .find(|control| control.module.kind.control_kind_id == button_id)
        .expect("button should be compiled");

    let entry = ControlCatalogEntryDescriptor::from_control_kind(&package, kind)
        .with_interaction_summary(&control.interaction.summary());

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
    assert!(!entry.interaction_executes_host_commands);
    assert!(!entry.interaction_mutates_product_state);
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
