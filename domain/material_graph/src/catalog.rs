//! File: domain/material_graph/src/catalog.rs
//! Purpose: Material graph node catalog boundaries.

use graph::PortTypeId;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialNodeDescriptor {
    pub key: String,
    pub label: String,
    pub semantic_version: u32,
    pub compiler_op: crate::MaterialNodeOp,
    pub inputs: Vec<MaterialInputContract>,
    pub outputs: Vec<MaterialOutputContract>,
    pub values: Vec<MaterialValueContract>,
    pub resources: Vec<MaterialResourceContract>,
    pub output_targets: Vec<crate::MaterialOutputTarget>,
}

impl MaterialNodeDescriptor {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        let key = key.into();
        Self {
            compiler_op: crate::MaterialNodeOp::from_catalog_key(&key)
                .expect("material node descriptor keys must have compiler semantics"),
            key,
            label: label.into(),
            semantic_version: 1,
            inputs: Vec::new(),
            outputs: Vec::new(),
            values: Vec::new(),
            resources: Vec::new(),
            output_targets: Vec::new(),
        }
    }

    pub fn with_semantic_version(mut self, semantic_version: u32) -> Self {
        self.semantic_version = semantic_version;
        self
    }

    pub fn with_inputs(mut self, inputs: impl IntoIterator<Item = MaterialInputContract>) -> Self {
        self.inputs = inputs.into_iter().collect();
        self
    }

    pub fn with_outputs(
        mut self,
        outputs: impl IntoIterator<Item = MaterialOutputContract>,
    ) -> Self {
        self.outputs = outputs.into_iter().collect();
        self
    }

    pub fn with_values(mut self, values: impl IntoIterator<Item = MaterialValueContract>) -> Self {
        self.values = values.into_iter().collect();
        self
    }

    pub fn with_resources(
        mut self,
        resources: impl IntoIterator<Item = MaterialResourceContract>,
    ) -> Self {
        self.resources = resources.into_iter().collect();
        self
    }

    pub fn with_output_targets(
        mut self,
        output_targets: impl IntoIterator<Item = crate::MaterialOutputTarget>,
    ) -> Self {
        self.output_targets = output_targets.into_iter().collect();
        self
    }

    pub fn supports_output_target(&self, target: crate::MaterialOutputTarget) -> bool {
        self.output_targets.is_empty() || self.output_targets.contains(&target)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialValueType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Color,
    Bool,
    ResourceTexture2D,
    ResourceTexture3D,
}

impl MaterialValueType {
    pub const fn port_type_id(self) -> PortTypeId {
        match self {
            Self::Color => PortTypeId::new(1),
            Self::Float => PortTypeId::new(2),
            Self::Vec2 => PortTypeId::new(3),
            Self::Vec3 => PortTypeId::new(4),
            Self::Vec4 => PortTypeId::new(5),
            Self::Bool => PortTypeId::new(6),
            Self::ResourceTexture2D => PortTypeId::new(7),
            Self::ResourceTexture3D => PortTypeId::new(8),
        }
    }

    pub const fn from_port_type_id(port_type: PortTypeId) -> Option<Self> {
        match port_type.raw() {
            1 => Some(Self::Color),
            2 => Some(Self::Float),
            3 => Some(Self::Vec2),
            4 => Some(Self::Vec3),
            5 => Some(Self::Vec4),
            6 => Some(Self::Bool),
            7 => Some(Self::ResourceTexture2D),
            8 => Some(Self::ResourceTexture3D),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Float => "float",
            Self::Vec2 => "vec2",
            Self::Vec3 => "vec3",
            Self::Vec4 => "vec4",
            Self::Color => "color",
            Self::Bool => "bool",
            Self::ResourceTexture2D => "resource_texture_2d",
            Self::ResourceTexture3D => "resource_texture_3d",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MaterialLiteral {
    Bool(bool),
    Float(String),
    Vec2([String; 2]),
    Vec3([String; 3]),
    Vec4([String; 4]),
    Color([String; 4]),
}

impl MaterialLiteral {
    pub fn float(value: impl Into<String>) -> Self {
        Self::Float(value.into())
    }

    pub fn vec2(x: impl Into<String>, y: impl Into<String>) -> Self {
        Self::Vec2([x.into(), y.into()])
    }

    pub fn vec3(x: impl Into<String>, y: impl Into<String>, z: impl Into<String>) -> Self {
        Self::Vec3([x.into(), y.into(), z.into()])
    }

    pub fn vec4(
        x: impl Into<String>,
        y: impl Into<String>,
        z: impl Into<String>,
        w: impl Into<String>,
    ) -> Self {
        Self::Vec4([x.into(), y.into(), z.into(), w.into()])
    }

    pub fn color(
        r: impl Into<String>,
        g: impl Into<String>,
        b: impl Into<String>,
        a: impl Into<String>,
    ) -> Self {
        Self::Color([r.into(), g.into(), b.into(), a.into()])
    }

    pub fn canonical_component(&self) -> String {
        match self {
            Self::Bool(value) => format!("bool:{value}"),
            Self::Float(value) => format!("float:{}:{value}", value.len()),
            Self::Vec2(values) => canonical_vector("vec2", values),
            Self::Vec3(values) => canonical_vector("vec3", values),
            Self::Vec4(values) => canonical_vector("vec4", values),
            Self::Color(values) => canonical_vector("color", values),
        }
    }
}

fn canonical_vector<const N: usize>(label: &str, values: &[String; N]) -> String {
    let mut component = format!("{label}:{N}");
    for value in values {
        component.push(':');
        component.push_str(&value.len().to_string());
        component.push(':');
        component.push_str(value);
    }
    component
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialInputContract {
    pub name: String,
    pub value_type: MaterialValueType,
    pub default_value: Option<MaterialLiteral>,
}

impl MaterialInputContract {
    pub fn required(name: impl Into<String>, value_type: MaterialValueType) -> Self {
        Self {
            name: name.into(),
            value_type,
            default_value: None,
        }
    }

    pub fn defaulted(
        name: impl Into<String>,
        value_type: MaterialValueType,
        default_value: MaterialLiteral,
    ) -> Self {
        Self {
            name: name.into(),
            value_type,
            default_value: Some(default_value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialOutputContract {
    pub name: String,
    pub value_type: MaterialValueType,
}

impl MaterialOutputContract {
    pub fn new(name: impl Into<String>, value_type: MaterialValueType) -> Self {
        Self {
            name: name.into(),
            value_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialValueContract {
    pub key: String,
    pub value_type: MaterialValueType,
    pub default_value: Option<MaterialLiteral>,
}

impl MaterialValueContract {
    pub fn required(key: impl Into<String>, value_type: MaterialValueType) -> Self {
        Self {
            key: key.into(),
            value_type,
            default_value: None,
        }
    }

    pub fn defaulted(
        key: impl Into<String>,
        value_type: MaterialValueType,
        default_value: MaterialLiteral,
    ) -> Self {
        Self {
            key: key.into(),
            value_type,
            default_value: Some(default_value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialResourceKind {
    Texture2D,
    Texture3D,
}

impl MaterialResourceKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Texture2D => "texture_2d",
            Self::Texture3D => "texture_3d",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialResourceContract {
    pub key: String,
    pub kind: MaterialResourceKind,
}

impl MaterialResourceContract {
    pub fn new(key: impl Into<String>, kind: MaterialResourceKind) -> Self {
        Self {
            key: key.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaterialNodeCatalog {
    stable_id: String,
    version: u32,
    nodes_by_key: BTreeMap<String, MaterialNodeDescriptor>,
}

impl MaterialNodeCatalog {
    pub fn new(
        nodes: impl IntoIterator<Item = MaterialNodeDescriptor>,
    ) -> Result<Self, MaterialNodeCatalogError> {
        Self::new_with_identity("material_node_catalog.custom", 1, nodes)
    }

    pub fn new_with_identity(
        stable_id: impl Into<String>,
        version: u32,
        nodes: impl IntoIterator<Item = MaterialNodeDescriptor>,
    ) -> Result<Self, MaterialNodeCatalogError> {
        let stable_id = stable_id.into();
        if stable_id.trim().is_empty() {
            return Err(MaterialNodeCatalogError::EmptyCatalogId);
        }
        if version == 0 {
            return Err(MaterialNodeCatalogError::ZeroCatalogVersion {
                stable_id: stable_id.clone(),
            });
        }

        let mut nodes_by_key = BTreeMap::<String, MaterialNodeDescriptor>::new();
        for node in nodes {
            if node.key.trim().is_empty() {
                return Err(MaterialNodeCatalogError::EmptyNodeKey);
            }
            let key = node.key.clone();
            if node.semantic_version == 0 {
                return Err(MaterialNodeCatalogError::ZeroNodeSemanticVersion { key });
            }
            validate_node_semantic_contract(&node)?;
            if nodes_by_key.insert(key.clone(), node).is_some() {
                return Err(MaterialNodeCatalogError::DuplicateNodeKey { key });
            }
        }

        Ok(Self {
            stable_id,
            version,
            nodes_by_key,
        })
    }

    pub fn first_slice() -> Self {
        Self::new_with_identity(
            "material_node_catalog.first_slice",
            1,
            [
                pbr_output_descriptor(),
                color_output_descriptor("pbr.base_color", "Base Color", [0.72, 0.74, 0.77, 1.0]),
                scalar_output_descriptor("pbr.roughness", "Roughness", 0.55),
                scalar_output_descriptor("pbr.metallic", "Metallic", 0.0),
                scalar_output_descriptor("pbr.normal_strength", "Normal Strength", 1.0),
                vec3_output_descriptor("pbr.emissive", "Emissive", [0.0, 0.0, 0.0]),
                scalar_output_descriptor("pbr.opacity", "Opacity", 1.0),
                scalar_output_descriptor("pbr.material_channel", "Material Channel", 0.0),
                vec3_context_descriptor("sdf.position", "SDF Position"),
                vec3_context_descriptor("sdf.normal", "SDF Normal"),
                scalar_context_descriptor("sdf.distance", "SDF Distance"),
                scalar_context_descriptor("sdf.material_channel", "SDF Material Channel"),
                scalar_context_descriptor("sdf.density", "SDF Density"),
                scalar_context_descriptor("sdf.support", "SDF Support"),
                scalar_context_descriptor("sdf.wetness", "SDF Wetness"),
                scalar_math_descriptor("proc.noise", "Noise"),
                scalar_math_descriptor("proc.fbm", "FBM"),
                ramp_descriptor(),
                remap_descriptor(),
                clamp_descriptor(),
                mix_descriptor(),
                mask_descriptor(),
                texture_descriptor(
                    "texture.sample_2d",
                    "Texture2D Sample",
                    MaterialResourceKind::Texture2D,
                    MaterialValueType::ResourceTexture2D,
                ),
                texture_descriptor(
                    "texture.sample_3d",
                    "Texture3D Sample",
                    MaterialResourceKind::Texture3D,
                    MaterialValueType::ResourceTexture3D,
                ),
                triplanar_descriptor(),
            ],
        )
        .expect("built-in first-slice material node catalog must be valid")
    }

    pub fn stable_id(&self) -> &str {
        &self.stable_id
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn contains(&self, key: &str) -> bool {
        self.nodes_by_key.contains_key(key)
    }

    pub fn descriptor(&self, key: &str) -> Option<&MaterialNodeDescriptor> {
        self.nodes_by_key.get(key)
    }

    pub fn len(&self) -> usize {
        self.nodes_by_key.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes_by_key.is_empty()
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &MaterialNodeDescriptor> {
        self.nodes_by_key.values()
    }
}

fn validate_node_semantic_contract(
    node: &MaterialNodeDescriptor,
) -> Result<(), MaterialNodeCatalogError> {
    let key = node.key.clone();
    let mut inputs = BTreeSet::<String>::new();
    for input in &node.inputs {
        if input.name.trim().is_empty() {
            return Err(MaterialNodeCatalogError::EmptySemanticField {
                key,
                field: "input name",
            });
        }
        if !inputs.insert(input.name.clone()) {
            return Err(MaterialNodeCatalogError::DuplicateInput {
                key,
                input: input.name.clone(),
            });
        }
    }
    let key = node.key.clone();
    let mut outputs = BTreeSet::<String>::new();
    for output in &node.outputs {
        if output.name.trim().is_empty() {
            return Err(MaterialNodeCatalogError::EmptySemanticField {
                key,
                field: "output name",
            });
        }
        if !outputs.insert(output.name.clone()) {
            return Err(MaterialNodeCatalogError::DuplicateOutput {
                key,
                output: output.name.clone(),
            });
        }
    }
    let key = node.key.clone();
    let mut values = BTreeSet::<String>::new();
    for value in &node.values {
        if value.key.trim().is_empty() {
            return Err(MaterialNodeCatalogError::EmptySemanticField {
                key,
                field: "value key",
            });
        }
        if !values.insert(value.key.clone()) {
            return Err(MaterialNodeCatalogError::DuplicateValue {
                key,
                value: value.key.clone(),
            });
        }
    }
    let key = node.key.clone();
    let mut resources = BTreeSet::<String>::new();
    for resource in &node.resources {
        if resource.key.trim().is_empty() {
            return Err(MaterialNodeCatalogError::EmptySemanticField {
                key,
                field: "resource key",
            });
        }
        if !resources.insert(resource.key.clone()) {
            return Err(MaterialNodeCatalogError::DuplicateResource {
                key,
                resource: resource.key.clone(),
            });
        }
    }
    Ok(())
}

fn pbr_output_descriptor() -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new("pbr.output", "PBR Output")
        .with_inputs([
            MaterialInputContract::defaulted(
                "base_color",
                MaterialValueType::Color,
                MaterialLiteral::color("0.72", "0.74", "0.77", "1.0"),
            ),
            MaterialInputContract::defaulted(
                "roughness",
                MaterialValueType::Float,
                MaterialLiteral::float("0.55"),
            ),
            MaterialInputContract::defaulted(
                "metallic",
                MaterialValueType::Float,
                MaterialLiteral::float("0.0"),
            ),
            MaterialInputContract::defaulted(
                "normal_strength",
                MaterialValueType::Float,
                MaterialLiteral::float("1.0"),
            ),
            MaterialInputContract::defaulted(
                "emissive",
                MaterialValueType::Vec3,
                MaterialLiteral::vec3("0.0", "0.0", "0.0"),
            ),
            MaterialInputContract::defaulted(
                "opacity",
                MaterialValueType::Float,
                MaterialLiteral::float("1.0"),
            ),
            MaterialInputContract::defaulted(
                "material_channel",
                MaterialValueType::Float,
                MaterialLiteral::float("0.0"),
            ),
        ])
        .with_output_targets([
            crate::MaterialOutputTarget::PbrPreview,
            crate::MaterialOutputTarget::RenderMaterial,
            crate::MaterialOutputTarget::FieldMaterialChannel,
        ])
}

fn color_output_descriptor(
    key: &'static str,
    label: &'static str,
    color: [f32; 4],
) -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new(key, label)
        .with_outputs([MaterialOutputContract::new(
            "color",
            MaterialValueType::Color,
        )])
        .with_values([MaterialValueContract::defaulted(
            "color",
            MaterialValueType::Color,
            MaterialLiteral::color(
                color[0].to_string(),
                color[1].to_string(),
                color[2].to_string(),
                color[3].to_string(),
            ),
        )])
}

fn scalar_output_descriptor(
    key: &'static str,
    label: &'static str,
    default: f32,
) -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new(key, label)
        .with_outputs([MaterialOutputContract::new(
            "value",
            MaterialValueType::Float,
        )])
        .with_values([MaterialValueContract::defaulted(
            "value",
            MaterialValueType::Float,
            MaterialLiteral::float(default.to_string()),
        )])
}

fn vec3_output_descriptor(
    key: &'static str,
    label: &'static str,
    default: [f32; 3],
) -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new(key, label)
        .with_outputs([MaterialOutputContract::new(
            "value",
            MaterialValueType::Vec3,
        )])
        .with_values([MaterialValueContract::defaulted(
            "value",
            MaterialValueType::Vec3,
            MaterialLiteral::vec3(
                default[0].to_string(),
                default[1].to_string(),
                default[2].to_string(),
            ),
        )])
}

fn vec3_context_descriptor(key: &'static str, label: &'static str) -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new(key, label).with_outputs([MaterialOutputContract::new(
        "value",
        MaterialValueType::Vec3,
    )])
}

fn scalar_context_descriptor(key: &'static str, label: &'static str) -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new(key, label).with_outputs([MaterialOutputContract::new(
        "value",
        MaterialValueType::Float,
    )])
}

fn scalar_math_descriptor(key: &'static str, label: &'static str) -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new(key, label)
        .with_inputs([MaterialInputContract::defaulted(
            "position",
            MaterialValueType::Vec3,
            MaterialLiteral::vec3("0.0", "0.0", "0.0"),
        )])
        .with_outputs([MaterialOutputContract::new(
            "value",
            MaterialValueType::Float,
        )])
}

fn ramp_descriptor() -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new("proc.ramp", "Ramp")
        .with_inputs([MaterialInputContract::defaulted(
            "value",
            MaterialValueType::Float,
            MaterialLiteral::float("0.5"),
        )])
        .with_values([
            MaterialValueContract::defaulted(
                "low_color",
                MaterialValueType::Color,
                MaterialLiteral::color("0.0", "0.0", "0.0", "1.0"),
            ),
            MaterialValueContract::defaulted(
                "high_color",
                MaterialValueType::Color,
                MaterialLiteral::color("1.0", "1.0", "1.0", "1.0"),
            ),
        ])
        .with_outputs([MaterialOutputContract::new(
            "color",
            MaterialValueType::Color,
        )])
}

fn remap_descriptor() -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new("math.remap", "Remap")
        .with_inputs([MaterialInputContract::defaulted(
            "value",
            MaterialValueType::Float,
            MaterialLiteral::float("0.0"),
        )])
        .with_values([
            MaterialValueContract::defaulted(
                "from_min",
                MaterialValueType::Float,
                MaterialLiteral::float("0.0"),
            ),
            MaterialValueContract::defaulted(
                "from_max",
                MaterialValueType::Float,
                MaterialLiteral::float("1.0"),
            ),
            MaterialValueContract::defaulted(
                "to_min",
                MaterialValueType::Float,
                MaterialLiteral::float("0.0"),
            ),
            MaterialValueContract::defaulted(
                "to_max",
                MaterialValueType::Float,
                MaterialLiteral::float("1.0"),
            ),
        ])
        .with_outputs([MaterialOutputContract::new(
            "value",
            MaterialValueType::Float,
        )])
}

fn clamp_descriptor() -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new("math.clamp", "Clamp")
        .with_inputs([MaterialInputContract::defaulted(
            "value",
            MaterialValueType::Float,
            MaterialLiteral::float("0.0"),
        )])
        .with_values([
            MaterialValueContract::defaulted(
                "min",
                MaterialValueType::Float,
                MaterialLiteral::float("0.0"),
            ),
            MaterialValueContract::defaulted(
                "max",
                MaterialValueType::Float,
                MaterialLiteral::float("1.0"),
            ),
        ])
        .with_outputs([MaterialOutputContract::new(
            "value",
            MaterialValueType::Float,
        )])
}

fn mix_descriptor() -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new("math.mix", "Mix")
        .with_inputs([
            MaterialInputContract::defaulted(
                "a",
                MaterialValueType::Float,
                MaterialLiteral::float("0.0"),
            ),
            MaterialInputContract::defaulted(
                "b",
                MaterialValueType::Float,
                MaterialLiteral::float("1.0"),
            ),
            MaterialInputContract::defaulted(
                "factor",
                MaterialValueType::Float,
                MaterialLiteral::float("0.5"),
            ),
        ])
        .with_outputs([MaterialOutputContract::new(
            "value",
            MaterialValueType::Float,
        )])
}

fn mask_descriptor() -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new("math.mask", "Mask")
        .with_inputs([
            MaterialInputContract::defaulted(
                "value",
                MaterialValueType::Float,
                MaterialLiteral::float("1.0"),
            ),
            MaterialInputContract::defaulted(
                "mask",
                MaterialValueType::Float,
                MaterialLiteral::float("1.0"),
            ),
        ])
        .with_outputs([MaterialOutputContract::new(
            "value",
            MaterialValueType::Float,
        )])
}

fn texture_descriptor(
    key: &'static str,
    label: &'static str,
    resource_kind: MaterialResourceKind,
    resource_value_type: MaterialValueType,
) -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new(key, label)
        .with_inputs([MaterialInputContract::defaulted(
            "uv",
            MaterialValueType::Vec2,
            MaterialLiteral::vec2("0.0", "0.0"),
        )])
        .with_resources([MaterialResourceContract::new(
            crate::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
            resource_kind,
        )])
        .with_values([MaterialValueContract::required(
            crate::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
            resource_value_type,
        )])
        .with_outputs([MaterialOutputContract::new(
            "color",
            MaterialValueType::Color,
        )])
}

fn triplanar_descriptor() -> MaterialNodeDescriptor {
    MaterialNodeDescriptor::new("coord.triplanar", "Triplanar Coordinates")
        .with_inputs([
            MaterialInputContract::defaulted(
                "position",
                MaterialValueType::Vec3,
                MaterialLiteral::vec3("0.0", "0.0", "0.0"),
            ),
            MaterialInputContract::defaulted(
                "normal",
                MaterialValueType::Vec3,
                MaterialLiteral::vec3("0.0", "1.0", "0.0"),
            ),
        ])
        .with_outputs([
            MaterialOutputContract::new("uv_x", MaterialValueType::Vec2),
            MaterialOutputContract::new("uv_y", MaterialValueType::Vec2),
            MaterialOutputContract::new("uv_z", MaterialValueType::Vec2),
            MaterialOutputContract::new("weights", MaterialValueType::Vec3),
        ])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialNodeCatalogError {
    EmptyCatalogId,
    ZeroCatalogVersion { stable_id: String },
    EmptyNodeKey,
    DuplicateNodeKey { key: String },
    ZeroNodeSemanticVersion { key: String },
    DuplicateInput { key: String, input: String },
    DuplicateOutput { key: String, output: String },
    DuplicateValue { key: String, value: String },
    DuplicateResource { key: String, resource: String },
    EmptySemanticField { key: String, field: &'static str },
}

impl fmt::Display for MaterialNodeCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCatalogId => formatter.write_str("material node catalog id is empty"),
            Self::ZeroCatalogVersion { stable_id } => write!(
                formatter,
                "material node catalog '{stable_id}' version must be non-zero"
            ),
            Self::EmptyNodeKey => {
                formatter.write_str("material node catalog contains an empty node key")
            }
            Self::DuplicateNodeKey { key } => {
                write!(
                    formatter,
                    "material node catalog contains duplicate node key '{key}'"
                )
            }
            Self::ZeroNodeSemanticVersion { key } => write!(
                formatter,
                "material node catalog node '{key}' semantic version must be non-zero"
            ),
            Self::DuplicateInput { key, input } => {
                write!(
                    formatter,
                    "material node '{key}' has duplicate input '{input}'"
                )
            }
            Self::DuplicateOutput { key, output } => {
                write!(
                    formatter,
                    "material node '{key}' has duplicate output '{output}'"
                )
            }
            Self::DuplicateValue { key, value } => {
                write!(
                    formatter,
                    "material node '{key}' has duplicate value '{value}'"
                )
            }
            Self::DuplicateResource { key, resource } => {
                write!(
                    formatter,
                    "material node '{key}' has duplicate resource '{resource}'"
                )
            }
            Self::EmptySemanticField { key, field } => {
                write!(formatter, "material node '{key}' has empty {field}")
            }
        }
    }
}

impl Error for MaterialNodeCatalogError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_rejects_duplicate_descriptor_keys() {
        let error = MaterialNodeCatalog::new_with_identity(
            "material_node_catalog.test",
            1,
            [
                MaterialNodeDescriptor::new("pbr.output", "Output"),
                MaterialNodeDescriptor::new("pbr.output", "Duplicate"),
            ],
        )
        .expect_err("duplicate descriptor keys must be rejected");

        assert_eq!(
            error,
            MaterialNodeCatalogError::DuplicateNodeKey {
                key: "pbr.output".to_string(),
            }
        );
    }

    #[test]
    fn catalog_rejects_invalid_identity_and_semantic_versions() {
        assert_eq!(
            MaterialNodeCatalog::new_with_identity(
                "",
                1,
                [MaterialNodeDescriptor::new("pbr.output", "Output")]
            )
            .expect_err("empty catalog id must fail"),
            MaterialNodeCatalogError::EmptyCatalogId,
        );
        assert_eq!(
            MaterialNodeCatalog::new_with_identity(
                "material_node_catalog.test",
                0,
                [MaterialNodeDescriptor::new("pbr.output", "Output")]
            )
            .expect_err("zero catalog version must fail"),
            MaterialNodeCatalogError::ZeroCatalogVersion {
                stable_id: "material_node_catalog.test".to_string(),
            },
        );
        assert_eq!(
            MaterialNodeCatalog::new_with_identity(
                "material_node_catalog.test",
                1,
                [MaterialNodeDescriptor::new("pbr.output", "Output").with_semantic_version(0)]
            )
            .expect_err("zero semantic version must fail"),
            MaterialNodeCatalogError::ZeroNodeSemanticVersion {
                key: "pbr.output".to_string(),
            },
        );
    }
}
