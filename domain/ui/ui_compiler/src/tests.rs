use super::*;
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
