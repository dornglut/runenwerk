use ui_controls::runenwerk_control_package;

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
