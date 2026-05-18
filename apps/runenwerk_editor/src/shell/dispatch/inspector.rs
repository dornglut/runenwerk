//! File: apps/runenwerk_editor/src/shell/dispatch/inspector.rs
//! Purpose: Inspector surface command dispatch.

use editor_core::{ComponentTypeId, EditorMutationError};
use editor_inspector::{InspectorEditValue, InspectorValue, enum_symbol_edit_value_for_field};
use editor_shell::{
    InspectorSessionMutation, StructuralCommandTarget, ToolSurfaceKind, inspector_field_widget_id,
};
use ui_surface::{
    ObservationFrame, RatificationAdapter, RatificationDispatchError, RatificationOutcome,
    SessionScopeHandle, SurfaceCapability, SurfaceCapabilitySet, SurfaceIntent, SurfaceIntentKind,
    SurfacePresentationModel, ratify_surface_intent,
};

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::execute_intent_with_history_from_origin;
use crate::editor_panels::{InspectorPanelPresenter, InspectorPanelViewModel};
use crate::editor_runtime::{
    EditorInspectorUiState, is_local_transform_component, select_single_component_with_origin,
};
use crate::shell::RunenwerkEditorShellState;
use crate::shell::dispatch::{
    resolve_legacy_surface_command_contract, surface_capability_label, tool_surface_kind_label,
};

const TRANSFORM_STEPPER_INCREMENT: f64 = 0.25;

pub(crate) fn dispatch_session_mutation(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&mut RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    mutation: InspectorSessionMutation,
) -> Result<(), EditorMutationError> {
    let Some(surface_contract) = resolve_legacy_surface_command_contract(
        shell_state.as_deref(),
        target,
        ToolSurfaceKind::Inspector,
    ) else {
        app.append_console_line(
            "[inspector] session mutation ignored (missing structural tool-surface target)"
                .to_string(),
        );
        return Ok(());
    };
    if surface_contract.tool_surface_kind != ToolSurfaceKind::Inspector {
        app.append_console_line(format!(
            "[inspector] session mutation ignored (surface-kind mismatch): expected=inspector actual={}",
            tool_surface_kind_label(surface_contract.tool_surface_kind),
        ));
        return Ok(());
    }

    match mutation {
        InspectorSessionMutation::ActivateField { index } => {
            dispatch_activate_field(app, target, surface_contract, index)
        }
        InspectorSessionMutation::FocusField { index } => {
            mutate_inspector_surface_session(app, target, |app, state| {
                focus_inspector_field(app, state, index)
            })?;
            if let Some(state) = shell_state {
                state
                    .runtime_mut()
                    .set_focused_widget(Some(inspector_field_widget_id(index)));
            }
            Ok(())
        }
        InspectorSessionMutation::AppendFieldText { index, text } => {
            mutate_inspector_surface_session(app, target, |app, state| {
                append_inspector_field_text(app, state, index, &text)
            })
        }
        InspectorSessionMutation::BackspaceFieldText { index } => {
            mutate_inspector_surface_session(app, target, |app, state| {
                backspace_inspector_field_text(app, state, index)
            })
        }
        InspectorSessionMutation::CommitFieldText { index } => {
            mutate_inspector_surface_session(app, target, |app, state| {
                commit_inspector_field_text(app, state, index)
            })
        }
        InspectorSessionMutation::CancelFieldText { index: _ } => {
            mutate_inspector_surface_session(app, target, |_app, state| {
                state.cancel_field_draft();
                Ok(())
            })
        }
        InspectorSessionMutation::SetFieldBool { index, value } => {
            mutate_inspector_surface_session(app, target, |app, state| {
                commit_inspector_bool_field(app, state, index, value)
            })
        }
        InspectorSessionMutation::SetFieldNumber { index, value } => {
            mutate_inspector_surface_session(app, target, |app, state| {
                commit_inspector_number_field(app, state, index, value)
            })
        }
        InspectorSessionMutation::SetFieldEnum { index, value } => {
            mutate_inspector_surface_session(app, target, |app, state| {
                commit_inspector_enum_field(app, state, index, value)
            })
        }
    }
}

fn dispatch_activate_field(
    app: &mut RunenwerkEditorApp,
    target: StructuralCommandTarget,
    surface_contract: crate::shell::dispatch::LegacySurfaceCommandContract,
    index: usize,
) -> Result<(), EditorMutationError> {
    for required_capability in [SurfaceCapability::Observe, SurfaceCapability::Interact] {
        if !surface_contract.capabilities.allows(required_capability) {
            app.append_console_line(format!(
                "[inspector] field activation ignored (missing capability): index={} capability={}",
                index,
                surface_capability_label(required_capability),
            ));
            return Ok(());
        }
    }

    let Some(tool_surface_id) = target.active_tool_surface else {
        app.append_console_line(
            "[inspector] field activation ignored (missing tool-surface session target)"
                .to_string(),
        );
        return Ok(());
    };
    let mut session = app.surface_sessions().session_or_default(tool_surface_id);
    let inspector_view =
        InspectorPanelPresenter::build_view_model(app.runtime(), &session.inspector_ui_state);
    let presentation_model =
        build_inspector_surface_presentation_model(app.runtime(), &inspector_view);
    if app.debug_logs_enabled() {
        app.append_console_line(format!(
            "[surface.flow] observation inspector selected_field={:?}",
            presentation_model.selected_primary_item,
        ));
        app.append_console_line(format!(
            "[surface.flow] presentation inspector selectable={:?}",
            presentation_model
                .selectable_primary_items()
                .collect::<Vec<_>>(),
        ));
    }
    let field_index = index as u64;
    if !presentation_model.is_primary_selectable(field_index) {
        app.append_console_line(format!(
            "[inspector] field activation ignored (unavailable): index={}",
            index,
        ));
        return Ok(());
    }

    let _session_scope = SessionScopeHandle::new(
        surface_contract.surface_instance_id,
        target.panel_instance_id.raw(),
        surface_contract.retention_class,
    );
    let intent = SurfaceIntent::activate_field(surface_contract.surface_instance_id, field_index);
    if app.debug_logs_enabled() {
        app.append_console_line(format!(
            "[surface.flow] intent inspector surface_instance={} kind=ActivateField field_index={}",
            intent.surface_instance_id.raw(),
            field_index,
        ));
    }
    let mut ratification_adapter = InspectorFieldActivationRatificationAdapter::new(
        app,
        &mut session.inspector_ui_state,
        surface_contract.capabilities,
    );
    match ratify_surface_intent(&mut ratification_adapter, intent) {
        Ok(RatificationOutcome::Applied) => {
            if app.debug_logs_enabled() {
                app.append_console_line(
                    "[surface.flow] ratification inspector outcome=applied".to_string(),
                );
            }
        }
        Ok(RatificationOutcome::Ignored) => {
            if app.debug_logs_enabled() {
                app.append_console_line(
                    "[surface.flow] ratification inspector outcome=ignored".to_string(),
                );
            }
        }
        Err(RatificationDispatchError::MissingCapability(capability)) => {
            app.append_console_line(format!(
                "[inspector] field activation ignored (missing capability): index={} capability={}",
                index,
                surface_capability_label(capability),
            ));
        }
        Err(RatificationDispatchError::Adapter(error)) => return Err(error),
    }
    *app.surface_sessions_mut().session_mut(tool_surface_id) = session;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct InspectorObservationFrameAdapter {
    selected_field_index: Option<u64>,
}

impl ObservationFrame<u64> for InspectorObservationFrameAdapter {
    fn selected_primary_item(&self) -> Option<u64> {
        self.selected_field_index
    }

    fn is_item_available(&self, _item_id: u64) -> bool {
        true
    }
}

fn build_inspector_surface_presentation_model(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
    inspector_view: &InspectorPanelViewModel,
) -> SurfacePresentationModel<u64> {
    let selectable_indices: Vec<u64> = match inspector_view {
        InspectorPanelViewModel::Entity {
            components,
            available_component_types,
            ..
        } => (0..components.len() + available_component_types.len())
            .map(|index| index as u64)
            .collect(),
        InspectorPanelViewModel::Component {
            component_type,
            widget_fields,
            ..
        } => widget_fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| {
                next_shell_edit_value(runtime, *component_type, field).map(|_| index as u64)
            })
            .collect(),
        InspectorPanelViewModel::Empty
        | InspectorPanelViewModel::Resource { .. }
        | InspectorPanelViewModel::Unsupported { .. }
        | InspectorPanelViewModel::Error { .. } => Vec::new(),
    };

    let adapter = InspectorObservationFrameAdapter {
        selected_field_index: None,
    };
    SurfacePresentationModel::from_observation_frame(&adapter, selectable_indices)
}

struct InspectorFieldActivationRatificationAdapter<'a> {
    app: &'a mut RunenwerkEditorApp,
    inspector_state: &'a mut EditorInspectorUiState,
    capabilities: SurfaceCapabilitySet,
}

impl<'a> InspectorFieldActivationRatificationAdapter<'a> {
    fn new(
        app: &'a mut RunenwerkEditorApp,
        inspector_state: &'a mut EditorInspectorUiState,
        capabilities: SurfaceCapabilitySet,
    ) -> Self {
        Self {
            app,
            inspector_state,
            capabilities,
        }
    }
}

impl RatificationAdapter for InspectorFieldActivationRatificationAdapter<'_> {
    type Error = EditorMutationError;

    fn has_capability(&self, capability: SurfaceCapability) -> bool {
        self.capabilities.allows(capability)
    }

    fn ratify_intent(&mut self, intent: SurfaceIntent) -> Result<RatificationOutcome, Self::Error> {
        let field_index = match intent.kind {
            SurfaceIntentKind::ActivateField { field_index } => field_index,
            _ => return Ok(RatificationOutcome::Ignored),
        };
        let index = usize::try_from(field_index).map_err(|_| {
            EditorMutationError::inspector_rejected("inspector field index overflow")
        })?;
        activate_inspector_field(self.app, self.inspector_state, index)?;
        Ok(RatificationOutcome::Applied)
    }
}

fn activate_inspector_field(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
) -> Result<(), EditorMutationError> {
    let inspector_view = InspectorPanelPresenter::build_view_model(app.runtime(), inspector_state);

    match inspector_view {
        InspectorPanelViewModel::Entity {
            entity,
            components,
            available_component_types,
            ..
        } => {
            if let Some(component) = components.get(index) {
                select_single_component_with_origin(
                    app.runtime_mut(),
                    component.entity,
                    component.component_type,
                    editor_core::ChangeOrigin::InspectorPanel,
                )?;
                inspector_state.clear_draft();
                inspector_state.clear_focus();
                return Ok(());
            }

            let offset = index.saturating_sub(components.len());
            let Some(candidate) = available_component_types.get(offset) else {
                return Err(EditorMutationError::inspector_rejected(
                    "inspector field index out of range",
                ));
            };

            if candidate.already_attached {
                return Ok(());
            }

            execute_intent_with_history_from_origin(
                app.runtime_mut(),
                "Add Component",
                editor_scene::SceneCommandIntent::AddComponent {
                    entity,
                    component_type: candidate.component_type,
                },
                editor_core::ChangeOrigin::InspectorPanel,
            )
            .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
            inspector_state.clear_draft();
            inspector_state.clear_focus();
            Ok(())
        }
        InspectorPanelViewModel::Component {
            entity,
            component_type,
            widget_fields,
            ..
        } => {
            let field = widget_fields
                .get(index)
                .ok_or(EditorMutationError::inspector_rejected(
                    "inspector field index out of range",
                ))?;

            let next_value = next_shell_edit_value(app.runtime(), component_type, field).ok_or(
                EditorMutationError::inspector_rejected("inspector field is not editable"),
            )?;

            commit_inspector_component_field_edit(
                app,
                inspector_state,
                entity,
                component_type,
                field.path.clone(),
                next_value,
            )
        }
        InspectorPanelViewModel::Empty
        | InspectorPanelViewModel::Resource { .. }
        | InspectorPanelViewModel::Unsupported { .. }
        | InspectorPanelViewModel::Error { .. } => Err(EditorMutationError::inspector_rejected(
            "shell inspector field activation requires entity/component target",
        )),
    }
}

fn mutate_inspector_surface_session(
    app: &mut RunenwerkEditorApp,
    target: StructuralCommandTarget,
    mutate: impl FnOnce(
        &mut RunenwerkEditorApp,
        &mut EditorInspectorUiState,
    ) -> Result<(), EditorMutationError>,
) -> Result<(), EditorMutationError> {
    let Some(surface_id) = target.active_tool_surface else {
        return Err(EditorMutationError::inspector_rejected(
            "inspector command requires surface instance target",
        ));
    };
    let mut session = app.surface_sessions().session_or_default(surface_id);
    let result = mutate(app, &mut session.inspector_ui_state);
    *app.surface_sessions_mut().session_mut(surface_id) = session;
    result
}

fn focus_inspector_field(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
) -> Result<(), EditorMutationError> {
    let (entity, component_type, field) =
        inspector_component_field_at_index(app, inspector_state, index)?;
    let text = inspector_current_draft_text(&field, true);
    apply_inspector_draft_text(inspector_state, entity, component_type, &field, text)
}

fn append_inspector_field_text(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
    text: &str,
) -> Result<(), EditorMutationError> {
    let (entity, component_type, field) =
        inspector_component_field_at_index(app, inspector_state, index)?;
    let mut next_text = inspector_current_draft_text(&field, false);
    next_text.push_str(text);
    apply_inspector_draft_text(inspector_state, entity, component_type, &field, next_text)
}

fn backspace_inspector_field_text(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
) -> Result<(), EditorMutationError> {
    let (entity, component_type, field) =
        inspector_component_field_at_index(app, inspector_state, index)?;
    let mut next_text = inspector_current_draft_text(&field, true);
    let _ = next_text.pop();
    apply_inspector_draft_text(inspector_state, entity, component_type, &field, next_text)
}

fn commit_inspector_field_text(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
) -> Result<(), EditorMutationError> {
    let (entity, component_type, field) =
        inspector_component_field_at_index(app, inspector_state, index)?;
    let text = inspector_current_draft_text(&field, true);
    let value = parse_inspector_field_text(&field, &text).ok_or(
        EditorMutationError::inspector_rejected("inspector field text is invalid for target type"),
    )?;
    commit_inspector_component_field_edit(
        app,
        inspector_state,
        entity,
        component_type,
        field.path,
        value,
    )
}

fn commit_inspector_bool_field(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
    value: bool,
) -> Result<(), EditorMutationError> {
    let (entity, component_type, field) =
        inspector_component_field_at_index(app, inspector_state, index)?;
    match field.value {
        InspectorValue::Bool(_) => commit_inspector_component_field_edit(
            app,
            inspector_state,
            entity,
            component_type,
            field.path,
            InspectorEditValue::Bool(value),
        ),
        _ => Err(EditorMutationError::inspector_rejected(
            "inspector bool mutation requires bool field",
        )),
    }
}

fn commit_inspector_number_field(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
    value: f64,
) -> Result<(), EditorMutationError> {
    if !value.is_finite() {
        return Err(EditorMutationError::inspector_rejected(
            "inspector numeric mutation requires finite value",
        ));
    }
    let (entity, component_type, field) =
        inspector_component_field_at_index(app, inspector_state, index)?;
    let edit_value = match field.value {
        InspectorValue::Integer(_) => {
            if value < i64::MIN as f64 || value > i64::MAX as f64 {
                return Err(EditorMutationError::inspector_rejected(
                    "inspector integer mutation out of range",
                ));
            }
            InspectorEditValue::Integer(value.round() as i64)
        }
        InspectorValue::Float(_) => InspectorEditValue::Float(value),
        _ => {
            return Err(EditorMutationError::inspector_rejected(
                "inspector numeric mutation requires numeric field",
            ));
        }
    };
    commit_inspector_component_field_edit(
        app,
        inspector_state,
        entity,
        component_type,
        field.path,
        edit_value,
    )
}

fn commit_inspector_enum_field(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
    value: String,
) -> Result<(), EditorMutationError> {
    let (entity, component_type, field) =
        inspector_component_field_at_index(app, inspector_state, index)?;
    let edit_value = enum_symbol_edit_value_for_field(&field.value, value).map_err(|_| {
        EditorMutationError::inspector_rejected(
            "inspector enum mutation requires a declared enum option",
        )
    })?;
    commit_inspector_component_field_edit(
        app,
        inspector_state,
        entity,
        component_type,
        field.path,
        edit_value,
    )
}

fn inspector_component_field_at_index(
    app: &mut RunenwerkEditorApp,
    inspector_state: &EditorInspectorUiState,
    index: usize,
) -> Result<
    (
        editor_core::EntityId,
        editor_core::ComponentTypeId,
        crate::editor_panels::InspectorWidgetField,
    ),
    EditorMutationError,
> {
    let inspector_view = InspectorPanelPresenter::build_view_model(app.runtime(), inspector_state);
    match inspector_view {
        InspectorPanelViewModel::Component {
            entity,
            component_type,
            widget_fields,
            ..
        } => {
            let field = widget_fields
                .get(index)
                .ok_or(EditorMutationError::inspector_rejected(
                    "inspector field index out of range",
                ))?
                .clone();
            Ok((entity, component_type, field))
        }
        _ => Err(EditorMutationError::inspector_rejected(
            "inspector text editing requires component target",
        )),
    }
}

fn inspector_current_draft_text(
    field: &crate::editor_panels::InspectorWidgetField,
    include_base_value: bool,
) -> String {
    if let Some(text) = &field.draft_text {
        return text.clone();
    }
    if include_base_value {
        return inspector_value_text(&field.value);
    }
    String::new()
}

fn apply_inspector_draft_text(
    inspector_state: &mut EditorInspectorUiState,
    entity: editor_core::EntityId,
    component_type: editor_core::ComponentTypeId,
    field: &crate::editor_panels::InspectorWidgetField,
    text: String,
) -> Result<(), EditorMutationError> {
    let parsed_value = parse_inspector_field_text(field, &text);
    let initial_value = parsed_value
        .clone()
        .or_else(|| editable_value_from_field(field))
        .ok_or(EditorMutationError::inspector_rejected(
            "inspector field is not editable",
        ))?;

    inspector_state.begin_field_edit(
        entity,
        component_type,
        field.path.clone(),
        initial_value,
        text.clone(),
    );
    inspector_state.update_field_draft_text(text)?;
    if let Some(value) = parsed_value {
        inspector_state.update_field_draft(value)?;
    }
    Ok(())
}

fn commit_inspector_component_field_edit(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    entity: editor_core::EntityId,
    component_type: editor_core::ComponentTypeId,
    path: editor_inspector::InspectorPath,
    value: InspectorEditValue,
) -> Result<(), EditorMutationError> {
    execute_intent_with_history_from_origin(
        app.runtime_mut(),
        "Edit Component Field",
        editor_scene::SceneCommandIntent::EditComponentField {
            entity,
            component_type,
            path,
            value,
        },
        editor_core::ChangeOrigin::InspectorPanel,
    )
    .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
    inspector_state.clear_draft();
    inspector_state.clear_focus();
    Ok(())
}

fn parse_inspector_field_text(
    field: &crate::editor_panels::InspectorWidgetField,
    text: &str,
) -> Option<InspectorEditValue> {
    match &field.value {
        InspectorValue::Bool(_) => {
            let normalized = text.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "true" | "1" => Some(InspectorEditValue::Bool(true)),
                "false" | "0" => Some(InspectorEditValue::Bool(false)),
                _ => None,
            }
        }
        InspectorValue::Integer(_) => text
            .trim()
            .parse::<i64>()
            .ok()
            .map(InspectorEditValue::Integer),
        InspectorValue::Float(_) => text
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .map(InspectorEditValue::Float),
        InspectorValue::Text(_) => Some(InspectorEditValue::Text(text.to_string())),
        InspectorValue::Enum { options, .. } => options
            .iter()
            .find(|option| option.as_str() == text.trim())
            .cloned()
            .map(InspectorEditValue::EnumSymbol),
        InspectorValue::ReadOnlyText(_)
        | InspectorValue::Group
        | InspectorValue::Unsupported { .. } => None,
    }
}

fn editable_value_from_field(
    field: &crate::editor_panels::InspectorWidgetField,
) -> Option<InspectorEditValue> {
    if let Some(value) = &field.draft_value {
        return Some(value.clone());
    }

    match &field.value {
        InspectorValue::Bool(value) => Some(InspectorEditValue::Bool(*value)),
        InspectorValue::Integer(value) => Some(InspectorEditValue::Integer(*value)),
        InspectorValue::Float(value) => Some(InspectorEditValue::Float(*value)),
        InspectorValue::Text(value) => Some(InspectorEditValue::Text(value.clone())),
        InspectorValue::Enum { current, options } => {
            if options.iter().any(|option| option == current) {
                Some(InspectorEditValue::EnumSymbol(current.clone()))
            } else {
                None
            }
        }
        InspectorValue::ReadOnlyText(_)
        | InspectorValue::Group
        | InspectorValue::Unsupported { .. } => None,
    }
}

fn inspector_value_text(value: &InspectorValue) -> String {
    match value {
        InspectorValue::Bool(value) => value.to_string(),
        InspectorValue::Integer(value) => value.to_string(),
        InspectorValue::Float(value) => value.to_string(),
        InspectorValue::Text(value) => value.clone(),
        InspectorValue::ReadOnlyText(value) => value.clone(),
        InspectorValue::Enum { current, .. } => current.clone(),
        InspectorValue::Group => "group".to_string(),
        InspectorValue::Unsupported { type_name } => format!("unsupported<{type_name}>"),
    }
}

fn next_shell_edit_value(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
    component_type: ComponentTypeId,
    field: &crate::editor_panels::InspectorWidgetField,
) -> Option<InspectorEditValue> {
    if is_local_transform_component(runtime, component_type)
        && let Some(stepper_value) = transform_stepper_value(field)
    {
        return Some(stepper_value);
    }

    if let Some(draft) = &field.draft_value {
        return match draft {
            InspectorEditValue::Bool(value) => Some(InspectorEditValue::Bool(!value)),
            InspectorEditValue::Integer(value) => {
                Some(InspectorEditValue::Integer(value.saturating_add(1)))
            }
            InspectorEditValue::Float(value) => Some(InspectorEditValue::Float(value + 1.0)),
            InspectorEditValue::Text(value) => Some(InspectorEditValue::Text(format!("{value}*"))),
            InspectorEditValue::EnumSymbol(value) => {
                next_enum_symbol(field, value).map(InspectorEditValue::EnumSymbol)
            }
        };
    }

    match &field.value {
        InspectorValue::Bool(value) => Some(InspectorEditValue::Bool(!value)),
        InspectorValue::Integer(value) => {
            Some(InspectorEditValue::Integer(value.saturating_add(1)))
        }
        InspectorValue::Float(value) => Some(InspectorEditValue::Float(value + 1.0)),
        InspectorValue::Text(value) => Some(InspectorEditValue::Text(format!("{value}*"))),
        InspectorValue::Enum { current, .. } => {
            next_enum_symbol(field, current).map(InspectorEditValue::EnumSymbol)
        }
        InspectorValue::ReadOnlyText(_)
        | InspectorValue::Group
        | InspectorValue::Unsupported { .. } => None,
    }
}

fn next_enum_symbol(
    field: &crate::editor_panels::InspectorWidgetField,
    current: &str,
) -> Option<String> {
    let InspectorValue::Enum { options, .. } = &field.value else {
        return None;
    };
    if options.is_empty() {
        return None;
    }
    let current_index = options
        .iter()
        .position(|option| option == current)
        .unwrap_or(options.len().saturating_sub(1));
    let next_index = (current_index + 1) % options.len();
    options.get(next_index).cloned()
}

fn transform_stepper_value(
    field: &crate::editor_panels::InspectorWidgetField,
) -> Option<InspectorEditValue> {
    let path = field.path.stable_key();
    if path != "translation.x" && path != "translation.y" && path != "translation.z" {
        return None;
    }

    let current = match field.draft_value.as_ref() {
        Some(InspectorEditValue::Float(value)) => *value,
        Some(_) => return None,
        None => match &field.value {
            InspectorValue::Float(value) => *value,
            _ => return None,
        },
    };

    Some(InspectorEditValue::Float(
        current + TRANSFORM_STEPPER_INCREMENT,
    ))
}
