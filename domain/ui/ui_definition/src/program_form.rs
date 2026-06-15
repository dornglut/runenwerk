// File: domain/ui/ui_definition/src/program_form.rs
// Functions: form_ui_program_from_node, form_control_nodes, control_package_from_kind, source_id_for_path

use std::collections::BTreeMap;

use crate::{AuthoredUiNodePath, UiNodeDefinition};
use ui_program::{
    AccessibilityNode, AccessibilityNodeId, AccessibilityRole, BindingEdge, BindingEdgeId,
    BindingEndpoint, BindingEndpointId, ControlGraphNode, ControlKernelRef, ControlKindRef,
    ControlNodeId, ControlPackageRef, InspectionEntry, InspectionEntryId, InteractionHandler,
    InteractionHandlerId, InteractionTrigger, LayoutConstraintId, LayoutGraphNode, RouteCapability,
    RouteId, StateRequirement, StateRequirementId, StateRequirementLifecycle, StyleRule,
    StyleRuleId, StyleSlotId, UiProgram, UiProgramDiagnostic, UiProgramId, UiProgramSource,
    UiProgramSourceId, UiProgramSourceMapAttachment, UiProgramSourceMapEntry, UiProgramTargetId,
    UiProgramVersion, VisualOperator, VisualOperatorId,
};
use ui_schema::UiSchemaRef;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiProgramFormationControlCatalog {
    control_kinds: BTreeMap<String, UiProgramFormationControlContract>,
}

impl UiProgramFormationControlCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_control_kind(mut self, contract: UiProgramFormationControlContract) -> Self {
        self.control_kinds
            .insert(contract.kind_id.clone(), contract);
        self
    }

    fn control_kind(&self, kind_id: &str) -> Option<&UiProgramFormationControlContract> {
        self.control_kinds.get(kind_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiProgramFormationControlContract {
    pub kind_id: String,
    pub package_id: String,
    pub display_name: String,
    pub property_schema: UiSchemaRef,
    pub state_schema: UiSchemaRef,
    pub event_payload_schema: UiSchemaRef,
    pub layout_kernel: ControlKernelRef,
    pub visual_kernel: ControlKernelRef,
    pub activation_capability: RouteCapability,
}

impl UiProgramFormationControlContract {
    pub fn new(
        kind_id: impl Into<String>,
        package_id: impl Into<String>,
        display_name: impl Into<String>,
        property_schema: UiSchemaRef,
        state_schema: UiSchemaRef,
        event_payload_schema: UiSchemaRef,
        layout_kernel: ControlKernelRef,
        visual_kernel: ControlKernelRef,
        activation_capability: RouteCapability,
    ) -> Self {
        Self {
            kind_id: kind_id.into(),
            package_id: package_id.into(),
            display_name: display_name.into(),
            event_payload_schema,
            layout_kernel,
            state_schema,
            activation_capability,
            property_schema,
            visual_kernel,
        }
    }
}

pub fn form_ui_program_from_node(
    program_id: impl Into<String>,
    source_id: impl Into<String>,
    root: &UiNodeDefinition,
) -> UiProgram {
    form_ui_program_from_node_with_catalog(
        program_id,
        source_id,
        root,
        &UiProgramFormationControlCatalog::default(),
    )
}

pub fn form_ui_program_from_node_with_catalog(
    program_id: impl Into<String>,
    source_id: impl Into<String>,
    root: &UiNodeDefinition,
    catalog: &UiProgramFormationControlCatalog,
) -> UiProgram {
    let program_id = program_id.into();
    let source_id = source_id.into();

    let mut program = UiProgram::new(
        UiProgramId::new(program_id.clone()),
        UiProgramVersion::new(1),
    )
    .with_source(UiProgramSource::authored(
        UiProgramSourceId::new(source_id.clone()),
        "authored UI definition",
    ))
    .with_source_map_entry(UiProgramSourceMapEntry::new(
        UiProgramSourceId::new(source_id),
        UiProgramTargetId::new(format!("{program_id}.root")),
    ));

    form_control_nodes(
        root,
        &AuthoredUiNodePath::root(root.id()),
        catalog,
        &mut program,
    );

    program
}

fn form_control_nodes(
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
        form_control_nodes(child, &path.child(child.id()), catalog, program);
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

fn source_map_for_path(
    path: &AuthoredUiNodePath,
    target_id: impl Into<String>,
) -> UiProgramSourceMapAttachment {
    UiProgramSourceMapAttachment::new(UiProgramSourceMapEntry::new(
        UiProgramSourceId::new(format!("definition.{}", path.as_str().replace('/', "."))),
        UiProgramTargetId::new(target_id),
    ))
}
