use std::collections::BTreeMap;

use editor_core::ToolId;
use editor_definition::{
    EditorMenuDefinition, EditorMenuItemDefinition, EditorToolbarBinding,
    EditorToolbarMenuItemBinding,
};
use editor_shell::{
    EDITOR_DESIGN_WORKSPACE_PROFILE_ID, MODELLING_WORKSPACE_PROFILE_ID, ObservationConsumerKind,
    ObservationFrameMetadata, ObservationSourceReality, SCENE_WORKSPACE_PROFILE_ID,
    ToolbarButtonViewModel, ToolbarMenuKind, ToolbarObservationFrame, ToolbarObservedButton,
    ToolbarViewModel, WorkspaceProfileId,
};
use ui_definition::{UiAvailabilityBinding, UiAvailabilityId, UiRouteSlotId};

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
const WORKSPACE_EDITOR_DESIGN_ID: ToolId = ToolId(3_004);

#[expect(
    clippy::too_many_arguments,
    reason = "toolbar observation projection takes explicit shell state inputs"
)]
pub fn build_toolbar_observation_frame(
    _active_tool: Option<ToolId>,
    can_undo: bool,
    can_redo: bool,
    _debug_logs_enabled: bool,
    active_toolbar_menu: Option<ToolbarMenuKind>,
    active_workspace_profile_id: WorkspaceProfileId,
    open_workspace_profile_ids: &[WorkspaceProfileId],
    source_version: editor_core::RealityVersion,
    active_menus: &BTreeMap<String, EditorMenuDefinition>,
) -> ToolbarObservationFrame {
    let mut buttons = vec![
        ToolbarObservedButton {
            id: MENU_FILE_ID,
            stable_name: "menu_file",
            label: active_menu_label(active_menus, "file", "File"),
            is_active: active_toolbar_menu == Some(ToolbarMenuKind::File),
            enabled: true,
        },
        ToolbarObservedButton {
            id: MENU_EDIT_ID,
            stable_name: "menu_edit",
            label: active_menu_label(active_menus, "edit", "Edit"),
            is_active: active_toolbar_menu == Some(ToolbarMenuKind::Edit),
            enabled: true,
        },
        ToolbarObservedButton {
            id: MENU_WINDOW_ID,
            stable_name: "menu_window",
            label: active_menu_label(active_menus, "window", "Window"),
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
    ];
    for profile_id in open_workspace_profile_ids {
        if let Some(button) = workspace_button_for_profile(*profile_id, active_workspace_profile_id)
        {
            buttons.push(button);
        }
    }
    buttons.push(ToolbarObservedButton {
        id: WORKSPACE_PLUS_ID,
        stable_name: "workspace_plus",
        label: "+".to_string(),
        is_active: active_toolbar_menu == Some(ToolbarMenuKind::Workspace),
        enabled: true,
    });
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

pub fn toolbar_binding_with_active_menus(
    binding: Option<EditorToolbarBinding>,
    active_menus: &BTreeMap<String, EditorMenuDefinition>,
) -> Option<EditorToolbarBinding> {
    let mut binding = binding?;
    let active_toolbar_menus = ["file", "edit", "window", "workspace"];
    if !active_toolbar_menus
        .iter()
        .any(|menu_id| active_menus.contains_key(*menu_id))
    {
        return Some(binding);
    }
    binding
        .menu_items
        .retain(|item| !active_toolbar_menus.contains(&item.menu_id.as_str()));
    for menu_id in active_toolbar_menus {
        let Some(menu) = active_menus.get(menu_id) else {
            continue;
        };
        extend_toolbar_menu_items_from_active_menu(menu_id, &menu.items, &mut binding.menu_items);
    }
    Some(binding)
}

fn active_menu_label(
    active_menus: &BTreeMap<String, EditorMenuDefinition>,
    menu_id: &str,
    fallback: &str,
) -> String {
    active_menus
        .get(menu_id)
        .map(|menu| menu.label.clone())
        .unwrap_or_else(|| fallback.to_string())
}

fn extend_toolbar_menu_items_from_active_menu(
    menu_id: &str,
    items: &[EditorMenuItemDefinition],
    output: &mut Vec<EditorToolbarMenuItemBinding>,
) {
    for item in items {
        if let Some(command) = item.command.as_deref() {
            output.push(EditorToolbarMenuItemBinding {
                menu_id: menu_id.to_string(),
                item_id: item.id.clone(),
                label: item.label.clone(),
                route: UiRouteSlotId::new(command),
                availability: item
                    .availability
                    .as_deref()
                    .map(|id| UiAvailabilityBinding::Ref(UiAvailabilityId::new(id))),
            });
        }
        extend_toolbar_menu_items_from_active_menu(menu_id, &item.children, output);
    }
}

fn workspace_button_for_profile(
    profile_id: WorkspaceProfileId,
    active_workspace_profile_id: WorkspaceProfileId,
) -> Option<ToolbarObservedButton> {
    let (id, stable_name, label) = if profile_id == SCENE_WORKSPACE_PROFILE_ID {
        (WORKSPACE_SCENE_ID, "workspace_scene", "Scene")
    } else if profile_id == MODELLING_WORKSPACE_PROFILE_ID {
        (WORKSPACE_MODELLING_ID, "workspace_modelling", "Modelling")
    } else if profile_id == EDITOR_DESIGN_WORKSPACE_PROFILE_ID {
        (
            WORKSPACE_EDITOR_DESIGN_ID,
            "workspace_editor_design",
            "Editor Design",
        )
    } else {
        return None;
    };
    Some(ToolbarObservedButton {
        id,
        stable_name,
        label: label.to_string(),
        is_active: active_workspace_profile_id == profile_id,
        enabled: true,
    })
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
            menu_item(2_307, "window_load_custom_label", "Load: Custom", false),
            menu_item(2_308, "window_load_custom", "No Custom Workspaces", false),
        ],
        Some(ToolbarMenuKind::Workspace) => vec![
            menu_item(2_400, "workspace_menu_scene", "Scene", true),
            menu_item(2_401, "workspace_menu_modelling", "Modelling", true),
            menu_item(2_402, "workspace_menu_editor_design", "Editor Design", true),
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
