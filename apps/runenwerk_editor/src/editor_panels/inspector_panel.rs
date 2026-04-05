use editor_core::{ComponentTypeId, EntityId, SelectionTarget};
use editor_inspector::{
	InspectTarget, InspectorEditValue, InspectorField, InspectorPath, InspectorSection,
};
use editor_scene::SceneComponentDescriptor;

use crate::editor_panels::{build_widget_fields, InspectorWidgetField};
use crate::editor_runtime::{
	build_component_inspector_sections, execute_scene_command_and_push_history,
	select_single_component, EditorInspectorUiState, InspectorFieldDraft,
	RunenwerkEditorRuntime,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorComponentItem {
	pub entity: EntityId,
	pub component_type: ComponentTypeId,
	pub display_name: String,
	pub is_selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorAvailableComponentItem {
	pub component_type: ComponentTypeId,
	pub display_name: String,
	pub already_attached: bool,
}

impl InspectorComponentItem {
	/// File: apps/runenwerk_editor/src/editor_panels/inspector_panel.rs
	/// Method: from_descriptor
	pub fn from_descriptor(
		descriptor: SceneComponentDescriptor,
		is_selected: bool,
	) -> Self {
		Self {
			entity: descriptor.entity,
			component_type: descriptor.component_type,
			display_name: descriptor.display_name,
			is_selected,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorSectionView {
	pub section: InspectorSection,
	pub expanded: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorPanelViewModel {
	Empty,
	Entity {
		entity: EntityId,
		display_name: String,
		components: Vec<InspectorComponentItem>,
		available_component_types: Vec<InspectorAvailableComponentItem>,
	},
	Component {
		entity: EntityId,
		entity_display_name: String,
		component_type: ComponentTypeId,
		component_display_name: String,
		components: Vec<InspectorComponentItem>,
		sections: Vec<InspectorSectionView>,
		widget_fields: Vec<InspectorWidgetField>,
		active_draft: Option<InspectorFieldDraft>,
		focused_field: Option<InspectorPath>,
		can_remove_component: bool,
	},
	Resource {
		resource_type: editor_core::ResourceTypeId,
	},
	Unsupported {
		target: String,
	},
	Error {
		message: String,
	},
}

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorPanelCommand {
	SelectComponent {
		entity: EntityId,
		component_type: ComponentTypeId,
	},
	AddComponentToEntity {
		entity: EntityId,
		component_type: ComponentTypeId,
	},
	RemoveComponentFromEntity {
		entity: EntityId,
		component_type: ComponentTypeId,
	},
	EditComponentField {
		entity: EntityId,
		component_type: ComponentTypeId,
		path: InspectorPath,
		value: InspectorEditValue,
	},
	BeginEditComponentField {
		entity: EntityId,
		component_type: ComponentTypeId,
		path: InspectorPath,
		value: InspectorEditValue,
	},
	UpdateDraftComponentField {
		value: InspectorEditValue,
	},
	CommitDraftComponentField,
	CancelDraftComponentField,
	ToggleSectionExpanded {
		key: String,
	},
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorPanelCommandResult {
	pub view_model: InspectorPanelViewModel,
}

pub struct InspectorPanelPresenter;

impl InspectorPanelPresenter {
	/// File: apps/runenwerk_editor/src/editor_panels/inspector_panel.rs
	/// Method: build_view_model
	pub fn build_view_model(
		runtime: &RunenwerkEditorRuntime,
		ui_state: &EditorInspectorUiState,
	) -> InspectorPanelViewModel {
		match runtime.primary_inspect_target() {
			Some(InspectTarget::Entity(entity)) => {
				let display_name = runtime
					.ids()
					.entity_display_name(entity)
					.unwrap_or("Entity")
					.to_string();

				let components = runtime
					.list_entity_components(entity)
					.into_iter()
					.map(|descriptor| InspectorComponentItem::from_descriptor(descriptor, false))
					.collect::<Vec<_>>();

				let available_component_types = runtime
					.list_registered_component_types()
					.into_iter()
					.map(|(component_type, display_name)| InspectorAvailableComponentItem {
						already_attached: runtime.entity_has_component(entity, component_type),
						component_type,
						display_name,
					})
					.collect();

				InspectorPanelViewModel::Entity {
					entity,
					display_name,
					components,
					available_component_types,
				}
			}
			Some(InspectTarget::Component {
				     entity,
				     component_type,
			     }) => {
				let entity_display_name = runtime
					.ids()
					.entity_display_name(entity)
					.unwrap_or("Entity")
					.to_string();

				let component_display_name = runtime
					.ids()
					.component_display_name(component_type)
					.unwrap_or("Component")
					.to_string();

				let components = runtime
					.list_entity_components(entity)
					.into_iter()
					.map(|descriptor| {
						let is_selected = descriptor.component_type == component_type;
						InspectorComponentItem::from_descriptor(descriptor, is_selected)
					})
					.collect::<Vec<_>>();

				let sections = match build_component_inspector_sections(runtime, entity, component_type) {
					Ok(sections) => sections
						.into_iter()
						.enumerate()
						.map(|(index, section)| {
							let key = inspector_section_key(entity, component_type, index, &section);
							let expanded = ui_state.is_expanded(&key);
							InspectorSectionView { section, expanded }
						})
						.collect::<Vec<_>>(),
					Err(error) => {
						return InspectorPanelViewModel::Error {
							message: format!("failed to build component inspector sections: {error:?}"),
						};
					}
				};

				let active_draft = ui_state.active_draft().cloned();
				let focused_field = ui_state.focused_field().cloned();

				let raw_sections = sections
					.iter()
					.map(|section_view| section_view.section.clone())
					.collect::<Vec<_>>();

				let widget_fields = build_widget_fields(
					&raw_sections,
					active_draft.as_ref(),
					focused_field.as_ref(),
				);

				InspectorPanelViewModel::Component {
					entity,
					entity_display_name,
					component_type,
					component_display_name,
					components,
					sections,
					widget_fields,
					active_draft,
					focused_field,
					can_remove_component: true,
				}
			}
			Some(InspectTarget::Resource(resource_type)) => {
				InspectorPanelViewModel::Resource { resource_type }
			}
			Some(InspectTarget::Document(_)) => InspectorPanelViewModel::Unsupported {
				target: "document".to_string(),
			},
			Some(InspectTarget::Asset(_)) => InspectorPanelViewModel::Unsupported {
				target: "asset".to_string(),
			},
			Some(InspectTarget::Query { .. }) => InspectorPanelViewModel::Unsupported {
				target: "query".to_string(),
			},
			Some(InspectTarget::Custom { .. }) => InspectorPanelViewModel::Unsupported {
				target: "custom".to_string(),
			},
			None => InspectorPanelViewModel::Empty,
		}
	}

	/// File: apps/runenwerk_editor/src/editor_panels/inspector_panel.rs
	/// Method: dispatch
	pub fn dispatch(
		runtime: &mut RunenwerkEditorRuntime,
		ui_state: &mut EditorInspectorUiState,
		command: InspectorPanelCommand,
	) -> Result<InspectorPanelCommandResult, &'static str> {
		match command {
			InspectorPanelCommand::SelectComponent {
				entity,
				component_type,
			} => {
				select_single_component(runtime, entity, component_type)?;
				ui_state.clear_draft();
				ui_state.clear_focus();
			}
			InspectorPanelCommand::AddComponentToEntity {
				entity,
				component_type,
			} => {
				let command_id = runtime.allocate_command_id();
				let transaction_id = runtime.allocate_transaction_id();

				let _ = execute_scene_command_and_push_history(
					runtime,
					editor_scene::scene_intent_to_command(
						command_id,
						editor_scene::SceneCommandIntent::AddComponent {
							entity,
							component_type,
						},
					),
					"Add Component",
					transaction_id,
				)?;

				ui_state.clear_draft();
				ui_state.clear_focus();
			}
			InspectorPanelCommand::RemoveComponentFromEntity {
				entity,
				component_type,
			} => {
				let command_id = runtime.allocate_command_id();
				let transaction_id = runtime.allocate_transaction_id();

				let _ = execute_scene_command_and_push_history(
					runtime,
					editor_scene::scene_intent_to_command(
						command_id,
						editor_scene::SceneCommandIntent::RemoveComponent {
							entity,
							component_type,
						},
					),
					"Remove Component",
					transaction_id,
				)?;

				ui_state.clear_draft();
				ui_state.clear_focus();
			}
			InspectorPanelCommand::EditComponentField {
				entity,
				component_type,
				path,
				value,
			} => {
				commit_component_field_edit(
					runtime,
					entity,
					component_type,
					path,
					value,
				)?;
				ui_state.clear_draft();
				ui_state.clear_focus();
			}
			InspectorPanelCommand::BeginEditComponentField {
				entity,
				component_type,
				path,
				value,
			} => {
				ui_state.begin_field_edit(entity, component_type, path, value);
			}
			InspectorPanelCommand::UpdateDraftComponentField { value } => {
				ui_state.update_field_draft(value)?;
			}
			InspectorPanelCommand::CommitDraftComponentField => {
				let draft = ui_state
					.take_active_draft()
					.ok_or("no active inspector field draft")?;

				commit_component_field_edit(
					runtime,
					draft.entity,
					draft.component_type,
					draft.path,
					draft.value,
				)?;
			}
			InspectorPanelCommand::CancelDraftComponentField => {
				ui_state.cancel_field_draft();
			}
			InspectorPanelCommand::ToggleSectionExpanded { key } => {
				ui_state.toggle_expanded(key);
			}
		}

		Ok(InspectorPanelCommandResult {
			view_model: Self::build_view_model(runtime, ui_state),
		})
	}
}

/// File: apps/runenwerk_editor/src/editor_panels/inspector_panel.rs
/// Method: flatten_editable_fields
pub fn flatten_editable_fields(
	sections: &[InspectorSection],
) -> Vec<InspectorEditableField> {
	let mut fields = Vec::new();

	for section in sections {
		for field in &section.fields {
			collect_editable_fields(field, &mut fields);
		}
	}

	fields
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorEditableField {
	pub display_name: String,
	pub path: InspectorPath,
	pub value: editor_inspector::InspectorValue,
}

fn collect_editable_fields(
	field: &InspectorField,
	out: &mut Vec<InspectorEditableField>,
) {
	if field.children.is_empty() && is_supported_edit_value(&field.value) {
		out.push(InspectorEditableField {
			display_name: field.display_name.clone(),
			path: field.path.clone(),
			value: field.value.clone(),
		});
		return;
	}

	for child in &field.children {
		collect_editable_fields(child, out);
	}
}

fn is_supported_edit_value(
	value: &editor_inspector::InspectorValue,
) -> bool {
	matches!(
		value,
		editor_inspector::InspectorValue::Bool(_)
			| editor_inspector::InspectorValue::Integer(_)
			| editor_inspector::InspectorValue::Float(_)
			| editor_inspector::InspectorValue::Text(_)
	)
}

/// File: apps/runenwerk_editor/src/editor_panels/inspector_panel.rs
/// Method: commit_component_field_edit
fn commit_component_field_edit(
	runtime: &mut RunenwerkEditorRuntime,
	entity: EntityId,
	component_type: ComponentTypeId,
	path: InspectorPath,
	value: InspectorEditValue,
) -> Result<(), &'static str> {
	let command_id = runtime.allocate_command_id();
	let transaction_id = runtime.allocate_transaction_id();

	let result = execute_scene_command_and_push_history(
		runtime,
		editor_scene::scene_intent_to_command(
			command_id,
			editor_scene::SceneCommandIntent::EditComponentField {
				entity,
				component_type,
				path,
				value,
			},
		),
		"Edit Component Field",
		transaction_id,
	)?;

	if result.is_none() {
		return Ok(());
	}

	Ok(())
}

fn _is_component_selected(
	runtime: &RunenwerkEditorRuntime,
	entity: EntityId,
	component_type: ComponentTypeId,
) -> bool {
	matches!(
		runtime.session().selection().primary(),
		Some(SelectionTarget::Component {
			entity: selected_entity,
			component_type: selected_component_type,
		}) if *selected_entity == entity && *selected_component_type == component_type
	)
}

/// File: apps/runenwerk_editor/src/editor_panels/inspector_panel.rs
/// Method: inspector_section_key
fn inspector_section_key(
	entity: EntityId,
	component_type: ComponentTypeId,
	index: usize,
	section: &InspectorSection,
) -> String {
	format!(
		"entity:{}:component:{}:section:{}:{}",
		entity.0,
		component_type.0,
		index,
		section.display_name
	)
}