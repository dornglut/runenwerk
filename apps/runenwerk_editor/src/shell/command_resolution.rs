//! File: apps/runenwerk_editor/src/shell/command_resolution.rs
//! Purpose: App-owned resolution of authored editor command keys to shell actions.

use std::collections::BTreeMap;

use editor_definition::EditorMenuItemDefinition;
use editor_shell::{
    EDITOR_DESIGN_WORKSPACE_PROFILE_ID, MATERIAL_WORKSPACE_PROFILE_ID,
    MODELLING_WORKSPACE_PROFILE_ID, RoutedShellAction, SCENE_WORKSPACE_PROFILE_ID, ShellCommand,
    ToolbarCommandKind, ToolbarMenuKind,
};

use crate::shell::ActiveEditorDefinitionCatalogs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KnownEditorCommand {
    ActivateSelectTool,
    ActivateTranslateTool,
    ActivateRotateTool,
    ActivateScaleTool,
    ToggleFileMenu,
    ToggleEditMenu,
    ToggleWindowMenu,
    ToggleWorkspaceMenu,
    Undo,
    Redo,
    SaveScene,
    SaveSceneAs,
    LoadScene,
    OpenRecent,
    EditPreferences,
    NewWindow,
    NextWorkspace,
    PreviousWorkspace,
    SaveWorkspace,
    LoadSceneWorkspace,
    LoadModellingWorkspace,
    LoadMaterialWorkspace,
    LoadCustomWorkspace,
    AddWorkspace,
    SwitchSceneWorkspace,
    SwitchModellingWorkspace,
    SwitchEditorDesignWorkspace,
    SwitchMaterialWorkspace,
    CloseSceneWorkspace,
    CloseModellingWorkspace,
    CloseEditorDesignWorkspace,
    CloseMaterialWorkspace,
    ToggleDebugLogs,
    ApplySelectedEditorDefinition,
}

impl KnownEditorCommand {
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "editor.tool.select" => Some(Self::ActivateSelectTool),
            "editor.tool.translate" => Some(Self::ActivateTranslateTool),
            "editor.tool.rotate" => Some(Self::ActivateRotateTool),
            "editor.tool.scale" => Some(Self::ActivateScaleTool),
            "editor.toolbar.menu.file" | "editor.menu.file.toggle" => Some(Self::ToggleFileMenu),
            "editor.toolbar.menu.edit" | "editor.menu.edit.toggle" => Some(Self::ToggleEditMenu),
            "editor.toolbar.menu.window" | "editor.menu.window.toggle" => {
                Some(Self::ToggleWindowMenu)
            }
            "editor.toolbar.menu.workspace" | "editor.menu.workspace.toggle" => {
                Some(Self::ToggleWorkspaceMenu)
            }
            "editor.history.undo" | "editor.undo" | "editor.toolbar.edit.undo" => Some(Self::Undo),
            "editor.history.redo" | "editor.redo" | "editor.toolbar.edit.redo" => Some(Self::Redo),
            "editor.scene.save" | "editor.toolbar.file.save" => Some(Self::SaveScene),
            "editor.scene.save_as" | "editor.toolbar.file.save_as" => Some(Self::SaveSceneAs),
            "editor.scene.load" | "editor.toolbar.file.open" => Some(Self::LoadScene),
            "editor.scene.open_recent" | "editor.toolbar.file.open_recent" => {
                Some(Self::OpenRecent)
            }
            "editor.preferences.open" | "editor.toolbar.edit.preferences" => {
                Some(Self::EditPreferences)
            }
            "editor.window.new" | "editor.toolbar.window.new_window" => Some(Self::NewWindow),
            "editor.workspace.next" | "editor.toolbar.window.next_workspace" => {
                Some(Self::NextWorkspace)
            }
            "editor.workspace.previous" | "editor.toolbar.window.previous_workspace" => {
                Some(Self::PreviousWorkspace)
            }
            "editor.workspace.save" | "editor.toolbar.window.save_workspace" => {
                Some(Self::SaveWorkspace)
            }
            "editor.workspace.load.scene" | "editor.toolbar.window.load_scene_workspace" => {
                Some(Self::LoadSceneWorkspace)
            }
            "editor.workspace.load.modelling"
            | "editor.toolbar.window.load_modelling_workspace" => {
                Some(Self::LoadModellingWorkspace)
            }
            "editor.workspace.load.materials"
            | "editor.toolbar.window.load_materials_workspace" => Some(Self::LoadMaterialWorkspace),
            "editor.workspace.load.custom" | "editor.toolbar.window.load_custom_workspace" => {
                Some(Self::LoadCustomWorkspace)
            }
            "editor.workspace.create" => Some(Self::AddWorkspace),
            "editor.workspace.scene.activate" => Some(Self::SwitchSceneWorkspace),
            "editor.workspace.modelling.activate" => Some(Self::SwitchModellingWorkspace),
            "editor.workspace.editor_design.activate" => Some(Self::SwitchEditorDesignWorkspace),
            "editor.workspace.materials.activate" => Some(Self::SwitchMaterialWorkspace),
            "editor.workspace.scene.close" => Some(Self::CloseSceneWorkspace),
            "editor.workspace.modelling.close" => Some(Self::CloseModellingWorkspace),
            "editor.workspace.editor_design.close" => Some(Self::CloseEditorDesignWorkspace),
            "editor.workspace.materials.close" => Some(Self::CloseMaterialWorkspace),
            "editor.debug.toggle_logs" => Some(Self::ToggleDebugLogs),
            "editor.definition.apply_selected" => Some(Self::ApplySelectedEditorDefinition),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::ActivateSelectTool => "editor.tool.select",
            Self::ActivateTranslateTool => "editor.tool.translate",
            Self::ActivateRotateTool => "editor.tool.rotate",
            Self::ActivateScaleTool => "editor.tool.scale",
            Self::ToggleFileMenu => "editor.menu.file.toggle",
            Self::ToggleEditMenu => "editor.menu.edit.toggle",
            Self::ToggleWindowMenu => "editor.menu.window.toggle",
            Self::ToggleWorkspaceMenu => "editor.menu.workspace.toggle",
            Self::Undo => "editor.history.undo",
            Self::Redo => "editor.history.redo",
            Self::SaveScene => "editor.scene.save",
            Self::SaveSceneAs => "editor.scene.save_as",
            Self::LoadScene => "editor.scene.load",
            Self::OpenRecent => "editor.scene.open_recent",
            Self::EditPreferences => "editor.preferences.open",
            Self::NewWindow => "editor.window.new",
            Self::NextWorkspace => "editor.workspace.next",
            Self::PreviousWorkspace => "editor.workspace.previous",
            Self::SaveWorkspace => "editor.workspace.save",
            Self::LoadSceneWorkspace => "editor.workspace.load.scene",
            Self::LoadModellingWorkspace => "editor.workspace.load.modelling",
            Self::LoadMaterialWorkspace => "editor.workspace.load.materials",
            Self::LoadCustomWorkspace => "editor.workspace.load.custom",
            Self::AddWorkspace => "editor.workspace.create",
            Self::SwitchSceneWorkspace => "editor.workspace.scene.activate",
            Self::SwitchModellingWorkspace => "editor.workspace.modelling.activate",
            Self::SwitchEditorDesignWorkspace => "editor.workspace.editor_design.activate",
            Self::SwitchMaterialWorkspace => "editor.workspace.materials.activate",
            Self::CloseSceneWorkspace => "editor.workspace.scene.close",
            Self::CloseModellingWorkspace => "editor.workspace.modelling.close",
            Self::CloseEditorDesignWorkspace => "editor.workspace.editor_design.close",
            Self::CloseMaterialWorkspace => "editor.workspace.materials.close",
            Self::ToggleDebugLogs => "editor.debug.toggle_logs",
            Self::ApplySelectedEditorDefinition => "editor.definition.apply_selected",
        }
    }

    pub fn all() -> &'static [KnownEditorCommand] {
        &[
            Self::ActivateSelectTool,
            Self::ActivateTranslateTool,
            Self::ActivateRotateTool,
            Self::ActivateScaleTool,
            Self::ToggleFileMenu,
            Self::ToggleEditMenu,
            Self::ToggleWindowMenu,
            Self::ToggleWorkspaceMenu,
            Self::Undo,
            Self::Redo,
            Self::SaveScene,
            Self::SaveSceneAs,
            Self::LoadScene,
            Self::OpenRecent,
            Self::EditPreferences,
            Self::NewWindow,
            Self::NextWorkspace,
            Self::PreviousWorkspace,
            Self::SaveWorkspace,
            Self::LoadSceneWorkspace,
            Self::LoadModellingWorkspace,
            Self::LoadMaterialWorkspace,
            Self::LoadCustomWorkspace,
            Self::AddWorkspace,
            Self::SwitchSceneWorkspace,
            Self::SwitchModellingWorkspace,
            Self::SwitchEditorDesignWorkspace,
            Self::SwitchMaterialWorkspace,
            Self::CloseSceneWorkspace,
            Self::CloseModellingWorkspace,
            Self::CloseEditorDesignWorkspace,
            Self::CloseMaterialWorkspace,
            Self::ToggleDebugLogs,
            Self::ApplySelectedEditorDefinition,
        ]
    }

    pub fn to_routed_shell_action(self, can_undo: bool, can_redo: bool) -> RoutedShellAction {
        match self {
            Self::ActivateSelectTool => RoutedShellAction::ActivateSelectTool,
            Self::ActivateTranslateTool => RoutedShellAction::ActivateTranslateTool,
            Self::ActivateRotateTool => RoutedShellAction::ActivateRotateTool,
            Self::ActivateScaleTool => RoutedShellAction::ActivateScaleTool,
            Self::ToggleFileMenu => RoutedShellAction::ToggleToolbarMenu {
                menu: ToolbarMenuKind::File,
            },
            Self::ToggleEditMenu => RoutedShellAction::ToggleToolbarMenu {
                menu: ToolbarMenuKind::Edit,
            },
            Self::ToggleWindowMenu => RoutedShellAction::ToggleToolbarMenu {
                menu: ToolbarMenuKind::Window,
            },
            Self::ToggleWorkspaceMenu => RoutedShellAction::ToggleToolbarMenu {
                menu: ToolbarMenuKind::Workspace,
            },
            Self::Undo => RoutedShellAction::Undo { enabled: can_undo },
            Self::Redo => RoutedShellAction::Redo { enabled: can_redo },
            Self::SaveScene => RoutedShellAction::SaveScene { enabled: true },
            Self::LoadScene => RoutedShellAction::LoadScene { enabled: true },
            Self::ToggleDebugLogs => RoutedShellAction::ToggleDebugLogs,
            Self::SwitchSceneWorkspace => RoutedShellAction::SwitchWorkspaceProfile {
                profile_id: SCENE_WORKSPACE_PROFILE_ID,
                enabled: true,
            },
            Self::SwitchModellingWorkspace => RoutedShellAction::SwitchWorkspaceProfile {
                profile_id: MODELLING_WORKSPACE_PROFILE_ID,
                enabled: true,
            },
            Self::SwitchEditorDesignWorkspace => RoutedShellAction::SwitchWorkspaceProfile {
                profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
                enabled: true,
            },
            Self::SwitchMaterialWorkspace => RoutedShellAction::SwitchWorkspaceProfile {
                profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
                enabled: true,
            },
            Self::CloseSceneWorkspace => RoutedShellAction::CloseWorkspaceProfile {
                profile_id: SCENE_WORKSPACE_PROFILE_ID,
                enabled: true,
            },
            Self::CloseModellingWorkspace => RoutedShellAction::CloseWorkspaceProfile {
                profile_id: MODELLING_WORKSPACE_PROFILE_ID,
                enabled: true,
            },
            Self::CloseEditorDesignWorkspace => RoutedShellAction::CloseWorkspaceProfile {
                profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
                enabled: true,
            },
            Self::CloseMaterialWorkspace => RoutedShellAction::CloseWorkspaceProfile {
                profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
                enabled: true,
            },
            Self::SaveSceneAs
            | Self::OpenRecent
            | Self::EditPreferences
            | Self::NewWindow
            | Self::NextWorkspace
            | Self::PreviousWorkspace
            | Self::SaveWorkspace
            | Self::LoadSceneWorkspace
            | Self::LoadModellingWorkspace
            | Self::LoadMaterialWorkspace
            | Self::LoadCustomWorkspace
            | Self::AddWorkspace => RoutedShellAction::RunToolbarCommand {
                command: self.toolbar_command().expect("toolbar command variant"),
                enabled: true,
            },
            Self::ApplySelectedEditorDefinition => RoutedShellAction::ApplySelectedEditorDefinition,
        }
    }

    pub fn to_shell_command(self) -> ShellCommand {
        match self {
            Self::ActivateSelectTool => ShellCommand::ActivateSelectTool,
            Self::ActivateTranslateTool => ShellCommand::ActivateTranslateTool,
            Self::ActivateRotateTool => ShellCommand::ActivateRotateTool,
            Self::ActivateScaleTool => ShellCommand::ActivateScaleTool,
            Self::ToggleFileMenu => ShellCommand::ToggleToolbarMenu {
                menu: ToolbarMenuKind::File,
            },
            Self::ToggleEditMenu => ShellCommand::ToggleToolbarMenu {
                menu: ToolbarMenuKind::Edit,
            },
            Self::ToggleWindowMenu => ShellCommand::ToggleToolbarMenu {
                menu: ToolbarMenuKind::Window,
            },
            Self::ToggleWorkspaceMenu => ShellCommand::ToggleToolbarMenu {
                menu: ToolbarMenuKind::Workspace,
            },
            Self::Undo => ShellCommand::Undo,
            Self::Redo => ShellCommand::Redo,
            Self::SaveScene => ShellCommand::SaveScene,
            Self::LoadScene => ShellCommand::LoadScene,
            Self::ToggleDebugLogs => ShellCommand::ToggleDebugLogs,
            Self::ApplySelectedEditorDefinition => ShellCommand::ApplySelectedEditorDefinition,
            Self::SwitchSceneWorkspace => ShellCommand::SwitchWorkspaceProfile {
                profile_id: SCENE_WORKSPACE_PROFILE_ID,
            },
            Self::SwitchModellingWorkspace => ShellCommand::SwitchWorkspaceProfile {
                profile_id: MODELLING_WORKSPACE_PROFILE_ID,
            },
            Self::SwitchEditorDesignWorkspace => ShellCommand::SwitchWorkspaceProfile {
                profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
            },
            Self::SwitchMaterialWorkspace => ShellCommand::SwitchWorkspaceProfile {
                profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
            },
            Self::CloseSceneWorkspace => ShellCommand::CloseWorkspaceProfile {
                profile_id: SCENE_WORKSPACE_PROFILE_ID,
            },
            Self::CloseModellingWorkspace => ShellCommand::CloseWorkspaceProfile {
                profile_id: MODELLING_WORKSPACE_PROFILE_ID,
            },
            Self::CloseEditorDesignWorkspace => ShellCommand::CloseWorkspaceProfile {
                profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
            },
            Self::CloseMaterialWorkspace => ShellCommand::CloseWorkspaceProfile {
                profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
            },
            Self::SaveSceneAs
            | Self::OpenRecent
            | Self::EditPreferences
            | Self::NewWindow
            | Self::NextWorkspace
            | Self::PreviousWorkspace
            | Self::SaveWorkspace
            | Self::LoadSceneWorkspace
            | Self::LoadModellingWorkspace
            | Self::LoadMaterialWorkspace
            | Self::LoadCustomWorkspace
            | Self::AddWorkspace => ShellCommand::RunToolbarCommand {
                command: self.toolbar_command().expect("toolbar command variant"),
            },
        }
    }

    fn toolbar_command(self) -> Option<ToolbarCommandKind> {
        match self {
            Self::SaveSceneAs => Some(ToolbarCommandKind::SaveSceneAs),
            Self::OpenRecent => Some(ToolbarCommandKind::OpenRecent),
            Self::EditPreferences => Some(ToolbarCommandKind::EditPreferences),
            Self::NewWindow => Some(ToolbarCommandKind::NewWindow),
            Self::NextWorkspace => Some(ToolbarCommandKind::NextWorkspace),
            Self::PreviousWorkspace => Some(ToolbarCommandKind::PreviousWorkspace),
            Self::SaveWorkspace => Some(ToolbarCommandKind::SaveWorkspace),
            Self::LoadSceneWorkspace => Some(ToolbarCommandKind::LoadWorkspaceProfile(
                SCENE_WORKSPACE_PROFILE_ID,
            )),
            Self::LoadModellingWorkspace => Some(ToolbarCommandKind::LoadWorkspaceProfile(
                MODELLING_WORKSPACE_PROFILE_ID,
            )),
            Self::LoadMaterialWorkspace => Some(ToolbarCommandKind::LoadWorkspaceProfile(
                MATERIAL_WORKSPACE_PROFILE_ID,
            )),
            Self::LoadCustomWorkspace => Some(ToolbarCommandKind::LoadCustomWorkspace),
            Self::AddWorkspace => Some(ToolbarCommandKind::AddWorkspace),
            _ => None,
        }
    }
}

pub fn is_known_editor_command_key(key: &str) -> bool {
    KnownEditorCommand::from_key(key).is_some()
}

pub fn active_route_actions_by_target(
    catalogs: &ActiveEditorDefinitionCatalogs,
    can_undo: bool,
    can_redo: bool,
) -> BTreeMap<String, RoutedShellAction> {
    let mut routes = BTreeMap::new();
    for command in KnownEditorCommand::all() {
        routes.insert(
            command.key().to_string(),
            command.to_routed_shell_action(can_undo, can_redo),
        );
    }
    for binding in catalogs
        .command_bindings()
        .values()
        .flat_map(|set| set.bindings.iter())
    {
        if let Some(command) = KnownEditorCommand::from_key(&binding.command) {
            routes.insert(
                binding.route_target.clone(),
                command.to_routed_shell_action(can_undo, can_redo),
            );
            routes.insert(
                binding.command.clone(),
                command.to_routed_shell_action(can_undo, can_redo),
            );
        }
    }
    let mut menu_commands = Vec::new();
    for menu in catalogs.menus().values() {
        collect_menu_commands(&menu.items, &mut menu_commands);
    }
    for command in menu_commands {
        if let Some(command_key) = KnownEditorCommand::from_key(command) {
            routes.insert(
                command.to_string(),
                command_key.to_routed_shell_action(can_undo, can_redo),
            );
        }
    }
    routes
}

fn collect_menu_commands<'a>(items: &'a [EditorMenuItemDefinition], output: &mut Vec<&'a str>) {
    for item in items {
        if let Some(command) = item.command.as_deref() {
            output.push(command);
        }
        collect_menu_commands(&item.children, output);
    }
}
