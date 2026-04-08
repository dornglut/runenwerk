use ui_math::UiRect;
use ui_runtime::{UiInteraction, UiInteractionResults, UiRuntime};
use ui_theme::ThemeTokens;

use crate::{
	build_editor_shell, map_interactions_to_shell_commands, EditorShellViewModel,
	InspectorFieldViewModel, InspectorTargetViewModel, InspectorViewModel, OutlinerRowViewModel,
	OutlinerViewModel, ShellCommand, ToolbarButtonViewModel, ToolbarViewModel, ViewportViewModel,
};

#[test]
fn shell_view_model_builds_ui_tree_and_frame() {
	let theme = ThemeTokens::default();

	let shell = EditorShellViewModel {
		toolbar: ToolbarViewModel {
			buttons: vec![
				ToolbarButtonViewModel {
					id: editor_core::ToolId(1),
					stable_name: "select",
					label: "Select".to_string(),
					is_active: true,
				},
				ToolbarButtonViewModel {
					id: editor_core::ToolId(2),
					stable_name: "translate",
					label: "Translate".to_string(),
					is_active: false,
				},
			],
		},
		outliner: OutlinerViewModel {
			rows: vec![OutlinerRowViewModel {
				entity: editor_core::EntityId(1),
				display_name: "Player".to_string(),
				depth: 0,
				is_selected: true,
			}],
		},
		viewport: ViewportViewModel {
			selected_entity: Some(editor_core::EntityId(1)),
			hovered_entity: None,
			drag_in_progress: false,
			preview_active: false,
		},
		inspector: InspectorViewModel {
			target: InspectorTargetViewModel::Component {
				entity_display_name: "Player".to_string(),
				component_display_name: "LocalTransform".to_string(),
			},
			fields: vec![InspectorFieldViewModel {
				label: "translation.x".to_string(),
				value_summary: "1.0".to_string(),
				is_focused: false,
			}],
		},
	};

	let tree = build_editor_shell(&shell, &theme);
	let runtime = UiRuntime::new();
	let frame = runtime.build_frame(&tree, UiRect::new(0.0, 0.0, 1600.0, 900.0));

	assert_eq!(tree.root_id().0, 1);
	assert_eq!(frame.surfaces.len(), 1);
	assert!(!frame.surfaces[0].layers[0].primitives.is_empty());
}

#[test]
fn toolbar_activation_maps_to_shell_command() {
	let interactions = UiInteractionResults {
		items: vec![UiInteraction::Activated(crate::TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID)],
	};

	let commands = map_interactions_to_shell_commands(&interactions);

	assert_eq!(commands, vec![ShellCommand::ActivateTranslateTool]);
}