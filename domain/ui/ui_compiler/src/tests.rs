use super::*;
use std::{fs, path::PathBuf};
use ui_definition::{
    UiNodeDefinition, UiProgramFormationControlCatalog, UiProgramFormationControlContract,
    form_ui_program_from_node_with_catalog,
};
use ui_program::UiSchemaRef;
use ui_program::{
    AccessibilityNode, AccessibilityNodeId, AccessibilityRole, BindingEdge, BindingEdgeId,
    BindingEndpoint, BindingEndpointId, ControlGraphNode, ControlKernelRef, ControlKindRef,
    ControlNodeId, ControlPackageRef, InspectionEntry, InspectionEntryId, InteractionHandler,
    InteractionHandlerId, InteractionTrigger, LayoutConstraintId, LayoutGraphNode, RouteCapability,
    RouteId, StateRequirement, StateRequirementId, StateRequirementLifecycle, UiProgram,
    UiProgramId, UiProgramSourceId, UiProgramSourceMapAttachment, UiProgramSourceMapEntry,
    UiProgramSourceSpan, UiProgramTargetId, UiProgramVersion, VisualOperator, VisualOperatorId,
};

#[test]
fn compiler_contract_resolves_packages_capabilities_cache_keys_and_source_maps() {
    let program = compiler_program(RouteCapability::new("editor.inspector.read"));
    let report = UiCompiler.compile_report(&program);

    assert!(report.passed());
    assert_eq!(report.package_resolution.packages.len(), 1);
    assert_eq!(
        report.package_resolution.packages[0].kernel_ids,
        [
            "runenwerk.ui.controls.label.layout",
            "runenwerk.ui.controls.label.visual"
        ]
    );
    assert!(
        report
            .capability_checks
            .iter()
            .any(|check| check.status == CapabilityCheckStatus::SatisfiedByControl)
    );
    assert!(
        report
            .cache_key()
            .as_str()
            .starts_with("ui-program:editor.inspector:2:")
    );
    assert!(
        report
            .compiled_source_map()
            .entries
            .iter()
            .any(|entry| entry.target_id == "program.control.title")
    );
    assert_eq!(report.artifact.manifest.diagnostics, []);
    assert_eq!(report.artifact.tables.controls.rows.len(), 1);
    assert_eq!(report.artifact.tables.binding_snapshots.rows.len(), 1);
}

#[test]
fn compiler_contract_reports_missing_capability_declarations() {
    let program = compiler_program(RouteCapability::new("editor.inspector.write"));
    let report = UiCompiler.compile_report(&program);

    assert!(!report.passed());
    assert!(
        report
            .artifact
            .manifest
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code
                == "ui.compiler.capability.missing_control_declaration")
    );
}

fn compiler_program(handler_capability: RouteCapability) -> UiProgram {
    let mut program = UiProgram::new(
        UiProgramId::new("editor.inspector"),
        UiProgramVersion::new(2),
    )
    .with_source_map_entry(UiProgramSourceMapEntry::new(
        UiProgramSourceId::new("definition.inspector"),
        UiProgramTargetId::new("program.inspector"),
    ));
    let source_map = || {
        UiProgramSourceMapAttachment::new(UiProgramSourceMapEntry::new(
            UiProgramSourceId::new("definition.title"),
            UiProgramTargetId::new("program.control.title"),
        ))
        .with_source_span(UiProgramSourceSpan::new(1, 12))
    };
    program.graphs.control.add_node(
        ControlGraphNode::new(
            ControlNodeId::new("control.title"),
            ControlPackageRef::new("runenwerk.ui.controls"),
            ControlKindRef::new("runenwerk.ui.controls.label"),
        )
        .with_capability(RouteCapability::new("editor.inspector.read"))
        .with_source_map(source_map()),
    );
    program.graphs.layout.constraints.push(
        LayoutGraphNode::new(
            LayoutConstraintId::new("layout.title"),
            ControlNodeId::new("control.title"),
        )
        .with_layout_kernel(ControlKernelRef::new("runenwerk.ui.controls.label.layout"))
        .with_source_map(source_map()),
    );
    program.graphs.state.requirements.push(
        StateRequirement::new(
            StateRequirementId::new("state.title"),
            ControlNodeId::new("control.title"),
            StateRequirementLifecycle::HostFed,
            UiSchemaRef::new("ui.label.state", 1),
        )
        .with_source_map(source_map()),
    );
    program.graphs.interaction.handlers.push(
        InteractionHandler::new(
            InteractionHandlerId::new("interaction.title.preview"),
            ControlNodeId::new("control.title"),
            InteractionTrigger::ValuePreview,
            RouteId::new("editor.inspector.preview"),
            UiSchemaRef::new("ui.label.properties", 1),
        )
        .with_capability(handler_capability)
        .with_source_map(source_map()),
    );
    program.graphs.binding.bindings.push(
        BindingEdge::new(
            BindingEdgeId::new("binding.title"),
            BindingEndpoint::HostData {
                endpoint_id: BindingEndpointId::new("host.selection.title"),
            },
            BindingEndpoint::ControlProperty {
                control_id: ControlNodeId::new("control.title"),
                endpoint_id: BindingEndpointId::new("control.title.text"),
            },
            UiSchemaRef::new("ui.label.properties", 1),
        )
        .with_capability(RouteCapability::new("editor.inspector.read")),
    );
    program.graphs.visual.operators.push(
        VisualOperator::new(
            VisualOperatorId::new("visual.title"),
            ControlNodeId::new("control.title"),
            ControlKernelRef::new("runenwerk.ui.controls.label.visual"),
        )
        .with_source_map(source_map()),
    );
    program
        .graphs
        .accessibility
        .nodes
        .push(AccessibilityNode::new(
            AccessibilityNodeId::new("accessibility.title"),
            ControlNodeId::new("control.title"),
            AccessibilityRole::Label,
        ));
    program.graphs.inspection.entries.push(InspectionEntry::new(
        InspectionEntryId::new("inspection.title"),
        ControlNodeId::new("control.title"),
        "Title",
        UiSchemaRef::new("ui.label.properties", 1),
    ));
    program
}

// File: domain/ui/ui_compiler/src/tests.rs
// Test: compiler_lowers_authored_button_program_to_runtime_artifact

#[test]
fn compiler_lowers_authored_button_program_to_runtime_artifact() {
    let node = load_authored_node("assets/ui_gallery/button/selected.ron");

    let program = form_ui_program_from_node_with_catalog(
        "ui_gallery.button.selected",
        "assets.ui_gallery.button.selected",
        &node,
        &button_catalog(),
    );

    let report = UiCompiler.compile_report(&program);

    assert!(
        report.passed(),
        "{:?}",
        report.artifact.manifest.diagnostics
    );
    assert_eq!(report.artifact.manifest.diagnostics, []);

    assert_eq!(report.artifact.tables.controls.rows.len(), 1);
    assert_eq!(report.artifact.tables.layout.rows.len(), 1);
    assert_eq!(report.artifact.tables.style.rows.len(), 1);
    assert_eq!(report.artifact.tables.state.rows.len(), 1);
    assert_eq!(report.artifact.tables.interaction.rows.len(), 1);
    assert_eq!(report.artifact.tables.binding_snapshots.rows.len(), 1);
    assert_eq!(report.artifact.tables.collection_diffs.rows.len(), 1);
    assert_eq!(report.artifact.tables.visual.rows.len(), 1);
    assert_eq!(report.artifact.tables.accessibility.rows.len(), 1);
    assert_eq!(report.artifact.tables.inspection.rows.len(), 1);

    assert!(
        report
            .artifact
            .manifest
            .package_ids
            .iter()
            .any(|package_id| package_id == "runenwerk.ui.controls")
    );

    assert!(
        report
            .artifact
            .manifest
            .control_kind_ids
            .iter()
            .any(|kind_id| kind_id == "runenwerk.ui.controls.button")
    );

    assert!(
        report
            .artifact
            .manifest
            .schema_ids
            .iter()
            .any(
                |schema| schema.schema_id == "runenwerk.ui.controls.button.properties"
                    && schema.schema_version == 1
            )
    );

    assert!(
        report
            .artifact
            .manifest
            .schema_ids
            .iter()
            .any(
                |schema| schema.schema_id == "runenwerk.ui.controls.button.state"
                    && schema.schema_version == 1
            )
    );

    assert!(
        report
            .artifact
            .manifest
            .schema_ids
            .iter()
            .any(
                |schema| schema.schema_id == "runenwerk.ui.controls.button.event"
                    && schema.schema_version == 1
            )
    );

    assert!(report.artifact.manifest.route_ids.iter().any(|route| {
        route.route_id == "ui_gallery.button.selected.activate"
            && route.payload_schema.schema_id == "runenwerk.ui.controls.button.event"
            && route.payload_schema.schema_version == 1
    }));

    assert!(
        report
            .artifact
            .manifest
            .kernel_ids
            .iter()
            .any(|kernel_id| kernel_id == "runenwerk.ui.controls.button.layout")
    );

    assert!(
        report
            .artifact
            .manifest
            .kernel_ids
            .iter()
            .any(|kernel_id| kernel_id == "runenwerk.ui.controls.button.visual")
    );

    assert!(
        report
            .artifact
            .manifest
            .capability_ids
            .iter()
            .any(|capability_id| capability_id == "runenwerk.ui.controls.activate")
    );
}

fn load_authored_node(path: &str) -> UiNodeDefinition {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("ui_compiler should live under domain/ui/ui_compiler")
        .to_path_buf();

    let fixture_path = repo_root.join(path);
    let source = fs::read_to_string(&fixture_path).unwrap_or_else(|error| {
        panic!(
            "fixture {} should be readable: {error}",
            fixture_path.display()
        )
    });

    ron::from_str(&source).expect("fixture should parse as UiNodeDefinition")
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
