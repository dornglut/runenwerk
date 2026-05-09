use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldProductResolution {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
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
            Self::ForeignBlend { .. } => "foreign_blend",
            Self::ForeignGltf => "foreign_gltf",
            Self::Scene => "scene",
            Self::Shader => "shader",
            Self::UiDefinition => "ui_definition",
            Self::RawRon { .. } => "raw_ron",
        }
    }
}
