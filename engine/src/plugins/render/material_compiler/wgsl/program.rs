//! Material IR to WGSL expression program generation.

use std::collections::BTreeMap;

use graph::NodeId;
use material_graph::{
    MaterialIr, MaterialIrInput, MaterialIrInputSource, MaterialIrNode, MaterialNodeOp,
    MaterialValueType,
};

use super::super::bindings::{sampler_binding_variable, texture_binding_variable};
use super::super::{
    CompiledMaterialResourceBinding, CompiledMaterialTextureDimension, MaterialShaderCompileError,
};
use super::literals::{canonical_value_to_wgsl, literal_to_wgsl, output_variable, wgsl_type};

pub(crate) struct WgslMaterialProgram {
    pub(crate) lines: Vec<String>,
    pub(crate) output: MaterialOutputExpressions,
    pub(crate) resource_bindings: Vec<CompiledMaterialResourceBinding>,
}

#[derive(Debug, Clone)]
pub(crate) struct MaterialOutputExpressions {
    pub(crate) base_color: String,
    pub(crate) roughness: String,
    pub(crate) metallic: String,
    pub(crate) normal_strength: String,
    pub(crate) emissive: String,
    pub(crate) opacity: String,
    pub(crate) material_channel: String,
}

impl Default for MaterialOutputExpressions {
    fn default() -> Self {
        Self {
            base_color: "vec4<f32>(0.72, 0.74, 0.77, 1.0)".to_string(),
            roughness: "0.55".to_string(),
            metallic: "0.0".to_string(),
            normal_strength: "1.0".to_string(),
            emissive: "vec3<f32>(0.0, 0.0, 0.0)".to_string(),
            opacity: "1.0".to_string(),
            material_channel: "0.0".to_string(),
        }
    }
}

impl WgslMaterialProgram {
    pub(crate) fn compile(
        ir: &MaterialIr,
        resource_bindings: &[CompiledMaterialResourceBinding],
    ) -> Result<Self, MaterialShaderCompileError> {
        let mut compiler = WgslProgramCompiler::new(resource_bindings);
        for node in &ir.nodes {
            compiler.compile_node(node)?;
        }
        let output_node = ir
            .nodes
            .iter()
            .find(|node| node.op == MaterialNodeOp::PbrOutput)
            .ok_or(MaterialShaderCompileError::MissingOutputNode)?;
        let output = compiler.compile_output(output_node)?;
        Ok(Self {
            lines: compiler.lines,
            output,
            resource_bindings: resource_bindings.to_vec(),
        })
    }
}

struct WgslProgramCompiler {
    lines: Vec<String>,
    outputs: BTreeMap<(u64, String), WgslExpression>,
    resource_bindings: BTreeMap<(u64, String), CompiledMaterialResourceBinding>,
}

#[derive(Debug, Clone)]
struct WgslExpression {
    value_type: MaterialValueType,
    expression: String,
}

impl WgslProgramCompiler {
    fn new(resource_bindings: &[CompiledMaterialResourceBinding]) -> Self {
        Self {
            lines: Vec::new(),
            outputs: BTreeMap::new(),
            resource_bindings: resource_bindings
                .iter()
                .cloned()
                .map(|binding| ((binding.node_id, binding.binding_key.clone()), binding))
                .collect(),
        }
    }

    fn compile_node(&mut self, node: &MaterialIrNode) -> Result<(), MaterialShaderCompileError> {
        let outputs = match node.op {
            MaterialNodeOp::PbrOutput => Vec::new(),
            MaterialNodeOp::PbrBaseColor => vec![(
                "color",
                MaterialValueType::Color,
                self.node_value(node, "color", MaterialValueType::Color)?,
            )],
            MaterialNodeOp::PbrRoughness
            | MaterialNodeOp::PbrMetallic
            | MaterialNodeOp::PbrNormalStrength
            | MaterialNodeOp::PbrOpacity
            | MaterialNodeOp::PbrMaterialChannel => vec![(
                "value",
                MaterialValueType::Float,
                self.node_value(node, "value", MaterialValueType::Float)?,
            )],
            MaterialNodeOp::PbrEmissive => vec![(
                "value",
                MaterialValueType::Vec3,
                self.node_value(node, "value", MaterialValueType::Vec3)?,
            )],
            MaterialNodeOp::SdfPosition => {
                vec![("value", MaterialValueType::Vec3, "ctx.position".to_string())]
            }
            MaterialNodeOp::SdfNormal => {
                vec![("value", MaterialValueType::Vec3, "ctx.normal".to_string())]
            }
            MaterialNodeOp::SdfDistance => {
                vec![(
                    "value",
                    MaterialValueType::Float,
                    "ctx.distance".to_string(),
                )]
            }
            MaterialNodeOp::SdfMaterialChannel => vec![(
                "value",
                MaterialValueType::Float,
                "ctx.material_channel".to_string(),
            )],
            MaterialNodeOp::SdfDensity => {
                vec![("value", MaterialValueType::Float, "ctx.density".to_string())]
            }
            MaterialNodeOp::SdfSupport => {
                vec![("value", MaterialValueType::Float, "ctx.support".to_string())]
            }
            MaterialNodeOp::SdfWetness => {
                vec![("value", MaterialValueType::Float, "ctx.wetness".to_string())]
            }
            MaterialNodeOp::ProcNoise => {
                let position = self.input_expr(node, "position", MaterialValueType::Vec3)?;
                vec![(
                    "value",
                    MaterialValueType::Float,
                    format!("rw_noise3({position})"),
                )]
            }
            MaterialNodeOp::ProcFbm => {
                let position = self.input_expr(node, "position", MaterialValueType::Vec3)?;
                vec![(
                    "value",
                    MaterialValueType::Float,
                    format!("rw_fbm3({position})"),
                )]
            }
            MaterialNodeOp::ProcRamp => {
                let value = self.input_expr(node, "value", MaterialValueType::Float)?;
                let low = self.node_value(node, "low_color", MaterialValueType::Color)?;
                let high = self.node_value(node, "high_color", MaterialValueType::Color)?;
                vec![(
                    "color",
                    MaterialValueType::Color,
                    format!("mix({low}, {high}, clamp({value}, 0.0, 1.0))"),
                )]
            }
            MaterialNodeOp::MathRemap => {
                let value = self.input_expr(node, "value", MaterialValueType::Float)?;
                let from_min = self.node_value(node, "from_min", MaterialValueType::Float)?;
                let from_max = self.node_value(node, "from_max", MaterialValueType::Float)?;
                let to_min = self.node_value(node, "to_min", MaterialValueType::Float)?;
                let to_max = self.node_value(node, "to_max", MaterialValueType::Float)?;
                vec![(
                    "value",
                    MaterialValueType::Float,
                    format!(
                        "({to_min} + ({to_max} - {to_min}) * clamp(({value} - {from_min}) / max(abs({from_max} - {from_min}), 0.000001), 0.0, 1.0))"
                    ),
                )]
            }
            MaterialNodeOp::MathClamp => {
                let value = self.input_expr(node, "value", MaterialValueType::Float)?;
                let min = self.node_value(node, "min", MaterialValueType::Float)?;
                let max = self.node_value(node, "max", MaterialValueType::Float)?;
                vec![(
                    "value",
                    MaterialValueType::Float,
                    format!("clamp({value}, min({min}, {max}), max({min}, {max}))"),
                )]
            }
            MaterialNodeOp::MathMix => {
                let a = self.input_expr(node, "a", MaterialValueType::Float)?;
                let b = self.input_expr(node, "b", MaterialValueType::Float)?;
                let factor = self.input_expr(node, "factor", MaterialValueType::Float)?;
                vec![(
                    "value",
                    MaterialValueType::Float,
                    format!("({a} + ({b} - {a}) * clamp({factor}, 0.0, 1.0))"),
                )]
            }
            MaterialNodeOp::MathMask => {
                let value = self.input_expr(node, "value", MaterialValueType::Float)?;
                let mask = self.input_expr(node, "mask", MaterialValueType::Float)?;
                vec![(
                    "value",
                    MaterialValueType::Float,
                    format!("({value} * clamp({mask}, 0.0, 1.0))"),
                )]
            }
            MaterialNodeOp::TextureSample2D => {
                let uv = self.input_expr(node, "uv", MaterialValueType::Vec2)?;
                let sample = self.texture_sample_expr(
                    node,
                    material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
                    CompiledMaterialTextureDimension::D2,
                    uv,
                )?;
                vec![("color", MaterialValueType::Color, sample)]
            }
            MaterialNodeOp::TextureSample3D => {
                let uv = self.input_expr(node, "uv", MaterialValueType::Vec2)?;
                let uvw = format!("vec3<f32>({uv}, ctx.material_channel)");
                let sample = self.texture_sample_expr(
                    node,
                    material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
                    CompiledMaterialTextureDimension::D3,
                    uvw,
                )?;
                vec![("color", MaterialValueType::Color, sample)]
            }
            MaterialNodeOp::CoordTriplanar => {
                let position = self.input_expr(node, "position", MaterialValueType::Vec3)?;
                let normal = self.input_expr(node, "normal", MaterialValueType::Vec3)?;
                vec![
                    ("uv_x", MaterialValueType::Vec2, format!("({position}).yz")),
                    ("uv_y", MaterialValueType::Vec2, format!("({position}).xz")),
                    ("uv_z", MaterialValueType::Vec2, format!("({position}).xy")),
                    (
                        "weights",
                        MaterialValueType::Vec3,
                        format!("rw_triplanar_weights({normal})"),
                    ),
                ]
            }
        };

        for (name, value_type, expression) in outputs {
            self.declare_output(node.node_id, name, value_type, expression);
        }
        Ok(())
    }

    fn declare_output(
        &mut self,
        node_id: NodeId,
        name: &str,
        value_type: MaterialValueType,
        expression: String,
    ) {
        let variable = output_variable(node_id, name);
        self.lines.push(format!(
            "    let {variable}: {} = {expression};",
            wgsl_type(value_type)
        ));
        self.outputs.insert(
            (node_id.raw(), name.to_string()),
            WgslExpression {
                value_type,
                expression: variable,
            },
        );
    }

    fn compile_output(
        &self,
        node: &MaterialIrNode,
    ) -> Result<MaterialOutputExpressions, MaterialShaderCompileError> {
        Ok(MaterialOutputExpressions {
            base_color: self.input_expr(node, "base_color", MaterialValueType::Color)?,
            roughness: self.input_expr(node, "roughness", MaterialValueType::Float)?,
            metallic: self.input_expr(node, "metallic", MaterialValueType::Float)?,
            normal_strength: self.input_expr(node, "normal_strength", MaterialValueType::Float)?,
            emissive: self.input_expr(node, "emissive", MaterialValueType::Vec3)?,
            opacity: self.input_expr(node, "opacity", MaterialValueType::Float)?,
            material_channel: self.input_expr(
                node,
                "material_channel",
                MaterialValueType::Float,
            )?,
        })
    }

    fn input_expr(
        &self,
        node: &MaterialIrNode,
        name: &str,
        expected_type: MaterialValueType,
    ) -> Result<String, MaterialShaderCompileError> {
        let input = node
            .inputs
            .iter()
            .find(|input| input.name == name)
            .ok_or_else(|| MaterialShaderCompileError::MissingInput {
                node_id: node.node_id.raw(),
                input: name.to_string(),
            })?;
        if input.value_type != expected_type {
            return Err(MaterialShaderCompileError::InvalidNodeContract {
                node_id: node.node_id.raw(),
                message: format!(
                    "input '{name}' has type {}, expected {}",
                    input.value_type.label(),
                    expected_type.label()
                ),
            });
        }
        self.input_source_expr(node, input)
    }

    fn input_source_expr(
        &self,
        node: &MaterialIrNode,
        input: &MaterialIrInput,
    ) -> Result<String, MaterialShaderCompileError> {
        match &input.source {
            MaterialIrInputSource::Connected {
                node_id,
                output_name,
            } => {
                let Some(output) = self.outputs.get(&(node_id.raw(), output_name.clone())) else {
                    return Err(MaterialShaderCompileError::MissingConnectedOutput {
                        node_id: node.node_id.raw(),
                        input: input.name.clone(),
                        source_node_id: node_id.raw(),
                        output: output_name.clone(),
                    });
                };
                if output.value_type != input.value_type {
                    return Err(MaterialShaderCompileError::InvalidNodeContract {
                        node_id: node.node_id.raw(),
                        message: format!(
                            "input '{}' consumes {}, but connected output '{}' has {}",
                            input.name,
                            input.value_type.label(),
                            output_name,
                            output.value_type.label()
                        ),
                    });
                }
                Ok(output.expression.clone())
            }
            MaterialIrInputSource::Constant(literal) => literal_to_wgsl(literal, input.value_type),
            MaterialIrInputSource::NodeValue {
                canonical_value, ..
            } => canonical_value_to_wgsl(canonical_value, input.value_type),
        }
    }

    fn node_value(
        &self,
        node: &MaterialIrNode,
        key: &str,
        value_type: MaterialValueType,
    ) -> Result<String, MaterialShaderCompileError> {
        let value = node
            .values
            .iter()
            .find(|value| value.key == key)
            .ok_or_else(|| MaterialShaderCompileError::MissingNodeValue {
                node_id: node.node_id.raw(),
                key: key.to_string(),
            })?;
        canonical_value_to_wgsl(&value.canonical_value, value_type)
    }

    fn require_resource(
        &self,
        node: &MaterialIrNode,
        key: &str,
    ) -> Result<(), MaterialShaderCompileError> {
        if node.values.iter().any(|value| value.key == key) {
            Ok(())
        } else {
            Err(MaterialShaderCompileError::MissingResourceBinding {
                node_id: node.node_id.raw(),
                key: key.to_string(),
            })
        }
    }

    fn resource_binding(
        &self,
        node: &MaterialIrNode,
        key: &str,
        expected_dimension: CompiledMaterialTextureDimension,
    ) -> Result<&CompiledMaterialResourceBinding, MaterialShaderCompileError> {
        self.require_resource(node, key)?;
        let Some(binding) = self
            .resource_bindings
            .get(&(node.node_id.raw(), key.to_string()))
        else {
            return Err(MaterialShaderCompileError::MissingResourceBinding {
                node_id: node.node_id.raw(),
                key: key.to_string(),
            });
        };
        if binding.texture_dimension != expected_dimension {
            return Err(MaterialShaderCompileError::InvalidNodeContract {
                node_id: node.node_id.raw(),
                message: format!(
                    "texture resource '{}' has {:?}, expected {:?}",
                    key, binding.texture_dimension, expected_dimension
                ),
            });
        }
        Ok(binding)
    }

    fn texture_sample_expr(
        &self,
        node: &MaterialIrNode,
        key: &str,
        expected_dimension: CompiledMaterialTextureDimension,
        coordinates: String,
    ) -> Result<String, MaterialShaderCompileError> {
        let binding = self.resource_binding(node, key, expected_dimension)?;
        Ok(format!(
            "textureSample({}, {}, {coordinates})",
            texture_binding_variable(binding),
            sampler_binding_variable(binding)
        ))
    }
}
