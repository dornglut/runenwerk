use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Scene,
    Prefab,
    SdfGraph,
    SdfBrushLayer,
    FieldWorldDefinition,
    WorldEditLog,
    FieldMaterialChannelSet,
    FormedFieldProduct,
    WorldSdfChunkPageArtifact,
    ClipmapBrickmapProduct,
    MaterialGraph,
    Material,
    ProceduralMaterial,
    ProceduralTexture,
    Texture2D,
    Texture3DVolume,
    GameplayGraph,
    GameplayRuleTrigger,
    GameplayAbility,
    GameplayQuest,
    GameplayAtrIrProduct,
    GameplayEcsLoweringProduct,
    ParticleGraph,
    ParticleEmitter,
    PhysicsConfig,
    AnimationClip,
    AnimationGraph,
    ProcgenGraph,
    UiLayout,
    UiDefinition,
    Graph,
    Script,
    Shader,
    Theme,
    Menu,
    Shortcut,
    WorkspaceDefinition,
    EditorDefinition,
    DiagnosticsCapture,
    ForeignMeshReferenceSource,
    ForeignMeshReferenceArtifact,
}

impl AssetKind {
    pub const fn is_primary_world_authoring(self) -> bool {
        matches!(
            self,
            Self::SdfGraph
                | Self::SdfBrushLayer
                | Self::FieldWorldDefinition
                | Self::WorldEditLog
                | Self::FieldMaterialChannelSet
                | Self::FormedFieldProduct
                | Self::WorldSdfChunkPageArtifact
                | Self::ClipmapBrickmapProduct
        )
    }

    pub const fn is_foreign_reference(self) -> bool {
        matches!(
            self,
            Self::ForeignMeshReferenceSource | Self::ForeignMeshReferenceArtifact
        )
    }

    pub const fn is_formed_product(self) -> bool {
        matches!(
            self,
            Self::FormedFieldProduct
                | Self::WorldSdfChunkPageArtifact
                | Self::ClipmapBrickmapProduct
                | Self::ProceduralMaterial
                | Self::ProceduralTexture
                | Self::Texture2D
                | Self::Texture3DVolume
                | Self::GameplayAtrIrProduct
                | Self::GameplayEcsLoweringProduct
                | Self::ForeignMeshReferenceArtifact
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreign_mesh_kinds_are_reference_only() {
        assert!(AssetKind::ForeignMeshReferenceSource.is_foreign_reference());
        assert!(AssetKind::ForeignMeshReferenceArtifact.is_foreign_reference());
        assert!(!AssetKind::ForeignMeshReferenceSource.is_primary_world_authoring());
        assert!(AssetKind::SdfGraph.is_primary_world_authoring());
    }
}
