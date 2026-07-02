use ui_controls::{
    runenwerk_control_package, BaseControlsPlugin, ACTION_PROMPT_CONTROL_KIND_ID,
    BUTTON_CONTROL_KIND_ID, COLOR_PICKER_CONTROL_KIND_ID, ControlInspectionSection,
    LABEL_CONTROL_KIND_ID, LIST_VIEW_CONTROL_KIND_ID,
};

#[test]
fn base_controls_package_exposes_overlay_descriptors_for_all_controls() {
    let package = runenwerk_control_package();

    assert_eq!(package.control_kinds.len(), 8);
    assert_eq!(package.overlay_descriptors.len(), 8);
    assert!(package.validate_contract().is_valid());

    for kind in &package.control_kinds {
        let descriptor = package
            .overlay_descriptor(&kind.control_kind_id)
            .expect("every base control kind exposes package-backed overlay support");
        assert!(!descriptor.requirements.is_empty());
        assert_eq!(descriptor.control_kind_id, kind.control_kind_id);
    }
}

#[test]
fn base_controls_catalog_projects_overlay_support() {
    let catalog = BaseControlsPlugin::new().catalog();

    let button = catalog.entry(BUTTON_CONTROL_KIND_ID).expect("button catalog entry");
    assert!(button.overlay_supported);
    assert!(button.overlay_kinds.iter().any(|kind| kind == "popup"));
    assert!(button.overlay_triggers.iter().any(|trigger| trigger == "pointer-press"));
    assert!(!button.executes_host_commands);
    assert!(!button.mutates_product_state);

    let action = catalog
        .entry(ACTION_PROMPT_CONTROL_KIND_ID)
        .expect("action prompt catalog entry");
    assert!(action.overlay_kinds.iter().any(|kind| kind == "menu"));

    let picker = catalog
        .entry(COLOR_PICKER_CONTROL_KIND_ID)
        .expect("color picker catalog entry");
    assert!(picker
        .overlay_kinds
        .iter()
        .any(|kind| kind == "picker-popup"));

    let list = catalog.entry(LIST_VIEW_CONTROL_KIND_ID).expect("list catalog entry");
    assert!(list.overlay_kinds.iter().any(|kind| kind == "dropdown"));
}

#[test]
fn base_controls_inspection_projects_overlay_facts() {
    let inspection = BaseControlsPlugin::new().inspection();

    let label = inspection
        .descriptor(LABEL_CONTROL_KIND_ID)
        .expect("label inspection descriptor");
    assert_eq!(
        label.fact(ControlInspectionSection::Overlay, "overlay.supported"),
        Some("true")
    );
    assert!(label
        .fact(ControlInspectionSection::Overlay, "overlay.kinds")
        .unwrap_or_default()
        .contains("tooltip"));
}
