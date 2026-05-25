//! File: apps/runenwerk_editor/src/shell/command_catalog/mod.rs
//! Purpose: App-owned source of truth for normal editor shell commands.

use editor_shell::{
    EDITOR_DESIGN_WORKSPACE_PROFILE_ID, MATERIAL_WORKSPACE_PROFILE_ID,
    MODELLING_WORKSPACE_PROFILE_ID, RoutedShellAction, SCENE_WORKSPACE_PROFILE_ID, ShellCommand,
    ToolbarCommandKind, ToolbarMenuKind,
};

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
        editor_command_catalog().command_for_key(key)
    }

    pub fn key(self) -> &'static str {
        editor_command_catalog()
            .descriptor(self)
            .expect("known command should have a catalog descriptor")
            .key
    }

    pub fn all() -> &'static [KnownEditorCommand] {
        EDITOR_COMMAND_ORDER
    }

    pub fn to_routed_shell_action(self, can_undo: bool, can_redo: bool) -> RoutedShellAction {
        editor_command_catalog()
            .descriptor(self)
            .expect("known command should have a catalog descriptor")
            .routed_shell_action(EditorCommandAvailabilityContext { can_undo, can_redo })
    }

    pub fn to_shell_command(self) -> ShellCommand {
        editor_command_catalog()
            .descriptor(self)
            .expect("known command should have a catalog descriptor")
            .shell_command()
    }

    pub fn toolbar_command(self) -> Option<ToolbarCommandKind> {
        editor_command_catalog()
            .descriptor(self)
            .and_then(EditorCommandDescriptor::toolbar_command)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorCommandAvailabilityRule {
    Always,
    CanUndo,
    CanRedo,
    StaticDisabled {
        diagnostic_code: &'static str,
        reason: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorCommandAvailability {
    Available,
    Disabled {
        diagnostic_code: &'static str,
        reason: &'static str,
    },
}

impl EditorCommandAvailability {
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Available)
    }

    pub fn diagnostic_code(self) -> Option<&'static str> {
        match self {
            Self::Available => None,
            Self::Disabled {
                diagnostic_code, ..
            } => Some(diagnostic_code),
        }
    }

    pub fn reason(self) -> Option<&'static str> {
        match self {
            Self::Available => None,
            Self::Disabled { reason, .. } => Some(reason),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorCommandAvailabilityContext {
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorCommandDescriptor {
    pub command: KnownEditorCommand,
    pub key: &'static str,
    pub aliases: &'static [&'static str],
    pub label: &'static str,
    pub availability: EditorCommandAvailabilityRule,
}

impl EditorCommandDescriptor {
    pub fn matches_key(&self, key: &str) -> bool {
        self.key == key || self.aliases.contains(&key)
    }

    pub fn route_targets(&self) -> impl Iterator<Item = &'static str> + '_ {
        std::iter::once(self.key).chain(self.aliases.iter().copied())
    }

    pub fn availability(
        &self,
        context: EditorCommandAvailabilityContext,
    ) -> EditorCommandAvailability {
        match self.availability {
            EditorCommandAvailabilityRule::Always => EditorCommandAvailability::Available,
            EditorCommandAvailabilityRule::CanUndo => {
                if context.can_undo {
                    EditorCommandAvailability::Available
                } else {
                    EditorCommandAvailability::Disabled {
                        diagnostic_code: "editor.command.unavailable.undo",
                        reason: "nothing to undo",
                    }
                }
            }
            EditorCommandAvailabilityRule::CanRedo => {
                if context.can_redo {
                    EditorCommandAvailability::Available
                } else {
                    EditorCommandAvailability::Disabled {
                        diagnostic_code: "editor.command.unavailable.redo",
                        reason: "nothing to redo",
                    }
                }
            }
            EditorCommandAvailabilityRule::StaticDisabled {
                diagnostic_code,
                reason,
            } => EditorCommandAvailability::Disabled {
                diagnostic_code,
                reason,
            },
        }
    }

    pub fn routed_shell_action(
        &self,
        context: EditorCommandAvailabilityContext,
    ) -> RoutedShellAction {
        let enabled = self.availability(context).is_enabled();
        match self.command {
            KnownEditorCommand::ActivateSelectTool => RoutedShellAction::ActivateSelectTool,
            KnownEditorCommand::ActivateTranslateTool => RoutedShellAction::ActivateTranslateTool,
            KnownEditorCommand::ActivateRotateTool => RoutedShellAction::ActivateRotateTool,
            KnownEditorCommand::ActivateScaleTool => RoutedShellAction::ActivateScaleTool,
            KnownEditorCommand::ToggleFileMenu => RoutedShellAction::ToggleToolbarMenu {
                menu: ToolbarMenuKind::File,
            },
            KnownEditorCommand::ToggleEditMenu => RoutedShellAction::ToggleToolbarMenu {
                menu: ToolbarMenuKind::Edit,
            },
            KnownEditorCommand::ToggleWindowMenu => RoutedShellAction::ToggleToolbarMenu {
                menu: ToolbarMenuKind::Window,
            },
            KnownEditorCommand::ToggleWorkspaceMenu => RoutedShellAction::ToggleToolbarMenu {
                menu: ToolbarMenuKind::Workspace,
            },
            KnownEditorCommand::Undo => RoutedShellAction::Undo { enabled },
            KnownEditorCommand::Redo => RoutedShellAction::Redo { enabled },
            KnownEditorCommand::SaveScene => RoutedShellAction::SaveScene { enabled },
            KnownEditorCommand::LoadScene => RoutedShellAction::LoadScene { enabled },
            KnownEditorCommand::ToggleDebugLogs => RoutedShellAction::ToggleDebugLogs,
            KnownEditorCommand::SwitchSceneWorkspace => RoutedShellAction::SwitchWorkspaceProfile {
                profile_id: SCENE_WORKSPACE_PROFILE_ID,
                enabled,
            },
            KnownEditorCommand::SwitchModellingWorkspace => {
                RoutedShellAction::SwitchWorkspaceProfile {
                    profile_id: MODELLING_WORKSPACE_PROFILE_ID,
                    enabled,
                }
            }
            KnownEditorCommand::SwitchEditorDesignWorkspace => {
                RoutedShellAction::SwitchWorkspaceProfile {
                    profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
                    enabled,
                }
            }
            KnownEditorCommand::SwitchMaterialWorkspace => {
                RoutedShellAction::SwitchWorkspaceProfile {
                    profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
                    enabled,
                }
            }
            KnownEditorCommand::CloseSceneWorkspace => RoutedShellAction::CloseWorkspaceProfile {
                profile_id: SCENE_WORKSPACE_PROFILE_ID,
                enabled,
            },
            KnownEditorCommand::CloseModellingWorkspace => {
                RoutedShellAction::CloseWorkspaceProfile {
                    profile_id: MODELLING_WORKSPACE_PROFILE_ID,
                    enabled,
                }
            }
            KnownEditorCommand::CloseEditorDesignWorkspace => {
                RoutedShellAction::CloseWorkspaceProfile {
                    profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
                    enabled,
                }
            }
            KnownEditorCommand::CloseMaterialWorkspace => {
                RoutedShellAction::CloseWorkspaceProfile {
                    profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
                    enabled,
                }
            }
            KnownEditorCommand::ApplySelectedEditorDefinition => {
                RoutedShellAction::ApplySelectedEditorDefinition
            }
            KnownEditorCommand::SaveSceneAs
            | KnownEditorCommand::OpenRecent
            | KnownEditorCommand::EditPreferences
            | KnownEditorCommand::NewWindow
            | KnownEditorCommand::NextWorkspace
            | KnownEditorCommand::PreviousWorkspace
            | KnownEditorCommand::SaveWorkspace
            | KnownEditorCommand::LoadSceneWorkspace
            | KnownEditorCommand::LoadModellingWorkspace
            | KnownEditorCommand::LoadMaterialWorkspace
            | KnownEditorCommand::LoadCustomWorkspace
            | KnownEditorCommand::AddWorkspace => RoutedShellAction::RunToolbarCommand {
                command: self
                    .toolbar_command()
                    .expect("toolbar command descriptor should have toolbar command"),
                enabled,
            },
        }
    }

    pub fn shell_command(&self) -> ShellCommand {
        match self.command {
            KnownEditorCommand::ActivateSelectTool => ShellCommand::ActivateSelectTool,
            KnownEditorCommand::ActivateTranslateTool => ShellCommand::ActivateTranslateTool,
            KnownEditorCommand::ActivateRotateTool => ShellCommand::ActivateRotateTool,
            KnownEditorCommand::ActivateScaleTool => ShellCommand::ActivateScaleTool,
            KnownEditorCommand::ToggleFileMenu => ShellCommand::ToggleToolbarMenu {
                menu: ToolbarMenuKind::File,
            },
            KnownEditorCommand::ToggleEditMenu => ShellCommand::ToggleToolbarMenu {
                menu: ToolbarMenuKind::Edit,
            },
            KnownEditorCommand::ToggleWindowMenu => ShellCommand::ToggleToolbarMenu {
                menu: ToolbarMenuKind::Window,
            },
            KnownEditorCommand::ToggleWorkspaceMenu => ShellCommand::ToggleToolbarMenu {
                menu: ToolbarMenuKind::Workspace,
            },
            KnownEditorCommand::Undo => ShellCommand::Undo,
            KnownEditorCommand::Redo => ShellCommand::Redo,
            KnownEditorCommand::SaveScene => ShellCommand::SaveScene,
            KnownEditorCommand::LoadScene => ShellCommand::LoadScene,
            KnownEditorCommand::ToggleDebugLogs => ShellCommand::ToggleDebugLogs,
            KnownEditorCommand::ApplySelectedEditorDefinition => {
                ShellCommand::ApplySelectedEditorDefinition
            }
            KnownEditorCommand::SwitchSceneWorkspace => ShellCommand::SwitchWorkspaceProfile {
                profile_id: SCENE_WORKSPACE_PROFILE_ID,
            },
            KnownEditorCommand::SwitchModellingWorkspace => ShellCommand::SwitchWorkspaceProfile {
                profile_id: MODELLING_WORKSPACE_PROFILE_ID,
            },
            KnownEditorCommand::SwitchEditorDesignWorkspace => {
                ShellCommand::SwitchWorkspaceProfile {
                    profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
                }
            }
            KnownEditorCommand::SwitchMaterialWorkspace => ShellCommand::SwitchWorkspaceProfile {
                profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
            },
            KnownEditorCommand::CloseSceneWorkspace => ShellCommand::CloseWorkspaceProfile {
                profile_id: SCENE_WORKSPACE_PROFILE_ID,
            },
            KnownEditorCommand::CloseModellingWorkspace => ShellCommand::CloseWorkspaceProfile {
                profile_id: MODELLING_WORKSPACE_PROFILE_ID,
            },
            KnownEditorCommand::CloseEditorDesignWorkspace => ShellCommand::CloseWorkspaceProfile {
                profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
            },
            KnownEditorCommand::CloseMaterialWorkspace => ShellCommand::CloseWorkspaceProfile {
                profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
            },
            KnownEditorCommand::SaveSceneAs
            | KnownEditorCommand::OpenRecent
            | KnownEditorCommand::EditPreferences
            | KnownEditorCommand::NewWindow
            | KnownEditorCommand::NextWorkspace
            | KnownEditorCommand::PreviousWorkspace
            | KnownEditorCommand::SaveWorkspace
            | KnownEditorCommand::LoadSceneWorkspace
            | KnownEditorCommand::LoadModellingWorkspace
            | KnownEditorCommand::LoadMaterialWorkspace
            | KnownEditorCommand::LoadCustomWorkspace
            | KnownEditorCommand::AddWorkspace => ShellCommand::RunToolbarCommand {
                command: self
                    .toolbar_command()
                    .expect("toolbar command descriptor should have toolbar command"),
            },
        }
    }

    pub fn toolbar_command(&self) -> Option<ToolbarCommandKind> {
        match self.command {
            KnownEditorCommand::SaveSceneAs => Some(ToolbarCommandKind::SaveSceneAs),
            KnownEditorCommand::OpenRecent => Some(ToolbarCommandKind::OpenRecent),
            KnownEditorCommand::EditPreferences => Some(ToolbarCommandKind::EditPreferences),
            KnownEditorCommand::NewWindow => Some(ToolbarCommandKind::NewWindow),
            KnownEditorCommand::NextWorkspace => Some(ToolbarCommandKind::NextWorkspace),
            KnownEditorCommand::PreviousWorkspace => Some(ToolbarCommandKind::PreviousWorkspace),
            KnownEditorCommand::SaveWorkspace => Some(ToolbarCommandKind::SaveWorkspace),
            KnownEditorCommand::LoadSceneWorkspace => Some(
                ToolbarCommandKind::LoadWorkspaceProfile(SCENE_WORKSPACE_PROFILE_ID),
            ),
            KnownEditorCommand::LoadModellingWorkspace => Some(
                ToolbarCommandKind::LoadWorkspaceProfile(MODELLING_WORKSPACE_PROFILE_ID),
            ),
            KnownEditorCommand::LoadMaterialWorkspace => Some(
                ToolbarCommandKind::LoadWorkspaceProfile(MATERIAL_WORKSPACE_PROFILE_ID),
            ),
            KnownEditorCommand::LoadCustomWorkspace => {
                Some(ToolbarCommandKind::LoadCustomWorkspace)
            }
            KnownEditorCommand::AddWorkspace => Some(ToolbarCommandKind::AddWorkspace),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorCommandCatalogDiagnostic {
    DuplicateCommandKey {
        key: &'static str,
        first: KnownEditorCommand,
        duplicate: KnownEditorCommand,
    },
    DuplicateAlias {
        alias: &'static str,
        first: KnownEditorCommand,
        duplicate: KnownEditorCommand,
    },
    DuplicateRouteTarget {
        route_target: &'static str,
        first: KnownEditorCommand,
        duplicate: KnownEditorCommand,
    },
    MissingKnownCommand {
        command: KnownEditorCommand,
    },
    EmptyCommandLabel {
        command: KnownEditorCommand,
    },
    EmptyStaticDisabledDiagnostic {
        command: KnownEditorCommand,
    },
    EmptyStaticDisabledReason {
        command: KnownEditorCommand,
    },
    RouteTargetDoesNotRoundTrip {
        command: KnownEditorCommand,
        route_target: &'static str,
    },
    ToolbarCommandDoesNotRoundTrip {
        command: KnownEditorCommand,
        toolbar_command: ToolbarCommandKind,
    },
}

pub struct EditorCommandCatalog {
    descriptors: &'static [EditorCommandDescriptor],
}

impl EditorCommandCatalog {
    pub const fn new(descriptors: &'static [EditorCommandDescriptor]) -> Self {
        Self { descriptors }
    }

    pub fn descriptors(&self) -> &'static [EditorCommandDescriptor] {
        self.descriptors
    }

    pub fn descriptor(
        &self,
        command: KnownEditorCommand,
    ) -> Option<&'static EditorCommandDescriptor> {
        self.descriptors
            .iter()
            .find(|descriptor| descriptor.command == command)
    }

    pub fn descriptor_for_key(&self, key: &str) -> Option<&'static EditorCommandDescriptor> {
        self.descriptors
            .iter()
            .find(|descriptor| descriptor.matches_key(key))
    }

    pub fn command_for_key(&self, key: &str) -> Option<KnownEditorCommand> {
        self.descriptor_for_key(key)
            .map(|descriptor| descriptor.command)
    }

    pub fn descriptor_for_toolbar_command(
        &self,
        toolbar_command: ToolbarCommandKind,
    ) -> Option<&'static EditorCommandDescriptor> {
        self.descriptors
            .iter()
            .find(|descriptor| descriptor.toolbar_command() == Some(toolbar_command))
    }

    pub fn is_known_key(&self, key: &str) -> bool {
        self.descriptor_for_key(key).is_some()
    }

    pub fn validate(&self) -> Result<(), Vec<EditorCommandCatalogDiagnostic>> {
        let mut diagnostics = Vec::new();
        let mut keys = std::collections::BTreeMap::<&'static str, KnownEditorCommand>::new();
        let mut aliases = std::collections::BTreeMap::<&'static str, KnownEditorCommand>::new();
        let mut route_targets =
            std::collections::BTreeMap::<&'static str, KnownEditorCommand>::new();
        for descriptor in self.descriptors {
            if let Some(first) = keys.insert(descriptor.key, descriptor.command) {
                diagnostics.push(EditorCommandCatalogDiagnostic::DuplicateCommandKey {
                    key: descriptor.key,
                    first,
                    duplicate: descriptor.command,
                });
            }
            for alias in descriptor.aliases {
                if let Some(first) = aliases.insert(*alias, descriptor.command) {
                    diagnostics.push(EditorCommandCatalogDiagnostic::DuplicateAlias {
                        alias,
                        first,
                        duplicate: descriptor.command,
                    });
                }
            }
            for route_target in descriptor.route_targets() {
                if let Some(first) = route_targets.insert(route_target, descriptor.command) {
                    diagnostics.push(EditorCommandCatalogDiagnostic::DuplicateRouteTarget {
                        route_target,
                        first,
                        duplicate: descriptor.command,
                    });
                }
                if self.command_for_key(route_target) != Some(descriptor.command) {
                    diagnostics.push(
                        EditorCommandCatalogDiagnostic::RouteTargetDoesNotRoundTrip {
                            command: descriptor.command,
                            route_target,
                        },
                    );
                }
            }
            if descriptor.label.trim().is_empty() {
                diagnostics.push(EditorCommandCatalogDiagnostic::EmptyCommandLabel {
                    command: descriptor.command,
                });
            }
            if let EditorCommandAvailabilityRule::StaticDisabled {
                diagnostic_code,
                reason,
            } = descriptor.availability
            {
                if diagnostic_code.trim().is_empty() {
                    diagnostics.push(
                        EditorCommandCatalogDiagnostic::EmptyStaticDisabledDiagnostic {
                            command: descriptor.command,
                        },
                    );
                }
                if reason.trim().is_empty() {
                    diagnostics.push(EditorCommandCatalogDiagnostic::EmptyStaticDisabledReason {
                        command: descriptor.command,
                    });
                }
            }
            if let Some(toolbar_command) = descriptor.toolbar_command()
                && self
                    .descriptor_for_toolbar_command(toolbar_command)
                    .map(|descriptor| descriptor.command)
                    != Some(descriptor.command)
            {
                diagnostics.push(
                    EditorCommandCatalogDiagnostic::ToolbarCommandDoesNotRoundTrip {
                        command: descriptor.command,
                        toolbar_command,
                    },
                );
            }
        }
        for command in KnownEditorCommand::all() {
            if self.descriptor(*command).is_none() {
                diagnostics.push(EditorCommandCatalogDiagnostic::MissingKnownCommand {
                    command: *command,
                });
            }
        }
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }
}

pub fn editor_command_catalog() -> &'static EditorCommandCatalog {
    &EDITOR_COMMAND_CATALOG
}

const EDITOR_COMMAND_CATALOG: EditorCommandCatalog =
    EditorCommandCatalog::new(EDITOR_COMMAND_DESCRIPTORS);

const EDITOR_COMMAND_ORDER: &[KnownEditorCommand] = &[
    KnownEditorCommand::ActivateSelectTool,
    KnownEditorCommand::ActivateTranslateTool,
    KnownEditorCommand::ActivateRotateTool,
    KnownEditorCommand::ActivateScaleTool,
    KnownEditorCommand::ToggleFileMenu,
    KnownEditorCommand::ToggleEditMenu,
    KnownEditorCommand::ToggleWindowMenu,
    KnownEditorCommand::ToggleWorkspaceMenu,
    KnownEditorCommand::Undo,
    KnownEditorCommand::Redo,
    KnownEditorCommand::SaveScene,
    KnownEditorCommand::SaveSceneAs,
    KnownEditorCommand::LoadScene,
    KnownEditorCommand::OpenRecent,
    KnownEditorCommand::EditPreferences,
    KnownEditorCommand::NewWindow,
    KnownEditorCommand::NextWorkspace,
    KnownEditorCommand::PreviousWorkspace,
    KnownEditorCommand::SaveWorkspace,
    KnownEditorCommand::LoadSceneWorkspace,
    KnownEditorCommand::LoadModellingWorkspace,
    KnownEditorCommand::LoadMaterialWorkspace,
    KnownEditorCommand::LoadCustomWorkspace,
    KnownEditorCommand::AddWorkspace,
    KnownEditorCommand::SwitchSceneWorkspace,
    KnownEditorCommand::SwitchModellingWorkspace,
    KnownEditorCommand::SwitchEditorDesignWorkspace,
    KnownEditorCommand::SwitchMaterialWorkspace,
    KnownEditorCommand::CloseSceneWorkspace,
    KnownEditorCommand::CloseModellingWorkspace,
    KnownEditorCommand::CloseEditorDesignWorkspace,
    KnownEditorCommand::CloseMaterialWorkspace,
    KnownEditorCommand::ToggleDebugLogs,
    KnownEditorCommand::ApplySelectedEditorDefinition,
];

const EDITOR_COMMAND_DESCRIPTORS: &[EditorCommandDescriptor] = &[
    descriptor(
        KnownEditorCommand::ActivateSelectTool,
        "editor.tool.select",
        &[],
        "Select",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::ActivateTranslateTool,
        "editor.tool.translate",
        &[],
        "Translate",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::ActivateRotateTool,
        "editor.tool.rotate",
        &[],
        "Rotate",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::ActivateScaleTool,
        "editor.tool.scale",
        &[],
        "Scale",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::ToggleFileMenu,
        "editor.menu.file.toggle",
        &["editor.toolbar.menu.file"],
        "File",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::ToggleEditMenu,
        "editor.menu.edit.toggle",
        &["editor.toolbar.menu.edit"],
        "Edit",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::ToggleWindowMenu,
        "editor.menu.window.toggle",
        &["editor.toolbar.menu.window"],
        "Window",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::ToggleWorkspaceMenu,
        "editor.menu.workspace.toggle",
        &["editor.toolbar.menu.workspace"],
        "Workspace",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::Undo,
        "editor.history.undo",
        &["editor.undo", "editor.toolbar.edit.undo"],
        "Undo",
        EditorCommandAvailabilityRule::CanUndo,
    ),
    descriptor(
        KnownEditorCommand::Redo,
        "editor.history.redo",
        &["editor.redo", "editor.toolbar.edit.redo"],
        "Redo",
        EditorCommandAvailabilityRule::CanRedo,
    ),
    descriptor(
        KnownEditorCommand::SaveScene,
        "editor.scene.save",
        &["editor.toolbar.file.save"],
        "Save",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::SaveSceneAs,
        "editor.scene.save_as",
        &["editor.toolbar.file.save_as"],
        "Save As",
        static_disabled(
            "editor.command.unavailable.save_as",
            "save as is not implemented yet",
        ),
    ),
    descriptor(
        KnownEditorCommand::LoadScene,
        "editor.scene.load",
        &["editor.toolbar.file.open"],
        "Open",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::OpenRecent,
        "editor.scene.open_recent",
        &["editor.toolbar.file.open_recent"],
        "Open Recent",
        static_disabled(
            "editor.command.unavailable.open_recent",
            "recent files are not implemented yet",
        ),
    ),
    descriptor(
        KnownEditorCommand::EditPreferences,
        "editor.preferences.open",
        &["editor.toolbar.edit.preferences"],
        "Preferences",
        static_disabled(
            "editor.command.unavailable.preferences",
            "preferences are not implemented yet",
        ),
    ),
    descriptor(
        KnownEditorCommand::NewWindow,
        "editor.window.new",
        &["editor.toolbar.window.new_window"],
        "New Window",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::NextWorkspace,
        "editor.workspace.next",
        &["editor.toolbar.window.next_workspace"],
        "Next Workspace",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::PreviousWorkspace,
        "editor.workspace.previous",
        &["editor.toolbar.window.previous_workspace"],
        "Previous Workspace",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::SaveWorkspace,
        "editor.workspace.save",
        &["editor.toolbar.window.save_workspace"],
        "Save Workspace",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::LoadSceneWorkspace,
        "editor.workspace.load.scene",
        &["editor.toolbar.window.load_scene_workspace"],
        "Load Scene Workspace",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::LoadModellingWorkspace,
        "editor.workspace.load.modelling",
        &["editor.toolbar.window.load_modelling_workspace"],
        "Load Modelling Workspace",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::LoadMaterialWorkspace,
        "editor.workspace.load.materials",
        &["editor.toolbar.window.load_materials_workspace"],
        "Load Materials Workspace",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::LoadCustomWorkspace,
        "editor.workspace.load.custom",
        &["editor.toolbar.window.load_custom_workspace"],
        "Load Custom Workspace",
        static_disabled(
            "editor.command.unavailable.custom_workspace",
            "custom workspaces are not implemented yet",
        ),
    ),
    descriptor(
        KnownEditorCommand::AddWorkspace,
        "editor.workspace.create",
        &[],
        "Add Workspace",
        static_disabled(
            "editor.command.unavailable.create_workspace",
            "workspace authoring is not implemented yet",
        ),
    ),
    descriptor(
        KnownEditorCommand::SwitchSceneWorkspace,
        "editor.workspace.scene.activate",
        &[],
        "Scene",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::SwitchModellingWorkspace,
        "editor.workspace.modelling.activate",
        &[],
        "Modelling",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::SwitchEditorDesignWorkspace,
        "editor.workspace.editor_design.activate",
        &[],
        "Editor Design",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::SwitchMaterialWorkspace,
        "editor.workspace.materials.activate",
        &[],
        "Materials",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::CloseSceneWorkspace,
        "editor.workspace.scene.close",
        &[],
        "Close Scene Workspace",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::CloseModellingWorkspace,
        "editor.workspace.modelling.close",
        &[],
        "Close Modelling Workspace",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::CloseEditorDesignWorkspace,
        "editor.workspace.editor_design.close",
        &[],
        "Close Editor Design Workspace",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::CloseMaterialWorkspace,
        "editor.workspace.materials.close",
        &[],
        "Close Materials Workspace",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::ToggleDebugLogs,
        "editor.debug.toggle_logs",
        &[],
        "Toggle Debug Logs",
        EditorCommandAvailabilityRule::Always,
    ),
    descriptor(
        KnownEditorCommand::ApplySelectedEditorDefinition,
        "editor.definition.apply_selected",
        &[],
        "Apply Selected Editor Definition",
        EditorCommandAvailabilityRule::Always,
    ),
];

const fn descriptor(
    command: KnownEditorCommand,
    key: &'static str,
    aliases: &'static [&'static str],
    label: &'static str,
    availability: EditorCommandAvailabilityRule,
) -> EditorCommandDescriptor {
    EditorCommandDescriptor {
        command,
        key,
        aliases,
        label,
        availability,
    }
}

const fn static_disabled(
    diagnostic_code: &'static str,
    reason: &'static str,
) -> EditorCommandAvailabilityRule {
    EditorCommandAvailabilityRule::StaticDisabled {
        diagnostic_code,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_command_catalog_has_unique_keys_and_aliases() {
        editor_command_catalog()
            .validate()
            .expect("compiled-in editor command catalog should be unique");
    }

    #[test]
    fn checked_in_toolbar_binding_routes_resolve_to_catalog_descriptors() {
        let bindings = ron::from_str::<editor_definition::EditorDefinitionBindings>(include_str!(
            "../../../../../assets/editor/ui/editor_bindings.ron"
        ))
        .expect("checked-in editor bindings should parse");
        let catalog = editor_command_catalog();

        for route in &bindings.toolbar.routes {
            assert!(
                catalog.is_known_key(route.route.as_str()),
                "toolbar route '{}' should resolve through the command catalog",
                route.route
            );
            assert!(
                catalog.is_known_key(route.ui_route_slot.as_str()),
                "toolbar ui route slot '{}' should resolve through the command catalog",
                route.ui_route_slot
            );
        }

        if let Some(workspaces) = &bindings.toolbar.workspace_catalog {
            for entry in &workspaces.entries {
                let descriptor = catalog
                    .descriptor_for_key(entry.route.as_str())
                    .unwrap_or_else(|| {
                        panic!(
                            "workspace entry route '{}' should resolve through the command catalog",
                            entry.route
                        )
                    });
                assert_eq!(
                    entry.label, descriptor.label,
                    "workspace entry route '{}' should use catalog label '{}'",
                    entry.route, descriptor.label
                );
            }
        }

        for item in &bindings.toolbar.menu_items {
            let descriptor = catalog
                .descriptor_for_key(item.route.as_str())
                .unwrap_or_else(|| {
                    panic!(
                        "menu item '{}' route '{}' should resolve through the command catalog",
                        item.item_id, item.route
                    )
                });
            assert_eq!(
                item.label, descriptor.label,
                "menu item '{}' route '{}' should use catalog label '{}'",
                item.item_id, item.route, descriptor.label
            );
            if let Some(ui_definition::UiAvailabilityBinding::Static(
                ui_definition::UiAvailability::Disabled { reason },
            )) = &item.availability
            {
                let availability = descriptor.availability(EditorCommandAvailabilityContext {
                    can_undo: false,
                    can_redo: false,
                });
                assert_eq!(
                    Some(reason.as_str()),
                    availability.reason(),
                    "menu item '{}' route '{}' should use catalog disabled reason",
                    item.item_id,
                    item.route
                );
            }
        }
    }

    #[test]
    fn stale_new_window_disabled_reason_is_removed_from_catalog() {
        let descriptor = editor_command_catalog()
            .descriptor(KnownEditorCommand::NewWindow)
            .expect("new-window command should exist");

        assert_eq!(
            descriptor.availability(EditorCommandAvailabilityContext {
                can_undo: false,
                can_redo: false,
            }),
            EditorCommandAvailability::Available
        );
    }
}
