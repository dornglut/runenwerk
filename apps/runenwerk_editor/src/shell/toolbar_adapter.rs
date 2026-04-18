use editor_core::ToolId;
use editor_shell::{
    ObservationConsumerKind, ObservationFrameMetadata, ObservationSourceReality,
    ToolbarButtonViewModel, ToolbarObservationFrame, ToolbarObservedButton, ToolbarViewModel,
};

pub const SELECT_TOOL_ID: ToolId = ToolId(1);
pub const TRANSLATE_TOOL_ID: ToolId = ToolId(2);
pub const TOOLBAR_UNDO_ID: ToolId = ToolId(1001);
pub const TOOLBAR_REDO_ID: ToolId = ToolId(1002);
pub const TOOLBAR_SAVE_ID: ToolId = ToolId(1003);
pub const TOOLBAR_LOAD_ID: ToolId = ToolId(1004);
pub const TOOLBAR_DEBUG_LOGS_ID: ToolId = ToolId(1005);

pub fn build_toolbar_observation_frame(
    active_tool: Option<ToolId>,
    can_undo: bool,
    can_redo: bool,
    debug_logs_enabled: bool,
    source_version: editor_core::RealityVersion,
) -> ToolbarObservationFrame {
    ToolbarObservationFrame {
        metadata: ObservationFrameMetadata::strict_current(
            ObservationSourceReality::ObservedScene,
            ObservationConsumerKind::Toolbar,
            source_version,
        ),
        buttons: vec![
            ToolbarObservedButton {
                id: SELECT_TOOL_ID,
                stable_name: "select",
                label: "Select".to_string(),
                is_active: active_tool == Some(SELECT_TOOL_ID),
                enabled: true,
            },
            ToolbarObservedButton {
                id: TRANSLATE_TOOL_ID,
                stable_name: "translate",
                label: "Translate".to_string(),
                is_active: active_tool == Some(TRANSLATE_TOOL_ID),
                enabled: true,
            },
            ToolbarObservedButton {
                id: TOOLBAR_UNDO_ID,
                stable_name: "undo",
                label: "Undo".to_string(),
                is_active: false,
                enabled: can_undo,
            },
            ToolbarObservedButton {
                id: TOOLBAR_REDO_ID,
                stable_name: "redo",
                label: "Redo".to_string(),
                is_active: false,
                enabled: can_redo,
            },
            ToolbarObservedButton {
                id: TOOLBAR_SAVE_ID,
                stable_name: "save",
                label: "Save".to_string(),
                is_active: false,
                enabled: true,
            },
            ToolbarObservedButton {
                id: TOOLBAR_LOAD_ID,
                stable_name: "load",
                label: "Load".to_string(),
                is_active: false,
                enabled: true,
            },
            ToolbarObservedButton {
                id: TOOLBAR_DEBUG_LOGS_ID,
                stable_name: "debug_logs",
                label: if debug_logs_enabled {
                    "Logs On".to_string()
                } else {
                    "Logs Off".to_string()
                },
                is_active: debug_logs_enabled,
                enabled: true,
            },
        ],
    }
}

pub fn build_toolbar_view_model(frame: &ToolbarObservationFrame) -> ToolbarViewModel {
    ToolbarViewModel {
        buttons: frame
            .buttons
            .iter()
            .map(|button| ToolbarButtonViewModel {
                id: button.id,
                stable_name: button.stable_name,
                label: button.label.clone(),
                is_active: button.is_active,
                enabled: button.enabled,
            })
            .collect(),
    }
}
