//! File: apps/runenwerk_editor/src/shell/dispatch/mod.rs
//! Purpose: Surface-scoped shell command dispatch entrypoints.

pub(crate) mod entity_table;
pub(crate) mod inspector;
pub(crate) mod outliner;
pub(crate) mod viewport;

use editor_core::EditorMutationError;
use editor_shell::{
    EditorDomainMutation, PanelKind, StructuralCommandTarget, SurfaceSessionMutation,
    ToolSurfaceKind, tool_surface_capability_set, tool_surface_session_retention_class,
};
use ui_surface::{
    SessionRetentionClass, SurfaceCapability, SurfaceCapabilitySet, SurfaceInstanceId,
};

use crate::editor_app::RunenwerkEditorApp;
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportPresentationStateResource, ViewportRenderStateCommandQueueResource,
};
use crate::shell::RunenwerkEditorShellState;

pub(crate) fn dispatch_surface_session_mutation(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    mutation: SurfaceSessionMutation,
) -> Result<(), EditorMutationError> {
    match mutation {
        SurfaceSessionMutation::EntityTable(mutation) => {
            entity_table::dispatch_session_mutation(app, shell_state.as_deref(), target, mutation)
        }
        SurfaceSessionMutation::Inspector(mutation) => {
            inspector::dispatch_session_mutation(app, shell_state.as_deref_mut(), target, mutation)
        }
        SurfaceSessionMutation::Viewport(mutation) => {
            viewport::dispatch_session_mutation(app, shell_state.as_deref(), target, mutation)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_editor_domain_mutation(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    mutation: EditorDomainMutation,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    viewport_render_commands: Option<&mut ViewportRenderStateCommandQueueResource>,
) -> Result<(), EditorMutationError> {
    match mutation {
        EditorDomainMutation::Outliner(mutation) => {
            outliner::dispatch_domain_mutation(app, shell_state, target, mutation)
        }
        EditorDomainMutation::EntityTable(mutation) => {
            entity_table::dispatch_domain_mutation(app, shell_state, target, mutation)
        }
        EditorDomainMutation::Viewport(mutation) => viewport::dispatch_domain_mutation(
            app,
            shell_state,
            target,
            mutation,
            viewport_presentations,
            viewport_observations,
            tool_surface_bindings,
            viewport_render_commands,
        ),
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedSurfaceCommandContract {
    pub(crate) surface_instance_id: SurfaceInstanceId,
    pub(crate) tool_surface_kind: ToolSurfaceKind,
    pub(crate) capabilities: SurfaceCapabilitySet,
    pub(crate) retention_class: SessionRetentionClass,
}

pub(crate) fn resolve_surface_command_contract(
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    fallback_kind: ToolSurfaceKind,
) -> Option<ResolvedSurfaceCommandContract> {
    let tool_surface_id = target.active_tool_surface?;
    let resolved_kind = if let Some(state) = shell_state {
        let surface = state.workspace_state().tool_surface(tool_surface_id)?;
        surface.tool_surface_kind
    } else {
        fallback_kind
    };
    Some(ResolvedSurfaceCommandContract {
        surface_instance_id: SurfaceInstanceId::new(tool_surface_id.raw()),
        tool_surface_kind: resolved_kind,
        capabilities: tool_surface_capability_set(resolved_kind),
        retention_class: tool_surface_session_retention_class(resolved_kind),
    })
}

pub(crate) fn tool_surface_kind_label(kind: ToolSurfaceKind) -> &'static str {
    match kind {
        ToolSurfaceKind::Outliner => "outliner",
        ToolSurfaceKind::EntityTable => "entity_table",
        ToolSurfaceKind::Viewport => "viewport",
        ToolSurfaceKind::Inspector => "inspector",
        ToolSurfaceKind::Console => "console",
        ToolSurfaceKind::EditorDesignOutliner => "editor_design_outliner",
        ToolSurfaceKind::UiHierarchy => "ui_hierarchy",
        ToolSurfaceKind::UiCanvas => "ui_canvas",
        ToolSurfaceKind::StyleInspector => "style_inspector",
        ToolSurfaceKind::Bindings => "bindings",
        ToolSurfaceKind::DockLayoutPreview => "dock_layout_preview",
        ToolSurfaceKind::ThemeEditor => "theme_editor",
        ToolSurfaceKind::ShortcutEditor => "shortcut_editor",
        ToolSurfaceKind::MenuEditor => "menu_editor",
        ToolSurfaceKind::DefinitionValidation => "definition_validation",
        ToolSurfaceKind::CommandDiff => "command_diff",
        ToolSurfaceKind::Placeholder => "placeholder",
    }
}

pub(crate) fn panel_kind_for_tool_surface_kind(kind: ToolSurfaceKind) -> PanelKind {
    match kind {
        ToolSurfaceKind::Outliner => PanelKind::Outliner,
        ToolSurfaceKind::EntityTable => PanelKind::EntityTable,
        ToolSurfaceKind::Viewport => PanelKind::Viewport,
        ToolSurfaceKind::Inspector => PanelKind::Inspector,
        ToolSurfaceKind::Console => PanelKind::Console,
        ToolSurfaceKind::EditorDesignOutliner => PanelKind::EditorDesignOutliner,
        ToolSurfaceKind::UiHierarchy => PanelKind::UiHierarchy,
        ToolSurfaceKind::UiCanvas => PanelKind::UiCanvas,
        ToolSurfaceKind::StyleInspector => PanelKind::StyleInspector,
        ToolSurfaceKind::Bindings => PanelKind::Bindings,
        ToolSurfaceKind::DockLayoutPreview => PanelKind::DockLayoutPreview,
        ToolSurfaceKind::ThemeEditor => PanelKind::ThemeEditor,
        ToolSurfaceKind::ShortcutEditor => PanelKind::ShortcutEditor,
        ToolSurfaceKind::MenuEditor => PanelKind::MenuEditor,
        ToolSurfaceKind::DefinitionValidation => PanelKind::DefinitionValidation,
        ToolSurfaceKind::CommandDiff => PanelKind::CommandDiff,
        ToolSurfaceKind::Placeholder => PanelKind::Placeholder,
    }
}

pub(crate) fn surface_capability_label(capability: SurfaceCapability) -> &'static str {
    match capability {
        SurfaceCapability::Observe => "observe",
        SurfaceCapability::Interact => "interact",
        SurfaceCapability::RequestMutation => "request_mutation",
        SurfaceCapability::Ratify => "ratify",
    }
}
