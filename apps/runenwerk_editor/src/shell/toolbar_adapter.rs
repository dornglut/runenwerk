use std::collections::BTreeMap;

use editor_core::ToolId;
use editor_definition::{
    EditorMenuDefinition, EditorMenuItemDefinition, EditorToolbarBinding,
    EditorToolbarMenuItemBinding,
};
use editor_shell::{
    ObservationConsumerKind, ObservationFrameMetadata, ObservationSourceReality,
    ToolbarButtonViewModel, ToolbarMenuKind, ToolbarObservationFrame, ToolbarObservedButton,
    ToolbarViewModel, ToolbarWorkspaceButtonDefinition, WorkspaceProfileId,
    toolbar_workspace_button_definitions,
};
use ui_definition::{UiAvailabilityBinding, UiAvailabilityId, UiRouteSlotId};

use crate::shell::{EditorCommandAvailabilityContext, editor_command_catalog};

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
const WORKSPACE_PLUS_ID: ToolId = ToolId(3_003);

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
    let definition = toolbar_workspace_button_definitions()
        .iter()
        .find(|definition| definition.profile_id == profile_id)?;
    Some(ToolbarObservedButton {
        id: definition.tool_id,
        stable_name: definition.stable_name,
        label: definition.label.to_string(),
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
            catalog_menu_item(
                2_100,
                "file_save",
                "editor.toolbar.file.save",
                can_undo,
                can_redo,
            ),
            catalog_menu_item(
                2_101,
                "file_save_as",
                "editor.toolbar.file.save_as",
                can_undo,
                can_redo,
            ),
            catalog_menu_item(
                2_102,
                "file_open",
                "editor.toolbar.file.open",
                can_undo,
                can_redo,
            ),
            catalog_menu_item(
                2_103,
                "file_open_recent",
                "editor.toolbar.file.open_recent",
                can_undo,
                can_redo,
            ),
        ],
        Some(ToolbarMenuKind::Edit) => vec![
            catalog_menu_item(
                2_200,
                "edit_undo",
                "editor.toolbar.edit.undo",
                can_undo,
                can_redo,
            ),
            catalog_menu_item(
                2_201,
                "edit_redo",
                "editor.toolbar.edit.redo",
                can_undo,
                can_redo,
            ),
            catalog_menu_item(
                2_202,
                "edit_preferences",
                "editor.toolbar.edit.preferences",
                can_undo,
                can_redo,
            ),
        ],
        Some(ToolbarMenuKind::Window) => vec![
            catalog_menu_item(
                2_300,
                "window_new_window",
                "editor.toolbar.window.new_window",
                can_undo,
                can_redo,
            ),
            catalog_menu_item(
                2_301,
                "window_next_workspace",
                "editor.toolbar.window.next_workspace",
                can_undo,
                can_redo,
            ),
            catalog_menu_item(
                2_302,
                "window_previous_workspace",
                "editor.toolbar.window.previous_workspace",
                can_undo,
                can_redo,
            ),
            catalog_menu_item(
                2_303,
                "window_save_workspace",
                "editor.toolbar.window.save_workspace",
                can_undo,
                can_redo,
            ),
            menu_item(2_307, "window_load_custom_label", "Load: Custom", false),
            catalog_menu_item(
                2_308,
                "window_load_custom",
                "editor.toolbar.window.load_custom_workspace",
                can_undo,
                can_redo,
            ),
        ],
        Some(ToolbarMenuKind::Workspace) => toolbar_workspace_button_definitions()
            .iter()
            .map(workspace_menu_item)
            .collect(),
        None => Vec::new(),
    }
}

fn workspace_menu_item(definition: &ToolbarWorkspaceButtonDefinition) -> ToolbarObservedButton {
    ToolbarObservedButton {
        id: definition.menu_item_tool_id,
        stable_name: definition.menu_item_stable_name,
        label: definition.label.to_string(),
        is_active: false,
        enabled: true,
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

fn catalog_menu_item(
    id: u64,
    stable_name: &'static str,
    route: &'static str,
    can_undo: bool,
    can_redo: bool,
) -> ToolbarObservedButton {
    let descriptor = editor_command_catalog()
        .descriptor_for_key(route)
        .expect("compiled-in toolbar route should have a command descriptor");
    let availability =
        descriptor.availability(EditorCommandAvailabilityContext { can_undo, can_redo });
    ToolbarObservedButton {
        id: ToolId(id),
        stable_name,
        label: descriptor.label.to_string(),
        is_active: false,
        enabled: availability.is_enabled(),
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use editor_core::RealityVersion;
    use editor_shell::{MATERIAL_WORKSPACE_PROFILE_ID, SCENE_WORKSPACE_PROFILE_ID};

    use super::*;

    #[test]
    fn material_profile_projects_as_workspace_button_and_menu_item() {
        let frame = build_toolbar_observation_frame(
            None,
            false,
            false,
            false,
            Some(ToolbarMenuKind::Workspace),
            MATERIAL_WORKSPACE_PROFILE_ID,
            &[SCENE_WORKSPACE_PROFILE_ID, MATERIAL_WORKSPACE_PROFILE_ID],
            RealityVersion(1),
            &BTreeMap::new(),
        );

        let material_workspace = frame
            .buttons
            .iter()
            .find(|button| button.stable_name == "workspace_materials")
            .expect("open Materials profile should project a top-bar workspace button");
        assert_eq!(material_workspace.label, "Materials");
        assert!(material_workspace.is_active);

        let material_menu_item = frame
            .buttons
            .iter()
            .find(|button| button.stable_name == "workspace_menu_materials")
            .expect("workspace menu should expose the Materials workspace");
        assert_eq!(material_menu_item.label, "Materials");
        assert!(material_menu_item.enabled);
    }
}
