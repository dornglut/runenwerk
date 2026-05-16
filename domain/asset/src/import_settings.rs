use serde::{Deserialize, Serialize};

use crate::AssetKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldProductResolution {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureProductResolution {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl TextureProductResolution {
    pub const fn new(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureImportColorSpace {
    Linear,
    Srgb,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureImportCompression {
    Uncompressed,
    Bc5,
    Bc7,
    Astc,
}

impl FieldProductResolution {
    pub const fn new(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ImportSettings {
    SdfGraph {
        resolution: FieldProductResolution,
    },
    SdfBrushLayer {
        resolution: FieldProductResolution,
    },
    FieldWorldDefinition {
        resolution: FieldProductResolution,
    },
    WorldSdfProduct {
        resolution: FieldProductResolution,
        scale_band: String,
    },
    FieldProductDescriptor {
        scale_band: String,
    },
    MaterialGraph {
        lowering_target: String,
    },
    Material {
        product_target: String,
    },
    Prefab {
        descriptor_profile: String,
    },
    ProceduralTexture {
        resolution: TextureProductResolution,
        color_space: TextureImportColorSpace,
    },
    Texture2D {
        color_space: TextureImportColorSpace,
        compression: TextureImportCompression,
    },
    Texture3DVolume {
        resolution: TextureProductResolution,
        color_space: TextureImportColorSpace,
        compression: TextureImportCompression,
    },
    ForeignBlend {
        blender_executable: Option<String>,
        export_format: String,
    },
    ForeignGltf,
    Scene,
    Shader,
    UiDefinition,
    RawRon {
        schema_hint: Option<String>,
    },
}

impl ImportSettings {
    pub fn stable_kind_label(&self) -> &'static str {
        match self {
            Self::SdfGraph { .. } => "sdf_graph",
            Self::SdfBrushLayer { .. } => "sdf_brush_layer",
            Self::FieldWorldDefinition { .. } => "field_world_definition",
            Self::WorldSdfProduct { .. } => "world_sdf_product",
            Self::FieldProductDescriptor { .. } => "field_product_descriptor",
            Self::MaterialGraph { .. } => "material_graph",
            Self::Material { .. } => "material",
            Self::Prefab { .. } => "prefab",
            Self::ProceduralTexture { .. } => "procedural_texture",
            Self::Texture2D { .. } => "texture_2d",
            Self::Texture3DVolume { .. } => "texture_3d_volume",
            Self::ForeignBlend { .. } => "foreign_blend",
            Self::ForeignGltf => "foreign_gltf",
            Self::Scene => "scene",
            Self::Shader => "shader",
            Self::UiDefinition => "ui_definition",
            Self::RawRon { .. } => "raw_ron",
        }
    }

    pub const fn supports_source_kind(&self, kind: AssetKind) -> bool {
        matches!(
            (self, kind),
            (Self::SdfGraph { .. }, AssetKind::SdfGraph)
                | (Self::SdfBrushLayer { .. }, AssetKind::SdfBrushLayer)
                | (
                    Self::FieldWorldDefinition { .. },
                    AssetKind::FieldWorldDefinition
                )
                | (Self::WorldSdfProduct { .. }, AssetKind::SdfGraph)
                | (
                    Self::WorldSdfProduct { .. },
                    AssetKind::FieldWorldDefinition
                )
                | (
                    Self::FieldProductDescriptor { .. },
                    AssetKind::FormedFieldProduct
                )
                | (Self::MaterialGraph { .. }, AssetKind::MaterialGraph)
                | (Self::Material { .. }, AssetKind::Material)
                | (Self::Prefab { .. }, AssetKind::Prefab)
                | (Self::ProceduralTexture { .. }, AssetKind::ProceduralTexture)
                | (Self::Texture2D { .. }, AssetKind::Texture2D)
                | (Self::Texture3DVolume { .. }, AssetKind::Texture3DVolume)
                | (
                    Self::ForeignBlend { .. },
                    AssetKind::ForeignMeshReferenceSource
                )
                | (Self::ForeignGltf, AssetKind::ForeignMeshReferenceSource)
                | (Self::Scene, AssetKind::Scene)
                | (Self::Shader, AssetKind::Shader)
                | (Self::UiDefinition, AssetKind::UiDefinition)
                | (Self::RawRon { .. }, AssetKind::Graph)
                | (Self::RawRon { .. }, AssetKind::Theme)
                | (Self::RawRon { .. }, AssetKind::Menu)
                | (Self::RawRon { .. }, AssetKind::Shortcut)
                | (Self::RawRon { .. }, AssetKind::WorkspaceDefinition)
                | (Self::RawRon { .. }, AssetKind::EditorDefinition)
        )
    }

    pub const fn supports_artifact_kind(&self, kind: AssetKind) -> bool {
        matches!(
            (self, kind),
            (Self::SdfGraph { .. }, AssetKind::FormedFieldProduct)
                | (Self::SdfBrushLayer { .. }, AssetKind::FormedFieldProduct)
                | (
                    Self::FieldWorldDefinition { .. },
                    AssetKind::FormedFieldProduct
                )
                | (
                    Self::WorldSdfProduct { .. },
                    AssetKind::WorldSdfChunkPageArtifact
                )
                | (
                    Self::FieldProductDescriptor { .. },
                    AssetKind::FormedFieldProduct
                )
                | (Self::MaterialGraph { .. }, AssetKind::Material)
                | (Self::Material { .. }, AssetKind::Material)
                | (Self::Prefab { .. }, AssetKind::Prefab)
                | (Self::ProceduralTexture { .. }, AssetKind::ProceduralTexture)
                | (Self::Texture2D { .. }, AssetKind::Texture2D)
                | (Self::Texture3DVolume { .. }, AssetKind::Texture3DVolume)
                | (
                    Self::ForeignBlend { .. },
                    AssetKind::ForeignMeshReferenceArtifact
                )
                | (Self::ForeignGltf, AssetKind::ForeignMeshReferenceArtifact)
                | (Self::Scene, AssetKind::Scene)
                | (Self::Shader, AssetKind::Shader)
                | (Self::UiDefinition, AssetKind::UiDefinition)
                | (Self::RawRon { .. }, AssetKind::Graph)
                | (Self::RawRon { .. }, AssetKind::Theme)
                | (Self::RawRon { .. }, AssetKind::Menu)
                | (Self::RawRon { .. }, AssetKind::Shortcut)
                | (Self::RawRon { .. }, AssetKind::WorkspaceDefinition)
                | (Self::RawRon { .. }, AssetKind::EditorDefinition)
        )
    }
}
