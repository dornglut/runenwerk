//! File: domain/editor/editor_core/src/document.rs
//! Purpose: Editor document id and document kind contracts.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocumentId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocumentKind {
    Scene,
    Prefab,
    SdfGraph,
    SdfBrushLayer,
    FieldWorldDefinition,
    FieldProductPreview,
    MaterialGraph,
    Material,
    ProceduralTexture,
    VolumeTexture,
    ProceduralGenerationGraph,
    GameplayGraph,
    GameplayRuleTrigger,
    Ability,
    Quest,
    ParticleGraph,
    ParticleEmitter,
    PhysicsScene,
    PhysicsConfig,
    AnimationClip,
    AnimationGraph,
    Timeline,
    UiLayout,
    Graph,
    Script,
    ForeignMeshReferenceImport,
    AssetCatalog,
    RuntimeDebug,
    WorkspaceDefinition,
    Theme,
    Shortcut,
    Menu,
    CommandBinding,
    PanelRegistry,
    ToolSurfaceDefinition,
}

impl DocumentKind {
    pub fn stable_name(&self) -> &'static str {
        match self {
            Self::Scene => "scene",
            Self::Prefab => "prefab",
            Self::SdfGraph => "sdf_graph",
            Self::SdfBrushLayer => "sdf_brush_layer",
            Self::FieldWorldDefinition => "field_world_definition",
            Self::FieldProductPreview => "field_product_preview",
            Self::MaterialGraph => "material_graph",
            Self::Material => "material",
            Self::ProceduralTexture => "procedural_texture",
            Self::VolumeTexture => "volume_texture",
            Self::ProceduralGenerationGraph => "procedural_generation_graph",
            Self::GameplayGraph => "gameplay_graph",
            Self::GameplayRuleTrigger => "gameplay_rule_trigger",
            Self::Ability => "ability",
            Self::Quest => "quest",
            Self::ParticleGraph => "particle_graph",
            Self::ParticleEmitter => "particle_emitter",
            Self::PhysicsScene => "physics_scene",
            Self::PhysicsConfig => "physics_config",
            Self::AnimationClip => "animation_clip",
            Self::AnimationGraph => "animation_graph",
            Self::Timeline => "timeline",
            Self::UiLayout => "ui_layout",
            Self::Graph => "graph",
            Self::Script => "script",
            Self::ForeignMeshReferenceImport => "foreign_mesh_reference_import",
            Self::AssetCatalog => "asset_catalog",
            Self::RuntimeDebug => "runtime_debug",
            Self::WorkspaceDefinition => "workspace_definition",
            Self::Theme => "theme",
            Self::Shortcut => "shortcut",
            Self::Menu => "menu",
            Self::CommandBinding => "command_binding",
            Self::PanelRegistry => "panel_registry",
            Self::ToolSurfaceDefinition => "tool_surface_definition",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentDescriptor {
    pub id: DocumentId,
    pub kind: DocumentKind,
    pub display_name: String,
    pub is_dirty: bool,
}

impl DocumentDescriptor {
    pub fn new(id: DocumentId, kind: DocumentKind, display_name: impl Into<String>) -> Self {
        Self {
            id,
            kind,
            display_name: display_name.into(),
            is_dirty: false,
        }
    }

    pub fn with_dirty(mut self, is_dirty: bool) -> Self {
        self.is_dirty = is_dirty;
        self
    }
}
