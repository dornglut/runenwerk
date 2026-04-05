use crate::editor_panels::{
	InspectorPanelCommand, InspectorPanelCommandResult, InspectorPanelPresenter,
	InspectorPanelViewModel, OutlinerPanelCommand, OutlinerPanelCommandResult,
	OutlinerPanelPresenter, OutlinerPanelState, ViewportPanelCommand,
	ViewportPanelPresenter, ViewportPanelState, ViewportToolState, ViewportInteractionState,
};
use crate::editor_runtime::{
	clear_selection, execute_scene_command_and_push_history, select_single_component,
	select_single_entity, EditorInspectorUiState, EditorToolRuntimeState,
	RunenwerkEditorRuntime,
};
use editor_core::{ComponentTypeId, EntityId};
use editor_inspector::{InspectorEditValue, InspectorPath};
use scene::LocalTransform;

pub struct RunenwerkEditorApp {
	runtime: RunenwerkEditorRuntime,
	inspector_ui_state: EditorInspectorUiState,
	tool_runtime_state: EditorToolRuntimeState,
	viewport_interaction_state: ViewportInteractionState,
}

impl RunenwerkEditorApp {
	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: new
	pub fn new() -> Self {
		Self {
			runtime: RunenwerkEditorRuntime::new(),
			inspector_ui_state: EditorInspectorUiState::new(),
			tool_runtime_state: EditorToolRuntimeState::new(),
			viewport_interaction_state: ViewportInteractionState::new(),
		}
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: runtime
	pub fn runtime(&self) -> &RunenwerkEditorRuntime {
		&self.runtime
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: runtime_mut
	pub fn runtime_mut(&mut self) -> &mut RunenwerkEditorRuntime {
		&mut self.runtime
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: inspector_ui_state
	pub fn inspector_ui_state(&self) -> &EditorInspectorUiState {
		&self.inspector_ui_state
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: inspector_ui_state_mut
	pub fn inspector_ui_state_mut(&mut self) -> &mut EditorInspectorUiState {
		&mut self.inspector_ui_state
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: outliner_state
	pub fn outliner_state(&self) -> OutlinerPanelState {
		OutlinerPanelPresenter::build_state(&self.runtime)
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: inspector_view_model
	pub fn inspector_view_model(&self) -> InspectorPanelViewModel {
		InspectorPanelPresenter::build_view_model(&self.runtime, &self.inspector_ui_state)
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: dispatch_outliner_command
	pub fn dispatch_outliner_command(
		&mut self,
		command: OutlinerPanelCommand,
	) -> Result<OutlinerPanelCommandResult, &'static str> {
		OutlinerPanelPresenter::dispatch(&mut self.runtime, command)
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: dispatch_inspector_command
	pub fn dispatch_inspector_command(
		&mut self,
		command: InspectorPanelCommand,
	) -> Result<InspectorPanelCommandResult, &'static str> {
		InspectorPanelPresenter::dispatch(
			&mut self.runtime,
			&mut self.inspector_ui_state,
			command,
		)
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: viewport_state
	pub fn viewport_state(&self) -> ViewportPanelState {
		ViewportPanelPresenter::build_state(&self.runtime)
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: dispatch_viewport_command
	pub fn dispatch_viewport_command(
		&mut self,
		command: ViewportPanelCommand,
	) -> Result<ViewportPanelState, &'static str> {
		ViewportPanelPresenter::dispatch(&mut self.runtime, command)
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: tool_runtime_state
	pub fn tool_runtime_state(&self) -> &EditorToolRuntimeState {
		&self.tool_runtime_state
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: tool_runtime_state_mut
	pub fn tool_runtime_state_mut(&mut self) -> &mut EditorToolRuntimeState {
		&mut self.tool_runtime_state
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: update_translation_preview
	pub fn update_translation_preview(
		&mut self,
		delta: scene::Vec3Value,
	) -> Result<(), &'static str> {
		self.tool_runtime_state.update_translation_preview(delta)
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: dispatch_tool_action
	pub fn dispatch_tool_action(
		&mut self,
		action: editor_tools::ToolAction,
	) -> Result<(), &'static str> {
		match action {
			editor_tools::ToolAction::SelectSingle(target) => match target {
				editor_core::SelectionTarget::Entity(entity) => {
					select_single_entity(&mut self.runtime, entity)?;
				}
				editor_core::SelectionTarget::Component {
					entity,
					component_type,
				} => {
					select_single_component(&mut self.runtime, entity, component_type)?;
				}
				_ => return Err("unsupported selection target for tool action"),
			},
			editor_tools::ToolAction::ClearSelection => {
				clear_selection(&mut self.runtime);
			}
			editor_tools::ToolAction::Scene(intent) => {
				let command_id = self.runtime.allocate_command_id();
				let transaction_id = self.runtime.allocate_transaction_id();

				let _ = execute_scene_command_and_push_history(
					&mut self.runtime,
					editor_scene::scene_intent_to_command(command_id, intent),
					"Tool Scene Action",
					transaction_id,
				)?;
			}
			editor_tools::ToolAction::HoverEntity(entity) => {
				self.tool_runtime_state.set_hovered_entity(entity);
			}
			editor_tools::ToolAction::BeginPreview => {
				let selection = self
					.runtime
					.session()
					.selection()
					.primary()
					.cloned()
					.ok_or("cannot begin preview without a primary selection")?;

				self.tool_runtime_state
					.begin_preview(selection, crate::editor_runtime::TransformToolKind::Translate)?;
			}
			editor_tools::ToolAction::UpdatePreview => {
				self.tool_runtime_state.update_preview()?;
			}
			editor_tools::ToolAction::CommitPreview => {
				let preview = self
					.tool_runtime_state
					.commit_preview()
					.ok_or("no active preview session")?;

				Self::commit_translation_preview_into_local_transform(
					&mut self.runtime,
					preview.entity,
					preview.translation_delta,
				)?;
			}
			editor_tools::ToolAction::CancelPreview => {
				let _preview = self
					.tool_runtime_state
					.cancel_preview()
					.ok_or("no active preview session")?;
			}
		}

		Ok(())
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: dispatch_tool_actions
	pub fn dispatch_tool_actions(
		&mut self,
		actions: impl IntoIterator<Item = editor_tools::ToolAction>,
	) -> Result<(), &'static str> {
		for action in actions {
			self.dispatch_tool_action(action)?;
		}

		Ok(())
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: commit_translation_preview_into_local_transform
	fn commit_translation_preview_into_local_transform(
		runtime: &mut RunenwerkEditorRuntime,
		entity: EntityId,
		delta: scene::Vec3Value,
	) -> Result<(), &'static str> {
		let local_transform_type = Self::find_registered_component_type::<LocalTransform>(runtime)
			.ok_or("LocalTransform is not registered in editor runtime")?;

		let ecs_entity = runtime
			.ids()
			.resolve_entity(entity)
			.ok_or("editor entity is not registered")?;

		let current = runtime
			.world()
			.get::<LocalTransform>(ecs_entity)
			.ok_or("entity does not have LocalTransform")?;

		let next_x = current.translation.x + delta.x;
		let next_y = current.translation.y + delta.y;
		let next_z = current.translation.z + delta.z;

		let transaction_id = runtime.allocate_transaction_id();
		let command_x = runtime.allocate_command_id();
		let command_y = runtime.allocate_command_id();
		let command_z = runtime.allocate_command_id();

		let mut commands = vec![
			editor_scene::scene_intent_to_command(
				command_x,
				editor_scene::SceneCommandIntent::EditComponentField {
					entity,
					component_type: local_transform_type,
					path: InspectorPath::root()
						.child_field("translation")
						.child_field("x"),
					value: InspectorEditValue::Float(next_x as f64),
				},
			),
			editor_scene::scene_intent_to_command(
				command_y,
				editor_scene::SceneCommandIntent::EditComponentField {
					entity,
					component_type: local_transform_type,
					path: InspectorPath::root()
						.child_field("translation")
						.child_field("y"),
					value: InspectorEditValue::Float(next_y as f64),
				},
			),
			editor_scene::scene_intent_to_command(
				command_z,
				editor_scene::SceneCommandIntent::EditComponentField {
					entity,
					component_type: local_transform_type,
					path: InspectorPath::root()
						.child_field("translation")
						.child_field("z"),
					value: InspectorEditValue::Float(next_z as f64),
				},
			),
		];

		let _ = crate::editor_runtime::execute_scene_transaction_and_push_history(
			runtime,
			transaction_id,
			"Apply Translation Preview",
			&mut commands,
		)?;

		Ok(())
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: find_registered_component_type
	fn find_registered_component_type<T>(
		runtime: &RunenwerkEditorRuntime,
	) -> Option<ComponentTypeId>
	where
		T: 'static,
	{
		let target = std::any::TypeId::of::<T>();

		runtime
			.ids()
			.component_type_ids()
			.find(|component_type| {
				runtime
					.ids()
					.resolve_component_rust_type_id(*component_type)
					== Some(target)
			})
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: viewport_tool_state
	pub fn viewport_tool_state(&self) -> ViewportToolState {
		ViewportToolState::from_runtime(&self.tool_runtime_state)
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: viewport_interaction_state
	pub fn viewport_interaction_state(&self) -> &ViewportInteractionState {
		&self.viewport_interaction_state
	}

	/// File: apps/runenwerk_editor/src/editor_app.rs
	/// Method: viewport_interaction_state_mut
	pub fn viewport_interaction_state_mut(&mut self) -> &mut ViewportInteractionState {
		&mut self.viewport_interaction_state
	}
}