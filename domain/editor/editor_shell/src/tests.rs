use crate::{UiInteraction, UiInteractionResults, UiRuntime};
use ui_math::UiRect;
use ui_text::{FontAtlasSource, FontFaceMetrics, FontId, GlyphMetrics, MsdfFontAtlas};
use ui_theme::ThemeTokens;

use crate::{
    ConsoleViewModel, EditorShellViewModel, InspectorFieldViewModel, InspectorTargetViewModel,
    InspectorViewModel, OutlinerRowViewModel, OutlinerViewModel, ShellCommand,
    ToolbarButtonViewModel, ToolbarViewModel, ViewportViewModel, build_editor_shell,
    map_interactions_to_shell_commands,
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
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2),
                    stable_name: "translate",
                    label: "Translate".to_string(),
                    is_active: false,
                    enabled: true,
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
        console: ConsoleViewModel {
            lines: vec!["boot".to_string()],
        },
    };

    let tree = build_editor_shell(&shell, &theme);
    let runtime = UiRuntime::new();
    let atlas_source = TestAtlasSource::ascii();
    let frame = runtime.build_frame(&tree, UiRect::new(0.0, 0.0, 1600.0, 900.0), &atlas_source);

    assert_eq!(tree.root_id().0, 1);
    assert_eq!(frame.surfaces.len(), 1);
    assert!(!frame.surfaces[0].layers[0].primitives.is_empty());
}

struct TestAtlasSource {
    atlas: MsdfFontAtlas,
}

impl TestAtlasSource {
    fn ascii() -> Self {
        let mut glyphs = std::collections::HashMap::new();
        for byte in 32_u8..=126_u8 {
            glyphs.insert(
                byte as char,
                GlyphMetrics {
                    advance: 10.0,
                    plane_left: 0.0,
                    plane_top: 8.0,
                    plane_right: 8.0,
                    plane_bottom: -2.0,
                    atlas_left: 0.0,
                    atlas_top: 0.0,
                    atlas_right: 0.1,
                    atlas_bottom: 0.1,
                },
            );
        }

        Self {
            atlas: MsdfFontAtlas {
                font_id: FontId(1),
                texture_width: 256,
                texture_height: 256,
                metrics: FontFaceMetrics {
                    ascender: 9.0,
                    descender: -3.0,
                    line_height: 12.0,
                    base_size: 12.0,
                },
                glyphs,
            },
        }
    }
}

impl FontAtlasSource for TestAtlasSource {
    fn atlas(&self, font_id: FontId) -> Option<&MsdfFontAtlas> {
        if font_id == self.atlas.font_id {
            Some(&self.atlas)
        } else {
            None
        }
    }
}

#[test]
fn toolbar_activation_maps_to_shell_command() {
    let interactions = UiInteractionResults {
        items: vec![UiInteraction::Activated(
            crate::TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
        )],
    };
    let view_model = EditorShellViewModel {
        toolbar: ToolbarViewModel::default(),
        outliner: OutlinerViewModel::default(),
        viewport: ViewportViewModel::default(),
        inspector: InspectorViewModel::default(),
        console: ConsoleViewModel::default(),
    };

    let commands = map_interactions_to_shell_commands(&interactions, &view_model);

    assert_eq!(commands, vec![ShellCommand::ActivateTranslateTool]);
}

#[test]
fn outliner_row_activation_maps_to_select_entity_command() {
    let interactions = UiInteractionResults {
        items: vec![UiInteraction::Activated(crate::outliner_row_widget_id(0))],
    };
    let view_model = EditorShellViewModel {
        toolbar: ToolbarViewModel::default(),
        outliner: OutlinerViewModel {
            rows: vec![OutlinerRowViewModel {
                entity: editor_core::EntityId(42),
                display_name: "Root".to_string(),
                depth: 0,
                is_selected: false,
            }],
        },
        viewport: ViewportViewModel::default(),
        inspector: InspectorViewModel::default(),
        console: ConsoleViewModel::default(),
    };

    let commands = map_interactions_to_shell_commands(&interactions, &view_model);

    assert_eq!(
        commands,
        vec![ShellCommand::SelectOutlinerEntity {
            entity: editor_core::EntityId(42),
        }]
    );
}

#[test]
fn inspector_field_activation_maps_to_shell_edit_command() {
    let interactions = UiInteractionResults {
        items: vec![UiInteraction::Activated(crate::inspector_field_widget_id(
            3,
        ))],
    };
    let view_model = EditorShellViewModel {
        toolbar: ToolbarViewModel::default(),
        outliner: OutlinerViewModel::default(),
        viewport: ViewportViewModel::default(),
        inspector: InspectorViewModel {
            target: InspectorTargetViewModel::Entity {
                display_name: "Entity".to_string(),
            },
            fields: vec![],
        },
        console: ConsoleViewModel::default(),
    };

    let commands = map_interactions_to_shell_commands(&interactions, &view_model);

    assert_eq!(
        commands,
        vec![ShellCommand::ActivateInspectorField { index: 3 }]
    );
}
