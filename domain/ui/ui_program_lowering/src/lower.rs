//! File: domain/ui/ui_program_lowering/src/lower.rs
//! Crate: ui_program_lowering
//!
//! Semantic lowering from authored UI nodes into typed UiProgram graph families.

use ui_definition::{AuthoredUiNodePath, UiNodeDefinition};
use ui_program::{
    AccessibilityNode, AccessibilityNodeId, AccessibilityRole, BindingEdge, BindingEdgeId,
    BindingEndpoint, BindingEndpointId, ControlGraphNode, ControlKindRef, ControlNodeId,
    ControlPackageRef, InspectionEntry, InspectionEntryId, InteractionHandler,
    InteractionHandlerId, InteractionTrigger, LayoutConstraintId, LayoutGraphNode, RouteId,
    StateRequirement, StateRequirementId, StateRequirementLifecycle, StyleRule, StyleRuleId,
    StyleSlotId, UiProgram, UiProgramDiagnostic, VisualOperator, VisualOperatorId,
};

use crate::catalog::UiProgramFormationControlCatalog;
use crate::source_map::source_map_for_path;

pub(crate) fn lower_control_nodes(
    node: &UiNodeDefinition,
    path: &AuthoredUiNodePath,
    catalog: &UiProgramFormationControlCatalog,
    program: &mut UiProgram,
) {
    if let UiNodeDefinition::Control {
        id,
        kind,
        bindings,
        route,
        accessibility,
        ..
    } = node
    {
        let control_id = ControlNodeId::new(format!("control.{}", id.as_str()));
        let contract = catalog.control_kind(kind.as_str());
        let package_id = contract
            .map(|contract| contract.package_id.clone())
            .unwrap_or_else(|| control_package_from_kind(kind.as_str()));

        let mut control = ControlGraphNode::new(
            control_id.clone(),
            ControlPackageRef::new(package_id),
            ControlKindRef::new(kind.as_str()),
        )
        .with_source_map(source_map_for_path(
            path,
            format!("program.control.{}", id.as_str()),
        ));

        if let (Some(accessibility), Some(_contract)) = (accessibility.as_ref(), contract) {
            let accessibility_source_map =
                source_map_for_path(path, format!("program.accessibility.{}", id.as_str()));

            if let Some(role) = accessibility_role_from_authored(accessibility.role.as_str()) {
                let mut accessibility_node = AccessibilityNode::new(
                    AccessibilityNodeId::new(format!("accessibility.{}", id.as_str())),
                    control_id.clone(),
                    role,
                )
                .with_source_map(accessibility_source_map);

                if let Some(label) = accessibility.label.as_ref() {
                    accessibility_node = accessibility_node.with_label(label.clone());
                }

                program.graphs.accessibility.nodes.push(accessibility_node);
            } else {
                program.diagnostics.push(
                    UiProgramDiagnostic::new(
                        "ui.program.accessibility.unknown_role",
                        format!(
                            "control {} declares unknown accessibility role {}",
                            id.as_str(),
                            accessibility.role
                        ),
                    )
                    .with_source_map(accessibility_source_map.entry),
                );
            }
        }

        if let Some(contract) = contract {
            program.graphs.inspection.entries.push(
                InspectionEntry::new(
                    InspectionEntryId::new(format!("inspection.{}", id.as_str())),
                    control_id.clone(),
                    contract.display_name.clone(),
                    contract.property_schema.clone(),
                )
                .with_source_map(source_map_for_path(
                    path,
                    format!("program.inspection.{}", id.as_str()),
                )),
            );
        }

        if route.is_some() {
            if let Some(contract) = contract {
                control = control.with_capability(contract.activation_capability.clone());
            }
        }

        program.graphs.control.add_node(control);

        if let Some(contract) = contract {
            program.graphs.layout.constraints.push(
                LayoutGraphNode::new(
                    LayoutConstraintId::new(format!("layout.{}", id.as_str())),
                    control_id.clone(),
                )
                .with_layout_kernel(contract.layout_kernel.clone())
                .with_source_map(source_map_for_path(
                    path,
                    format!("program.layout.{}", id.as_str()),
                )),
            );

            program.graphs.style.rules.push(
                StyleRule::new(
                    StyleRuleId::new(format!("style.{}", id.as_str())),
                    control_id.clone(),
                    StyleSlotId::new(format!("style_slot.{}", id.as_str())),
                    contract.property_schema.clone(),
                )
                .with_source_map(source_map_for_path(
                    path,
                    format!("program.style.{}", id.as_str()),
                )),
            );

            program.graphs.visual.operators.push(
                VisualOperator::new(
                    VisualOperatorId::new(format!("visual.{}", id.as_str())),
                    control_id.clone(),
                    contract.visual_kernel.clone(),
                )
                .with_source_map(source_map_for_path(
                    path,
                    format!("program.visual.{}", id.as_str()),
                )),
            );
        }

        if let (Some(route), Some(contract)) = (route.as_ref(), contract) {
            program.graphs.interaction.handlers.push(
                InteractionHandler::new(
                    InteractionHandlerId::new(format!("interaction.{}.activate", id.as_str())),
                    control_id.clone(),
                    InteractionTrigger::Press,
                    RouteId::new(route.as_str()),
                    contract.event_payload_schema.clone(),
                )
                .with_capability(contract.activation_capability.clone())
                .with_source_map(source_map_for_path(
                    path,
                    format!("program.interaction.{}.activate", id.as_str()),
                )),
            );
        }

        if let Some(contract) = contract {
            for (binding_name, binding_ref) in bindings {
                let state_id =
                    StateRequirementId::new(format!("state.{}.{}", id.as_str(), binding_name));
                let state_endpoint_id =
                    BindingEndpointId::new(format!("state.{}.{}", id.as_str(), binding_name));
                let binding_id =
                    BindingEdgeId::new(format!("binding.{}.{}", id.as_str(), binding_name));

                program.graphs.state.requirements.push(
                    StateRequirement::new(
                        state_id.clone(),
                        control_id.clone(),
                        StateRequirementLifecycle::HostFed,
                        contract.state_schema.clone(),
                    )
                    .with_source_map(source_map_for_path(
                        path,
                        format!("program.state.{}.{}", id.as_str(), binding_name),
                    )),
                );

                program.graphs.binding.bindings.push(
                    BindingEdge::new(
                        binding_id,
                        BindingEndpoint::HostData {
                            endpoint_id: BindingEndpointId::new(binding_ref.as_str()),
                        },
                        BindingEndpoint::UiState {
                            requirement_id: state_id,
                            endpoint_id: state_endpoint_id,
                        },
                        contract.state_schema.clone(),
                    )
                    .with_source_map(source_map_for_path(
                        path,
                        format!("program.binding.{}.{}", id.as_str(), binding_name),
                    )),
                );
            }
        }
    }

    for child in node.children() {
        lower_control_nodes(child, &path.child(child.id()), catalog, program);
    }
}

fn control_package_from_kind(kind: &str) -> String {
    kind.rsplit_once('.')
        .map(|(package, _)| package.to_owned())
        .unwrap_or_else(|| kind.to_owned())
}

fn accessibility_role_from_authored(role: &str) -> Option<AccessibilityRole> {
    match role {
        "button" => Some(AccessibilityRole::Button),
        "label" => Some(AccessibilityRole::Label),
        "text_field" | "textfield" | "text-field" => Some(AccessibilityRole::TextField),
        "color_picker" | "color-picker" => Some(AccessibilityRole::ColorPicker),
        "list" => Some(AccessibilityRole::List),
        "tree" => Some(AccessibilityRole::Tree),
        "table" => Some(AccessibilityRole::Table),
        "prompt" => Some(AccessibilityRole::Prompt),
        _ => None,
    }
}
