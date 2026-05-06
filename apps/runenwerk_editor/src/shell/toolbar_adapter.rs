use editor_core::ToolId;
use editor_shell::{
    MODELLING_WORKSPACE_PROFILE_ID, ObservationConsumerKind, ObservationFrameMetadata,
    ObservationSourceReality, SCENE_WORKSPACE_PROFILE_ID, ToolbarButtonViewModel, ToolbarMenuKind,
    ToolbarObservationFrame, ToolbarObservedButton, ToolbarViewModel, WorkspaceProfileId,
};

pub const SELECT_TOOL_ID: ToolId = ToolId(1);
pub const TRANSLATE_TOOL_ID: ToolId = ToolId(2);
pub const ROTATE_TOOL_ID: ToolId = ToolId(3);
pub const SCALE_TOOL_ID: ToolId = ToolId(4);
pub const TOOLBAR_UNDO_ID: ToolId = ToolId(1001);
pub const TOOLBAR_REDO_ID: ToolId = ToolId(1002);
pub const TOOLBAR_SAVE_ID: ToolId = ToolId(1003);
pub const TOOLBAR_LOAD_ID: ToolId = ToolId(1004);
pub const TOOLBAR_DEBUG_LOGS_ID: ToolId = ToolId(1005);
const MENU_FILE_ID: ToolId = ToolId(2_001);
const MENU_EDIT_ID: ToolId = ToolId(2_002);
const MENU_WINDOW_ID: ToolId = ToolId(2_003);
const WORKSPACE_SCENE_ID: ToolId = ToolId(3_001);
const WORKSPACE_MODELLING_ID: ToolId = ToolId(3_002);
const WORKSPACE_PLUS_ID: ToolId = ToolId(3_003);

pub fn build_toolbar_observation_frame(
    _active_tool: Option<ToolId>,
    can_undo: bool,
    can_redo: bool,
    _debug_logs_enabled: bool,
    active_toolbar_menu: Option<ToolbarMenuKind>,
    active_workspace_profile_id: WorkspaceProfileId,
    source_version: editor_core::RealityVersion,
) -> ToolbarObservationFrame {
    let mut buttons = vec![
        ToolbarObservedButton {
            id: MENU_FILE_ID,
            stable_name: "menu_file",
            label: "File".to_string(),
            is_active: active_toolbar_menu == Some(ToolbarMenuKind::File),
            enabled: true,
        },
        ToolbarObservedButton {
            id: MENU_EDIT_ID,
            stable_name: "menu_edit",
            label: "Edit".to_string(),
            is_active: active_toolbar_menu == Some(ToolbarMenuKind::Edit),
            enabled: true,
        },
        ToolbarObservedButton {
            id: MENU_WINDOW_ID,
            stable_name: "menu_window",
            label: "Window".to_string(),
            is_active: active_toolbar_menu == Some(ToolbarMenuKind::Window),
            enabled: true,
        },
        ToolbarObservedButton {
            id: ToolId(2_004),
            stable_name: "separator",
            label: "|".to_string(),
            is_active: false,
            enabled: false,
        },
        ToolbarObservedButton {
            id: WORKSPACE_SCENE_ID,
            stable_name: "workspace_scene",
            label: "Scene".to_string(),
            is_active: active_workspace_profile_id == SCENE_WORKSPACE_PROFILE_ID,
            enabled: true,
        },
        ToolbarObservedButton {
            id: WORKSPACE_MODELLING_ID,
            stable_name: "workspace_modelling",
            label: "Modelling".to_string(),
            is_active: active_workspace_profile_id == MODELLING_WORKSPACE_PROFILE_ID,
            enabled: true,
        },
        ToolbarObservedButton {
            id: WORKSPACE_PLUS_ID,
            stable_name: "workspace_plus",
            label: "+".to_string(),
            is_active: false,
            enabled: false,
        },
    ];
    buttons.extend(toolbar_menu_items(active_toolbar_menu, can_undo, can_redo));

    ToolbarObservationFrame {
        metadata: ObservationFrameMetadata::strict_current(
            ObservationSourceReality::ObservedScene,
            ObservationConsumerKind::Toolbar,
            source_version,
        ),
        buttons,
    }
}

fn toolbar_menu_items(
    active_toolbar_menu: Option<ToolbarMenuKind>,
    can_undo: bool,
    can_redo: bool,
) -> Vec<ToolbarObservedButton> {
    match active_toolbar_menu {
        Some(ToolbarMenuKind::File) => vec![
            menu_item(2_100, "file_save", "Save", true),
            menu_item(2_101, "file_save_as", "Save As", false),
            menu_item(2_102, "file_open", "Open", true),
            menu_item(2_103, "file_open_recent", "Open Recent", false),
        ],
        Some(ToolbarMenuKind::Edit) => vec![
            menu_item(2_200, "edit_undo", "Undo", can_undo),
            menu_item(2_201, "edit_redo", "Redo", can_redo),
            menu_item(2_202, "edit_preferences", "Preferences", false),
        ],
        Some(ToolbarMenuKind::Window) => vec![
            menu_item(2_300, "window_new_window", "New Window", false),
            menu_item(2_301, "window_next_workspace", "Next Workspace", true),
            menu_item(
                2_302,
                "window_previous_workspace",
                "Previous Workspace",
                true,
            ),
            menu_item(2_303, "window_save_workspace", "Save Workspace", true),
            menu_item(2_304, "window_load_general_label", "Load: General", false),
            menu_item(2_305, "window_load_general_scene", "Scene", true),
            menu_item(2_306, "window_load_general_modelling", "Modelling", true),
            menu_item(2_307, "window_load_custom_label", "Load: Custom", false),
            menu_item(2_308, "window_load_custom", "No Custom Workspaces", false),
        ],
        None => Vec::new(),
    }
}

fn menu_item(
    id: u64,
    stable_name: &'static str,
    label: &'static str,
    enabled: bool,
) -> ToolbarObservedButton {
    ToolbarObservedButton {
        id: ToolId(id),
        stable_name,
        label: label.to_string(),
        is_active: false,
        enabled,
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
