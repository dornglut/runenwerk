// File: domain/ui/ui_definition/tests/ui_program_button_formation.rs
// Test: ui_gallery_button_basic_forms_control_graph_node
// Function: load_node

use std::fs;

use ui_definition::{
    UiNodeDefinition, UiProgramFormationControlCatalog, UiProgramFormationControlContract,
    form_ui_program_from_node, form_ui_program_from_node_with_catalog,
};
use ui_program::{AccessibilityRole, ControlKernelRef};
use ui_program::{BindingEndpoint, InteractionTrigger, RouteCapability, StateRequirementLifecycle};
use ui_schema::UiSchemaRef;

#[test]
fn ui_gallery_button_basic_forms_control_graph_node() {
    let node = load_node("assets/ui_gallery/button/basic.ron");

    let program = form_ui_program_from_node(
        "ui_gallery.button.basic",
        "assets.ui_gallery.button.basic",
        &node,
    );

    assert_eq!(program.graphs.control.nodes.len(), 1);

    let control = &program.graphs.control.nodes[0];

    assert_eq!(control.node_id.as_str(), "control.button_basic");
    assert_eq!(control.package_id.as_str(), "runenwerk.ui.controls");
    assert_eq!(
        control.control_kind.as_str(),
        "runenwerk.ui.controls.button"
    );
    assert!(control.source_map.is_some());

    assert_eq!(program.graphs.layout.constraints.len(), 0);
    assert_eq!(program.graphs.style.rules.len(), 0);
    assert_eq!(program.graphs.state.requirements.len(), 0);
    assert_eq!(program.graphs.interaction.handlers.len(), 0);
    assert_eq!(program.graphs.binding.bindings.len(), 0);
    assert_eq!(program.graphs.visual.operators.len(), 0);
    assert_eq!(program.graphs.accessibility.nodes.len(), 0);
    assert_eq!(program.graphs.inspection.entries.len(), 0);
}

// File: domain/ui/ui_definition/tests/ui_program_button_formation.rs
// Test: ui_gallery_button_basic_route_forms_press_interaction_handler

#[test]
fn ui_gallery_button_basic_route_forms_press_interaction_handler() {
    let node = load_node("assets/ui_gallery/button/basic.ron");

    let program = form_ui_program_from_node_with_catalog(
        "ui_gallery.button.basic",
        "assets.ui_gallery.button.basic",
        &node,
        &button_catalog(),
    );

    assert_eq!(program.graphs.control.nodes.len(), 1);
    assert_eq!(program.graphs.interaction.handlers.len(), 1);

    let control = &program.graphs.control.nodes[0];
    assert_eq!(control.node_id.as_str(), "control.button_basic");
    assert!(
        control
            .required_capabilities
            .iter()
            .any(|capability| capability.as_str() == "runenwerk.ui.controls.activate")
    );

    let handler = &program.graphs.interaction.handlers[0];
    assert_eq!(
        handler.handler_id.as_str(),
        "interaction.button_basic.activate"
    );
    assert_eq!(handler.control_id.as_str(), "control.button_basic");
    assert_eq!(handler.trigger, InteractionTrigger::Press);
    assert_eq!(handler.route.as_str(), "ui_gallery.button.basic.activate");
    assert_eq!(
        handler.payload_schema.id.as_str(),
        "runenwerk.ui.controls.button.event"
    );
    assert_eq!(handler.payload_schema.version.value(), 1);
    assert!(
        handler
            .required_capabilities
            .iter()
            .any(|capability| capability.as_str() == "runenwerk.ui.controls.activate")
    );
    assert!(handler.source_map.is_some());

    assert_eq!(program.graphs.layout.constraints.len(), 1);
    assert_eq!(program.graphs.style.rules.len(), 1);
    assert_eq!(program.graphs.visual.operators.len(), 1);
    assert_eq!(program.graphs.state.requirements.len(), 0);
    assert_eq!(program.graphs.binding.bindings.len(), 0);
    assert_eq!(program.graphs.accessibility.nodes.len(), 1);
    assert_eq!(program.graphs.inspection.entries.len(), 1);
}

#[test]
fn ui_gallery_button_selected_binding_forms_host_fed_state_and_binding_edge() {
    let node = load_node("assets/ui_gallery/button/selected.ron");

    let program = form_ui_program_from_node_with_catalog(
        "ui_gallery.button.selected",
        "assets.ui_gallery.button.selected",
        &node,
        &button_catalog(),
    );

    assert_eq!(program.graphs.control.nodes.len(), 1);
    assert_eq!(program.graphs.interaction.handlers.len(), 1);
    assert_eq!(program.graphs.state.requirements.len(), 1);
    assert_eq!(program.graphs.binding.bindings.len(), 1);

    let state = &program.graphs.state.requirements[0];

    assert_eq!(
        state.requirement_id.as_str(),
        "state.button_selected.selected"
    );
    assert_eq!(state.owner_control.as_str(), "control.button_selected");
    assert_eq!(state.lifecycle, StateRequirementLifecycle::HostFed);
    assert_eq!(
        state.schema.id.as_str(),
        "runenwerk.ui.controls.button.state"
    );
    assert_eq!(state.schema.version.value(), 1);
    assert!(state.source_map.is_some());

    let binding = &program.graphs.binding.bindings[0];

    assert_eq!(binding.edge_id.as_str(), "binding.button_selected.selected");
    assert_eq!(
        binding.value_schema.id.as_str(),
        "runenwerk.ui.controls.button.state"
    );
    assert_eq!(binding.value_schema.version.value(), 1);
    assert!(binding.source_map.is_some());

    assert!(matches!(
        &binding.source,
        BindingEndpoint::HostData { endpoint_id }
            if endpoint_id.as_str() == "ui_gallery.button.selected.active"
    ));

    assert!(matches!(
        &binding.target,
        BindingEndpoint::UiState {
            requirement_id,
            endpoint_id,
        } if requirement_id.as_str() == "state.button_selected.selected"
            && endpoint_id.as_str() == "state.button_selected.selected"
    ));

    assert_eq!(program.graphs.layout.constraints.len(), 1);
    assert_eq!(program.graphs.style.rules.len(), 1);
    assert_eq!(program.graphs.visual.operators.len(), 1);
    assert_eq!(program.graphs.accessibility.nodes.len(), 1);
    assert_eq!(program.graphs.inspection.entries.len(), 1);
}

// File: domain/ui/ui_definition/tests/ui_program_button_formation.rs
// Test: ui_gallery_button_basic_forms_layout_style_and_visual_kernels

#[test]
fn ui_gallery_button_basic_forms_layout_style_and_visual_kernels() {
    let node = load_node("assets/ui_gallery/button/basic.ron");

    let program = form_ui_program_from_node_with_catalog(
        "ui_gallery.button.basic",
        "assets.ui_gallery.button.basic",
        &node,
        &button_catalog(),
    );

    assert_eq!(program.graphs.control.nodes.len(), 1);
    assert_eq!(program.graphs.layout.constraints.len(), 1);
    assert_eq!(program.graphs.style.rules.len(), 1);
    assert_eq!(program.graphs.visual.operators.len(), 1);

    let layout = &program.graphs.layout.constraints[0];

    assert_eq!(layout.constraint_id.as_str(), "layout.button_basic");
    assert_eq!(layout.target_control.as_str(), "control.button_basic");
    assert_eq!(
        layout.layout_kernel.as_ref().map(|kernel| kernel.as_str()),
        Some("runenwerk.ui.controls.button.layout")
    );
    assert!(layout.source_map.is_some());

    let style = &program.graphs.style.rules[0];

    assert_eq!(style.rule_id.as_str(), "style.button_basic");
    assert_eq!(style.target_control.as_str(), "control.button_basic");
    assert_eq!(style.style_slot.as_str(), "style_slot.button_basic");
    assert_eq!(
        style.property_schema.id.as_str(),
        "runenwerk.ui.controls.button.properties"
    );
    assert_eq!(style.property_schema.version.value(), 1);
    assert!(style.source_map.is_some());

    let visual = &program.graphs.visual.operators[0];

    assert_eq!(visual.operator_id.as_str(), "visual.button_basic");
    assert_eq!(visual.control_id.as_str(), "control.button_basic");
    assert_eq!(
        visual.visual_kernel.as_str(),
        "runenwerk.ui.controls.button.visual"
    );
    assert!(visual.source_map.is_some());

    assert_eq!(program.graphs.state.requirements.len(), 0);
    assert_eq!(program.graphs.binding.bindings.len(), 0);
    assert_eq!(program.graphs.accessibility.nodes.len(), 1);
    assert_eq!(program.graphs.inspection.entries.len(), 1);
}

#[test]
fn ui_gallery_button_basic_forms_accessibility_node() {
    let node = load_node("assets/ui_gallery/button/basic.ron");

    let program = form_ui_program_from_node_with_catalog(
        "ui_gallery.button.basic",
        "assets.ui_gallery.button.basic",
        &node,
        &button_catalog(),
    );

    assert_eq!(program.graphs.control.nodes.len(), 1);
    assert_eq!(program.graphs.layout.constraints.len(), 1);
    assert_eq!(program.graphs.style.rules.len(), 1);
    assert_eq!(program.graphs.visual.operators.len(), 1);
    assert_eq!(program.graphs.interaction.handlers.len(), 1);
    assert_eq!(program.graphs.accessibility.nodes.len(), 1);

    let accessibility = &program.graphs.accessibility.nodes[0];

    assert_eq!(accessibility.node_id.as_str(), "accessibility.button_basic");
    assert_eq!(accessibility.control_id.as_str(), "control.button_basic");
    assert_eq!(accessibility.role, AccessibilityRole::Button);
    assert_eq!(accessibility.label.as_deref(), Some("Press demo button"));
    assert!(accessibility.label_source.is_none());
    assert!(accessibility.source_map.is_some());

    assert_eq!(program.graphs.state.requirements.len(), 0);
    assert_eq!(program.graphs.binding.bindings.len(), 0);
    assert_eq!(program.graphs.inspection.entries.len(), 1);
}

#[test]
fn ui_gallery_button_unknown_accessibility_role_reports_diagnostic() {
    let mut node = load_node("assets/ui_gallery/button/basic.ron");

    let UiNodeDefinition::Control { accessibility, .. } = &mut node else {
        panic!("expected generic Control node");
    };

    let accessibility = accessibility
        .as_mut()
        .expect("button basic fixture should have accessibility metadata");

    accessibility.role = "spellbook".to_owned();

    let program = form_ui_program_from_node_with_catalog(
        "ui_gallery.button.basic.invalid_accessibility",
        "assets.ui_gallery.button.basic",
        &node,
        &button_catalog(),
    );

    assert_eq!(program.graphs.accessibility.nodes.len(), 0);

    assert!(program.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "ui.program.accessibility.unknown_role"
            && diagnostic.message.contains("button_basic")
            && diagnostic.message.contains("spellbook")
            && diagnostic.source_map.is_some()
    }));
}

#[test]
fn ui_gallery_button_basic_forms_inspection_entry() {
    let node = load_node("assets/ui_gallery/button/basic.ron");

    let program = form_ui_program_from_node_with_catalog(
        "ui_gallery.button.basic",
        "assets.ui_gallery.button.basic",
        &node,
        &button_catalog(),
    );

    assert_eq!(program.graphs.control.nodes.len(), 1);
    assert_eq!(program.graphs.layout.constraints.len(), 1);
    assert_eq!(program.graphs.style.rules.len(), 1);
    assert_eq!(program.graphs.visual.operators.len(), 1);
    assert_eq!(program.graphs.interaction.handlers.len(), 1);
    assert_eq!(program.graphs.accessibility.nodes.len(), 1);
    assert_eq!(program.graphs.inspection.entries.len(), 1);

    let inspection = &program.graphs.inspection.entries[0];

    assert_eq!(inspection.entry_id.as_str(), "inspection.button_basic");
    assert_eq!(inspection.control_id.as_str(), "control.button_basic");
    assert_eq!(inspection.display_name, "Button");
    assert_eq!(
        inspection.value_schema.id.as_str(),
        "runenwerk.ui.controls.button.properties"
    );
    assert_eq!(inspection.value_schema.version.value(), 1);
    assert!(inspection.binding.is_none());
    assert!(inspection.source_map.is_some());

    assert_eq!(program.graphs.state.requirements.len(), 0);
    assert_eq!(program.graphs.binding.bindings.len(), 0);
}

fn button_catalog() -> UiProgramFormationControlCatalog {
    UiProgramFormationControlCatalog::new().with_control_kind(
        UiProgramFormationControlContract::new(
            "runenwerk.ui.controls.button",
            "runenwerk.ui.controls",
            "Button",
            UiSchemaRef::new("runenwerk.ui.controls.button.properties", 1),
            UiSchemaRef::new("runenwerk.ui.controls.button.state", 1),
            UiSchemaRef::new("runenwerk.ui.controls.button.event", 1),
            ControlKernelRef::new("runenwerk.ui.controls.button.layout"),
            ControlKernelRef::new("runenwerk.ui.controls.button.visual"),
            RouteCapability::new("runenwerk.ui.controls.activate"),
        ),
    )
}

fn load_node(relative_repo_path: &str) -> UiNodeDefinition {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(relative_repo_path);

    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {:?}: {error}", path));

    ron::from_str(&source).unwrap_or_else(|error| panic!("failed to parse {:?}: {error}", path))
}
