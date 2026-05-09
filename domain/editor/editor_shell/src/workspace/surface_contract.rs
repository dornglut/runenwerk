//! File: domain/editor/editor_shell/src/workspace/surface_contract.rs
//! Purpose: Editor-shell tool-surface mapping into ui_surface mount contracts.

use ui_surface::{
    MountedSurfaceInstance, SessionRetentionClass, SurfaceCapabilitySet, SurfaceDefinition,
    SurfaceDefinitionId, SurfaceHostInstanceId, SurfaceInstanceId,
};

use crate::{PanelKind, ToolSurfaceKind, ToolSurfaceMount, ToolSurfaceState, WorkspaceState};

pub const OUTLINER_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(1);
pub const VIEWPORT_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(2);
pub const INSPECTOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(3);
pub const CONSOLE_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(4);
pub const PLACEHOLDER_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(5);
pub const ENTITY_TABLE_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(6);
pub const EDITOR_DESIGN_OUTLINER_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(7);
pub const UI_HIERARCHY_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(8);
pub const UI_CANVAS_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(9);
pub const STYLE_INSPECTOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(10);
pub const BINDINGS_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(11);
pub const DOCK_LAYOUT_PREVIEW_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(12);
pub const THEME_EDITOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(13);
pub const SHORTCUT_EDITOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(14);
pub const MENU_EDITOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(15);
pub const DEFINITION_VALIDATION_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(16);
pub const COMMAND_DIFF_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(17);
pub const ASSET_BROWSER_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(18);
pub const IMPORT_INSPECTOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(19);
pub const FIELD_PRODUCT_VIEWER_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(20);
pub const SDF_BRUSH_BROWSER_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(21);
pub const GRAPH_CANVAS_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(22);
pub const DIAGNOSTICS_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(23);
pub const RUNTIME_DEBUG_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(24);
pub const FIELD_LAYER_STACK_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(25);
pub const SDF_GRAPH_CANVAS_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(26);
pub const MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(27);
pub const MATERIAL_INSPECTOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(28);
pub const MATERIAL_PREVIEW_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(29);
pub const TEXTURE_VIEWER_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(30);
pub const VOLUME_TEXTURE_VIEWER_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(31);
pub const PROCGEN_GRAPH_CANVAS_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(32);
pub const PROCGEN_PREVIEW_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(33);
pub const GAMEPLAY_GRAPH_CANVAS_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(34);
pub const GAMEPLAY_COMPILER_DIAGNOSTICS_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(35);
pub const PARTICLE_GRAPH_CANVAS_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(36);
pub const PARTICLE_PREVIEW_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(37);
pub const PHYSICS_AUTHORING_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(38);
pub const PHYSICS_DEBUG_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(39);
pub const TIMELINE_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(40);
pub const CURVE_EDITOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(41);
pub const ANIMATION_GRAPH_CANVAS_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(42);
pub const SIMULATION_PREVIEW_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(43);
pub const SIMULATION_DIAGNOSTICS_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(44);

pub fn editor_surface_definitions() -> Vec<SurfaceDefinition> {
    vec![
        SurfaceDefinition::new(
            OUTLINER_SURFACE_DEFINITION_ID,
            "editor.tool_surface.outliner",
            "Outliner",
        ),
        SurfaceDefinition::new(
            VIEWPORT_SURFACE_DEFINITION_ID,
            "editor.tool_surface.viewport",
            "Viewport",
        ),
        SurfaceDefinition::new(
            INSPECTOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.inspector",
            "Inspector",
        ),
        SurfaceDefinition::new(
            CONSOLE_SURFACE_DEFINITION_ID,
            "editor.tool_surface.console",
            "Console",
        ),
        SurfaceDefinition::new(
            PLACEHOLDER_SURFACE_DEFINITION_ID,
            "editor.tool_surface.placeholder",
            "Placeholder",
        ),
        SurfaceDefinition::new(
            ENTITY_TABLE_SURFACE_DEFINITION_ID,
            "editor.tool_surface.entity_table",
            "Entity Table",
        ),
        SurfaceDefinition::new(
            EDITOR_DESIGN_OUTLINER_SURFACE_DEFINITION_ID,
            "editor.tool_surface.editor_design_outliner",
            "Definition Outliner",
        ),
        SurfaceDefinition::new(
            UI_HIERARCHY_SURFACE_DEFINITION_ID,
            "editor.tool_surface.ui_hierarchy",
            "UI Hierarchy",
        ),
        SurfaceDefinition::new(
            UI_CANVAS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.ui_canvas",
            "UI Canvas",
        ),
        SurfaceDefinition::new(
            STYLE_INSPECTOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.style_inspector",
            "Style Inspector",
        ),
        SurfaceDefinition::new(
            BINDINGS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.bindings",
            "Bindings",
        ),
        SurfaceDefinition::new(
            DOCK_LAYOUT_PREVIEW_SURFACE_DEFINITION_ID,
            "editor.tool_surface.dock_layout_preview",
            "Dock Layout Preview",
        ),
        SurfaceDefinition::new(
            THEME_EDITOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.theme_editor",
            "Theme Editor",
        ),
        SurfaceDefinition::new(
            SHORTCUT_EDITOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.shortcut_editor",
            "Shortcut Editor",
        ),
        SurfaceDefinition::new(
            MENU_EDITOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.menu_editor",
            "Menu Editor",
        ),
        SurfaceDefinition::new(
            DEFINITION_VALIDATION_SURFACE_DEFINITION_ID,
            "editor.tool_surface.definition_validation",
            "Definition Validation",
        ),
        SurfaceDefinition::new(
            COMMAND_DIFF_SURFACE_DEFINITION_ID,
            "editor.tool_surface.command_diff",
            "Command Diff",
        ),
        SurfaceDefinition::new(
            ASSET_BROWSER_SURFACE_DEFINITION_ID,
            "editor.tool_surface.asset_browser",
            "Asset Browser",
        ),
        SurfaceDefinition::new(
            IMPORT_INSPECTOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.import_inspector",
            "Import Inspector",
        ),
        SurfaceDefinition::new(
            FIELD_PRODUCT_VIEWER_SURFACE_DEFINITION_ID,
            "editor.tool_surface.field_product_viewer",
            "Field Product Viewer",
        ),
        SurfaceDefinition::new(
            SDF_BRUSH_BROWSER_SURFACE_DEFINITION_ID,
            "editor.tool_surface.sdf_brush_browser",
            "SDF Brush Browser",
        ),
        SurfaceDefinition::new(
            GRAPH_CANVAS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.graph_canvas",
            "Graph Canvas",
        ),
        SurfaceDefinition::new(
            DIAGNOSTICS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.diagnostics",
            "Diagnostics",
        ),
        SurfaceDefinition::new(
            RUNTIME_DEBUG_SURFACE_DEFINITION_ID,
            "editor.tool_surface.runtime_debug",
            "Runtime Debug",
        ),
        SurfaceDefinition::new(
            FIELD_LAYER_STACK_SURFACE_DEFINITION_ID,
            "editor.tool_surface.field_layer_stack",
            "Field Layer Stack",
        ),
        SurfaceDefinition::new(
            SDF_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.sdf_graph_canvas",
            "SDF Graph Canvas",
        ),
        SurfaceDefinition::new(
            MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.material_graph_canvas",
            "Material Graph Canvas",
        ),
        SurfaceDefinition::new(
            MATERIAL_INSPECTOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.material_inspector",
            "Material Inspector",
        ),
        SurfaceDefinition::new(
            MATERIAL_PREVIEW_SURFACE_DEFINITION_ID,
            "editor.tool_surface.material_preview",
            "Material Preview",
        ),
        SurfaceDefinition::new(
            TEXTURE_VIEWER_SURFACE_DEFINITION_ID,
            "editor.tool_surface.texture_viewer",
            "Texture Viewer",
        ),
        SurfaceDefinition::new(
            VOLUME_TEXTURE_VIEWER_SURFACE_DEFINITION_ID,
            "editor.tool_surface.volume_texture_viewer",
            "Volume Texture Viewer",
        ),
        SurfaceDefinition::new(
            PROCGEN_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.procgen_graph_canvas",
            "Procgen Graph Canvas",
        ),
        SurfaceDefinition::new(
            PROCGEN_PREVIEW_SURFACE_DEFINITION_ID,
            "editor.tool_surface.procgen_preview",
            "Procgen Preview",
        ),
        SurfaceDefinition::new(
            GAMEPLAY_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.gameplay_graph_canvas",
            "Gameplay Graph Canvas",
        ),
        SurfaceDefinition::new(
            GAMEPLAY_COMPILER_DIAGNOSTICS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.gameplay_compiler_diagnostics",
            "Gameplay Compiler Diagnostics",
        ),
        SurfaceDefinition::new(
            PARTICLE_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.particle_graph_canvas",
            "Particle Graph Canvas",
        ),
        SurfaceDefinition::new(
            PARTICLE_PREVIEW_SURFACE_DEFINITION_ID,
            "editor.tool_surface.particle_preview",
            "Particle Preview",
        ),
        SurfaceDefinition::new(
            PHYSICS_AUTHORING_SURFACE_DEFINITION_ID,
            "editor.tool_surface.physics_authoring",
            "Physics Authoring",
        ),
        SurfaceDefinition::new(
            PHYSICS_DEBUG_SURFACE_DEFINITION_ID,
            "editor.tool_surface.physics_debug",
            "Physics Debug",
        ),
        SurfaceDefinition::new(
            TIMELINE_SURFACE_DEFINITION_ID,
            "editor.tool_surface.timeline",
            "Timeline",
        ),
        SurfaceDefinition::new(
            CURVE_EDITOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.curve_editor",
            "Curve Editor",
        ),
        SurfaceDefinition::new(
            ANIMATION_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.animation_graph_canvas",
            "Animation Graph Canvas",
        ),
        SurfaceDefinition::new(
            SIMULATION_PREVIEW_SURFACE_DEFINITION_ID,
            "editor.tool_surface.simulation_preview",
            "Simulation Preview",
        ),
        SurfaceDefinition::new(
            SIMULATION_DIAGNOSTICS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.simulation_diagnostics",
            "Simulation Diagnostics",
        ),
    ]
}

pub fn tool_surface_definition_id(kind: ToolSurfaceKind) -> SurfaceDefinitionId {
    match kind {
        ToolSurfaceKind::Outliner => OUTLINER_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::EntityTable => ENTITY_TABLE_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Viewport => VIEWPORT_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Inspector => INSPECTOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Console => CONSOLE_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::EditorDesignOutliner => EDITOR_DESIGN_OUTLINER_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::UiHierarchy => UI_HIERARCHY_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::UiCanvas => UI_CANVAS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::StyleInspector => STYLE_INSPECTOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Bindings => BINDINGS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::DockLayoutPreview => DOCK_LAYOUT_PREVIEW_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::ThemeEditor => THEME_EDITOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::ShortcutEditor => SHORTCUT_EDITOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::MenuEditor => MENU_EDITOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::DefinitionValidation => DEFINITION_VALIDATION_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::CommandDiff => COMMAND_DIFF_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::AssetBrowser => ASSET_BROWSER_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::ImportInspector => IMPORT_INSPECTOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::FieldProductViewer => FIELD_PRODUCT_VIEWER_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::SdfBrushBrowser => SDF_BRUSH_BROWSER_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::GraphCanvas => GRAPH_CANVAS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Diagnostics => DIAGNOSTICS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::RuntimeDebug => RUNTIME_DEBUG_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::FieldLayerStack => FIELD_LAYER_STACK_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::SdfGraphCanvas => SDF_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::MaterialGraphCanvas => MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::MaterialInspector => MATERIAL_INSPECTOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::MaterialPreview => MATERIAL_PREVIEW_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::TextureViewer => TEXTURE_VIEWER_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::VolumeTextureViewer => VOLUME_TEXTURE_VIEWER_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::ProcgenGraphCanvas => PROCGEN_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::ProcgenPreview => PROCGEN_PREVIEW_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::GameplayGraphCanvas => GAMEPLAY_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::GameplayCompilerDiagnostics => {
            GAMEPLAY_COMPILER_DIAGNOSTICS_SURFACE_DEFINITION_ID
        }
        ToolSurfaceKind::ParticleGraphCanvas => PARTICLE_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::ParticlePreview => PARTICLE_PREVIEW_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::PhysicsAuthoring => PHYSICS_AUTHORING_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::PhysicsDebug => PHYSICS_DEBUG_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Timeline => TIMELINE_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::CurveEditor => CURVE_EDITOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::AnimationGraphCanvas => ANIMATION_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::SimulationPreview => SIMULATION_PREVIEW_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::SimulationDiagnostics => SIMULATION_DIAGNOSTICS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Placeholder => PLACEHOLDER_SURFACE_DEFINITION_ID,
    }
}

pub fn tool_surface_kind_definition_key(kind: ToolSurfaceKind) -> &'static str {
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
        ToolSurfaceKind::AssetBrowser => "asset_browser",
        ToolSurfaceKind::ImportInspector => "import_inspector",
        ToolSurfaceKind::FieldProductViewer => "field_product_viewer",
        ToolSurfaceKind::SdfBrushBrowser => "sdf_brush_browser",
        ToolSurfaceKind::GraphCanvas => "graph_canvas",
        ToolSurfaceKind::Diagnostics => "diagnostics",
        ToolSurfaceKind::RuntimeDebug => "runtime_debug",
        ToolSurfaceKind::FieldLayerStack => "field_layer_stack",
        ToolSurfaceKind::SdfGraphCanvas => "sdf_graph_canvas",
        ToolSurfaceKind::MaterialGraphCanvas => "material_graph_canvas",
        ToolSurfaceKind::MaterialInspector => "material_inspector",
        ToolSurfaceKind::MaterialPreview => "material_preview",
        ToolSurfaceKind::TextureViewer => "texture_viewer",
        ToolSurfaceKind::VolumeTextureViewer => "volume_texture_viewer",
        ToolSurfaceKind::ProcgenGraphCanvas => "procgen_graph_canvas",
        ToolSurfaceKind::ProcgenPreview => "procgen_preview",
        ToolSurfaceKind::GameplayGraphCanvas => "gameplay_graph_canvas",
        ToolSurfaceKind::GameplayCompilerDiagnostics => "gameplay_compiler_diagnostics",
        ToolSurfaceKind::ParticleGraphCanvas => "particle_graph_canvas",
        ToolSurfaceKind::ParticlePreview => "particle_preview",
        ToolSurfaceKind::PhysicsAuthoring => "physics_authoring",
        ToolSurfaceKind::PhysicsDebug => "physics_debug",
        ToolSurfaceKind::Timeline => "timeline",
        ToolSurfaceKind::CurveEditor => "curve_editor",
        ToolSurfaceKind::AnimationGraphCanvas => "animation_graph_canvas",
        ToolSurfaceKind::SimulationPreview => "simulation_preview",
        ToolSurfaceKind::SimulationDiagnostics => "simulation_diagnostics",
        ToolSurfaceKind::Placeholder => "placeholder",
    }
}

pub fn panel_kind_definition_key(kind: PanelKind) -> &'static str {
    match kind {
        PanelKind::Outliner => "outliner",
        PanelKind::EntityTable => "entity_table",
        PanelKind::Viewport => "viewport",
        PanelKind::Inspector => "inspector",
        PanelKind::Console => "console",
        PanelKind::EditorDesignOutliner => "editor_design_outliner",
        PanelKind::UiHierarchy => "ui_hierarchy",
        PanelKind::UiCanvas => "ui_canvas",
        PanelKind::StyleInspector => "style_inspector",
        PanelKind::Bindings => "bindings",
        PanelKind::DockLayoutPreview => "dock_layout_preview",
        PanelKind::ThemeEditor => "theme_editor",
        PanelKind::ShortcutEditor => "shortcut_editor",
        PanelKind::MenuEditor => "menu_editor",
        PanelKind::DefinitionValidation => "definition_validation",
        PanelKind::CommandDiff => "command_diff",
        PanelKind::AssetBrowser => "asset_browser",
        PanelKind::ImportInspector => "import_inspector",
        PanelKind::FieldProductViewer => "field_product_viewer",
        PanelKind::SdfBrushBrowser => "sdf_brush_browser",
        PanelKind::GraphCanvas => "graph_canvas",
        PanelKind::Diagnostics => "diagnostics",
        PanelKind::RuntimeDebug => "runtime_debug",
        PanelKind::FieldLayerStack => "field_layer_stack",
        PanelKind::SdfGraphCanvas => "sdf_graph_canvas",
        PanelKind::MaterialGraphCanvas => "material_graph_canvas",
        PanelKind::MaterialInspector => "material_inspector",
        PanelKind::MaterialPreview => "material_preview",
        PanelKind::TextureViewer => "texture_viewer",
        PanelKind::VolumeTextureViewer => "volume_texture_viewer",
        PanelKind::ProcgenGraphCanvas => "procgen_graph_canvas",
        PanelKind::ProcgenPreview => "procgen_preview",
        PanelKind::GameplayGraphCanvas => "gameplay_graph_canvas",
        PanelKind::GameplayCompilerDiagnostics => "gameplay_compiler_diagnostics",
        PanelKind::ParticleGraphCanvas => "particle_graph_canvas",
        PanelKind::ParticlePreview => "particle_preview",
        PanelKind::PhysicsAuthoring => "physics_authoring",
        PanelKind::PhysicsDebug => "physics_debug",
        PanelKind::Timeline => "timeline",
        PanelKind::CurveEditor => "curve_editor",
        PanelKind::AnimationGraphCanvas => "animation_graph_canvas",
        PanelKind::SimulationPreview => "simulation_preview",
        PanelKind::SimulationDiagnostics => "simulation_diagnostics",
        PanelKind::Placeholder => "placeholder",
    }
}

pub fn tool_surface_kind_from_definition_key(value: &str) -> Option<ToolSurfaceKind> {
    match value {
        "outliner" => Some(ToolSurfaceKind::Outliner),
        "entity_table" => Some(ToolSurfaceKind::EntityTable),
        "viewport" => Some(ToolSurfaceKind::Viewport),
        "inspector" => Some(ToolSurfaceKind::Inspector),
        "console" => Some(ToolSurfaceKind::Console),
        "editor_design_outliner" => Some(ToolSurfaceKind::EditorDesignOutliner),
        "ui_hierarchy" => Some(ToolSurfaceKind::UiHierarchy),
        "ui_canvas" => Some(ToolSurfaceKind::UiCanvas),
        "style_inspector" => Some(ToolSurfaceKind::StyleInspector),
        "bindings" => Some(ToolSurfaceKind::Bindings),
        "dock_layout_preview" => Some(ToolSurfaceKind::DockLayoutPreview),
        "theme_editor" => Some(ToolSurfaceKind::ThemeEditor),
        "shortcut_editor" => Some(ToolSurfaceKind::ShortcutEditor),
        "menu_editor" => Some(ToolSurfaceKind::MenuEditor),
        "definition_validation" => Some(ToolSurfaceKind::DefinitionValidation),
        "command_diff" => Some(ToolSurfaceKind::CommandDiff),
        "asset_browser" => Some(ToolSurfaceKind::AssetBrowser),
        "import_inspector" => Some(ToolSurfaceKind::ImportInspector),
        "field_product_viewer" => Some(ToolSurfaceKind::FieldProductViewer),
        "sdf_brush_browser" => Some(ToolSurfaceKind::SdfBrushBrowser),
        "graph_canvas" => Some(ToolSurfaceKind::GraphCanvas),
        "diagnostics" => Some(ToolSurfaceKind::Diagnostics),
        "runtime_debug" => Some(ToolSurfaceKind::RuntimeDebug),
        "field_layer_stack" => Some(ToolSurfaceKind::FieldLayerStack),
        "sdf_graph_canvas" => Some(ToolSurfaceKind::SdfGraphCanvas),
        "material_graph_canvas" => Some(ToolSurfaceKind::MaterialGraphCanvas),
        "material_inspector" => Some(ToolSurfaceKind::MaterialInspector),
        "material_preview" => Some(ToolSurfaceKind::MaterialPreview),
        "texture_viewer" => Some(ToolSurfaceKind::TextureViewer),
        "volume_texture_viewer" => Some(ToolSurfaceKind::VolumeTextureViewer),
        "procgen_graph_canvas" => Some(ToolSurfaceKind::ProcgenGraphCanvas),
        "procgen_preview" => Some(ToolSurfaceKind::ProcgenPreview),
        "gameplay_graph_canvas" => Some(ToolSurfaceKind::GameplayGraphCanvas),
        "gameplay_compiler_diagnostics" => Some(ToolSurfaceKind::GameplayCompilerDiagnostics),
        "particle_graph_canvas" => Some(ToolSurfaceKind::ParticleGraphCanvas),
        "particle_preview" => Some(ToolSurfaceKind::ParticlePreview),
        "physics_authoring" => Some(ToolSurfaceKind::PhysicsAuthoring),
        "physics_debug" => Some(ToolSurfaceKind::PhysicsDebug),
        "timeline" => Some(ToolSurfaceKind::Timeline),
        "curve_editor" => Some(ToolSurfaceKind::CurveEditor),
        "animation_graph_canvas" => Some(ToolSurfaceKind::AnimationGraphCanvas),
        "simulation_preview" => Some(ToolSurfaceKind::SimulationPreview),
        "simulation_diagnostics" => Some(ToolSurfaceKind::SimulationDiagnostics),
        "placeholder" => Some(ToolSurfaceKind::Placeholder),
        _ => None,
    }
}

pub fn panel_kind_for_tool_surface_kind(kind: ToolSurfaceKind) -> PanelKind {
    kind.panel_kind()
}

pub fn tool_surface_capability_set(kind: ToolSurfaceKind) -> SurfaceCapabilitySet {
    match kind {
        ToolSurfaceKind::Outliner => SurfaceCapabilitySet::new(true, true, true, false),
        ToolSurfaceKind::EntityTable => SurfaceCapabilitySet::new(true, true, true, false),
        ToolSurfaceKind::Viewport => SurfaceCapabilitySet::new(true, true, true, false),
        ToolSurfaceKind::Inspector => SurfaceCapabilitySet::new(true, true, true, false),
        ToolSurfaceKind::Console => SurfaceCapabilitySet::new(true, true, false, false),
        ToolSurfaceKind::EditorDesignOutliner
        | ToolSurfaceKind::UiHierarchy
        | ToolSurfaceKind::UiCanvas
        | ToolSurfaceKind::StyleInspector
        | ToolSurfaceKind::Bindings
        | ToolSurfaceKind::DockLayoutPreview
        | ToolSurfaceKind::ThemeEditor
        | ToolSurfaceKind::ShortcutEditor
        | ToolSurfaceKind::MenuEditor
        | ToolSurfaceKind::DefinitionValidation
        | ToolSurfaceKind::CommandDiff => SurfaceCapabilitySet::new(true, true, true, true),
        ToolSurfaceKind::AssetBrowser
        | ToolSurfaceKind::ImportInspector
        | ToolSurfaceKind::FieldProductViewer
        | ToolSurfaceKind::SdfBrushBrowser
        | ToolSurfaceKind::GraphCanvas
        | ToolSurfaceKind::FieldLayerStack
        | ToolSurfaceKind::SdfGraphCanvas
        | ToolSurfaceKind::MaterialGraphCanvas
        | ToolSurfaceKind::MaterialInspector
        | ToolSurfaceKind::MaterialPreview
        | ToolSurfaceKind::TextureViewer
        | ToolSurfaceKind::VolumeTextureViewer
        | ToolSurfaceKind::ProcgenGraphCanvas
        | ToolSurfaceKind::ProcgenPreview
        | ToolSurfaceKind::GameplayGraphCanvas
        | ToolSurfaceKind::ParticleGraphCanvas
        | ToolSurfaceKind::ParticlePreview
        | ToolSurfaceKind::PhysicsAuthoring
        | ToolSurfaceKind::Timeline
        | ToolSurfaceKind::CurveEditor
        | ToolSurfaceKind::AnimationGraphCanvas
        | ToolSurfaceKind::SimulationPreview => SurfaceCapabilitySet::new(true, true, true, false),
        ToolSurfaceKind::Diagnostics
        | ToolSurfaceKind::RuntimeDebug
        | ToolSurfaceKind::GameplayCompilerDiagnostics
        | ToolSurfaceKind::PhysicsDebug
        | ToolSurfaceKind::SimulationDiagnostics => {
            SurfaceCapabilitySet::new(true, true, false, false)
        }
        ToolSurfaceKind::Placeholder => SurfaceCapabilitySet::new(true, false, false, false),
    }
}

pub fn tool_surface_session_retention_class(kind: ToolSurfaceKind) -> SessionRetentionClass {
    match kind {
        ToolSurfaceKind::Outliner => SessionRetentionClass::Restorable,
        ToolSurfaceKind::EntityTable => SessionRetentionClass::Restorable,
        ToolSurfaceKind::Viewport => SessionRetentionClass::Restorable,
        ToolSurfaceKind::Inspector => SessionRetentionClass::Persistent,
        ToolSurfaceKind::Console => SessionRetentionClass::Ephemeral,
        ToolSurfaceKind::EditorDesignOutliner
        | ToolSurfaceKind::UiHierarchy
        | ToolSurfaceKind::UiCanvas
        | ToolSurfaceKind::StyleInspector
        | ToolSurfaceKind::Bindings
        | ToolSurfaceKind::DockLayoutPreview
        | ToolSurfaceKind::ThemeEditor
        | ToolSurfaceKind::ShortcutEditor
        | ToolSurfaceKind::MenuEditor
        | ToolSurfaceKind::DefinitionValidation
        | ToolSurfaceKind::CommandDiff
        | ToolSurfaceKind::AssetBrowser
        | ToolSurfaceKind::ImportInspector
        | ToolSurfaceKind::FieldProductViewer
        | ToolSurfaceKind::SdfBrushBrowser
        | ToolSurfaceKind::GraphCanvas
        | ToolSurfaceKind::Diagnostics
        | ToolSurfaceKind::RuntimeDebug
        | ToolSurfaceKind::FieldLayerStack
        | ToolSurfaceKind::SdfGraphCanvas
        | ToolSurfaceKind::MaterialGraphCanvas
        | ToolSurfaceKind::MaterialInspector
        | ToolSurfaceKind::MaterialPreview
        | ToolSurfaceKind::TextureViewer
        | ToolSurfaceKind::VolumeTextureViewer
        | ToolSurfaceKind::ProcgenGraphCanvas
        | ToolSurfaceKind::ProcgenPreview
        | ToolSurfaceKind::GameplayGraphCanvas
        | ToolSurfaceKind::GameplayCompilerDiagnostics
        | ToolSurfaceKind::ParticleGraphCanvas
        | ToolSurfaceKind::ParticlePreview
        | ToolSurfaceKind::PhysicsAuthoring
        | ToolSurfaceKind::PhysicsDebug
        | ToolSurfaceKind::Timeline
        | ToolSurfaceKind::CurveEditor
        | ToolSurfaceKind::AnimationGraphCanvas
        | ToolSurfaceKind::SimulationPreview
        | ToolSurfaceKind::SimulationDiagnostics => SessionRetentionClass::Restorable,
        ToolSurfaceKind::Placeholder => SessionRetentionClass::Ephemeral,
    }
}

pub fn mounted_surface_instance(tool_surface: ToolSurfaceState) -> Option<MountedSurfaceInstance> {
    let ToolSurfaceMount::Mounted { panel_id } = tool_surface.mount else {
        return None;
    };

    Some(MountedSurfaceInstance::new(
        SurfaceInstanceId::new(tool_surface.id.raw()),
        tool_surface_definition_id(tool_surface.tool_surface_kind),
        SurfaceHostInstanceId::new(panel_id.raw()),
    ))
}

pub fn mounted_surface_instances(
    workspace_state: &WorkspaceState,
) -> impl Iterator<Item = MountedSurfaceInstance> + '_ {
    workspace_state
        .tool_surfaces()
        .copied()
        .filter_map(mounted_surface_instance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WorkspaceId, WorkspaceIdentityAllocator};

    #[test]
    fn tool_surface_kind_maps_to_stable_definition_identity() {
        assert_eq!(
            tool_surface_definition_id(ToolSurfaceKind::Viewport),
            VIEWPORT_SURFACE_DEFINITION_ID
        );
        assert_eq!(
            tool_surface_definition_id(ToolSurfaceKind::Outliner),
            OUTLINER_SURFACE_DEFINITION_ID
        );
    }

    #[test]
    fn mounted_surface_instances_follow_workspace_mount_state() {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let workspace = WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);

        let mounted = mounted_surface_instances(&workspace).collect::<Vec<_>>();

        assert_eq!(workspace_id, WorkspaceId::try_from_raw(1).unwrap());
        assert_eq!(mounted.len(), 5);
        assert!(
            mounted
                .iter()
                .any(|instance| instance.definition_id == VIEWPORT_SURFACE_DEFINITION_ID)
        );
    }

    #[test]
    fn tool_surface_capabilities_are_explicit_per_surface_kind() {
        let outliner_caps = tool_surface_capability_set(ToolSurfaceKind::Outliner);
        let entity_table_caps = tool_surface_capability_set(ToolSurfaceKind::EntityTable);
        let console_caps = tool_surface_capability_set(ToolSurfaceKind::Console);
        let placeholder_caps = tool_surface_capability_set(ToolSurfaceKind::Placeholder);

        assert!(outliner_caps.allows(ui_surface::SurfaceCapability::Observe));
        assert!(outliner_caps.allows(ui_surface::SurfaceCapability::Interact));
        assert!(outliner_caps.allows(ui_surface::SurfaceCapability::RequestMutation));
        assert!(!outliner_caps.allows(ui_surface::SurfaceCapability::Ratify));

        assert!(entity_table_caps.allows(ui_surface::SurfaceCapability::Observe));
        assert!(entity_table_caps.allows(ui_surface::SurfaceCapability::Interact));
        assert!(entity_table_caps.allows(ui_surface::SurfaceCapability::RequestMutation));
        assert!(!entity_table_caps.allows(ui_surface::SurfaceCapability::Ratify));

        assert!(console_caps.allows(ui_surface::SurfaceCapability::Observe));
        assert!(console_caps.allows(ui_surface::SurfaceCapability::Interact));
        assert!(!console_caps.allows(ui_surface::SurfaceCapability::RequestMutation));
        assert!(!console_caps.allows(ui_surface::SurfaceCapability::Ratify));

        assert!(placeholder_caps.allows(ui_surface::SurfaceCapability::Observe));
        assert!(!placeholder_caps.allows(ui_surface::SurfaceCapability::Interact));
        assert!(!placeholder_caps.allows(ui_surface::SurfaceCapability::RequestMutation));
        assert!(!placeholder_caps.allows(ui_surface::SurfaceCapability::Ratify));
    }

    #[test]
    fn tool_surface_retention_class_is_explicit_per_surface_kind() {
        assert_eq!(
            tool_surface_session_retention_class(ToolSurfaceKind::Outliner),
            SessionRetentionClass::Restorable,
        );
        assert_eq!(
            tool_surface_session_retention_class(ToolSurfaceKind::Viewport),
            SessionRetentionClass::Restorable,
        );
        assert_eq!(
            tool_surface_session_retention_class(ToolSurfaceKind::EntityTable),
            SessionRetentionClass::Restorable,
        );
        assert_eq!(
            tool_surface_session_retention_class(ToolSurfaceKind::Inspector),
            SessionRetentionClass::Persistent,
        );
        assert_eq!(
            tool_surface_session_retention_class(ToolSurfaceKind::Console),
            SessionRetentionClass::Ephemeral,
        );
        assert_eq!(
            tool_surface_session_retention_class(ToolSurfaceKind::Placeholder),
            SessionRetentionClass::Ephemeral,
        );
    }
}
