//! File: engine/src/plugins/render/material_compiler/mod.rs
//! Purpose: Engine-owned material IR to WGSL compiler and validation boundary.

use graph::NodeId;
use material_graph::{
    MATERIAL_IR_CONTRACT_VERSION, MaterialIr, MaterialIrInput, MaterialIrInputSource,
    MaterialIrNode, MaterialLiteral, MaterialNodeOp, MaterialOutputTarget, MaterialValueType,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub const MATERIAL_WGSL_COMPILER_CONTRACT_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledMaterialShader {
    pub shader_id: String,
    pub wgsl: String,
    pub scene_wgsl: String,
    pub resource_bindings: Vec<CompiledMaterialResourceBinding>,
    pub identity: String,
    pub scene_identity: String,
    pub output_target: MaterialOutputTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledMaterialResourceBinding {
    pub node_id: u64,
    pub binding_key: String,
    pub bind_group: u32,
    pub texture_binding: u32,
    pub sampler_binding: u32,
    pub texture_dimension: CompiledMaterialTextureDimension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledMaterialTextureDimension {
    D2,
    D3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialPreviewFixture {
    Sphere,
    Box,
    Plane,
    SdfPrimitive,
    FieldMaterial,
}

impl MaterialPreviewFixture {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sphere => "sphere",
            Self::Box => "box",
            Self::Plane => "plane",
            Self::SdfPrimitive => "sdf_primitive",
            Self::FieldMaterial => "field_material",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialShaderCompileRequest<'a> {
    pub ir: &'a MaterialIr,
    pub fixture: MaterialPreviewFixture,
}

pub fn compile_material_shader(
    request: MaterialShaderCompileRequest<'_>,
) -> Result<CompiledMaterialShader, MaterialShaderCompileError> {
    validate_ir(request.ir)?;
    let resource_bindings = compiled_resource_bindings(request.ir);
    let generated = WgslMaterialProgram::compile(request.ir, &resource_bindings)?;
    let wgsl = material_program_wgsl(request.ir, request.fixture, &generated);
    let scene_wgsl = material_scene_product_wgsl(request.ir, &generated);
    validate_wgsl(&wgsl)?;
    validate_wgsl(&scene_wgsl)?;
    let identity = material_shader_identity(request.ir, request.fixture);
    let scene_identity = material_scene_shader_identity(request.ir);
    Ok(CompiledMaterialShader {
        shader_id: format!("material.generated.{}", request.ir.document_id.raw()),
        wgsl,
        scene_wgsl,
        resource_bindings,
        identity,
        scene_identity,
        output_target: request.ir.output_target,
    })
}

fn compiled_resource_bindings(ir: &MaterialIr) -> Vec<CompiledMaterialResourceBinding> {
    ir.required_resources
        .iter()
        .enumerate()
        .map(|(index, resource)| {
            let texture_dimension = match resource.reference.kind.as_str() {
                "asset.catalog.texture3d"
                | "asset.catalog.texture_3d"
                | "texture3d"
                | "texture_3d" => CompiledMaterialTextureDimension::D3,
                _ => CompiledMaterialTextureDimension::D2,
            };
            CompiledMaterialResourceBinding {
                node_id: resource.node_id.raw(),
                binding_key: resource.binding_key.clone(),
                bind_group: 1,
                texture_binding: (index as u32).saturating_mul(2),
                sampler_binding: (index as u32).saturating_mul(2).saturating_add(1),
                texture_dimension,
            }
        })
        .collect()
}

fn material_resource_declarations(bindings: &[CompiledMaterialResourceBinding]) -> String {
    if bindings.is_empty() {
        return String::new();
    }
    let mut lines = Vec::new();
    for binding in bindings {
        let texture_type = match binding.texture_dimension {
            CompiledMaterialTextureDimension::D2 => "texture_2d<f32>",
            CompiledMaterialTextureDimension::D3 => "texture_3d<f32>",
        };
        lines.push(format!(
            "@group({}) @binding({})\nvar {} : {};",
            binding.bind_group,
            binding.texture_binding,
            texture_binding_variable(binding),
            texture_type
        ));
        lines.push(format!(
            "@group({}) @binding({})\nvar {} : sampler;",
            binding.bind_group,
            binding.sampler_binding,
            sampler_binding_variable(binding)
        ));
    }
    let mut declarations = lines.join("\n");
    declarations.push('\n');
    declarations
}

fn texture_binding_variable(binding: &CompiledMaterialResourceBinding) -> String {
    format!("rw_material_texture_{}", binding.node_id)
}

fn sampler_binding_variable(binding: &CompiledMaterialResourceBinding) -> String {
    format!("rw_material_sampler_{}", binding.node_id)
}

fn validate_ir(ir: &MaterialIr) -> Result<(), MaterialShaderCompileError> {
    if ir.contract_version != MATERIAL_IR_CONTRACT_VERSION {
        return Err(MaterialShaderCompileError::UnsupportedIrVersion {
            found: ir.contract_version,
            expected: MATERIAL_IR_CONTRACT_VERSION,
        });
    }
    let output_nodes = ir
        .nodes
        .iter()
        .filter(|node| node.op == MaterialNodeOp::PbrOutput)
        .count();
    if output_nodes == 0 {
        return Err(MaterialShaderCompileError::MissingOutputNode);
    }
    if output_nodes > 1 {
        return Err(MaterialShaderCompileError::DuplicateOutputNode);
    }

    let mut outputs = BTreeSet::<(u64, String)>::new();
    for node in &ir.nodes {
        if !node.outputs.is_empty() && node.op == MaterialNodeOp::PbrOutput {
            return Err(MaterialShaderCompileError::InvalidNodeContract {
                node_id: node.node_id.raw(),
                message: "pbr.output must not declare value outputs".to_string(),
            });
        }
        for output in &node.outputs {
            outputs.insert((node.node_id.raw(), output.name.clone()));
        }
    }
    for node in &ir.nodes {
        for input in &node.inputs {
            if let MaterialIrInputSource::Connected {
                node_id,
                output_name,
            } = &input.source
            {
                if !outputs.contains(&(node_id.raw(), output_name.clone())) {
                    return Err(MaterialShaderCompileError::MissingConnectedOutput {
                        node_id: node.node_id.raw(),
                        input: input.name.clone(),
                        source_node_id: node_id.raw(),
                        output: output_name.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

struct WgslMaterialProgram {
    lines: Vec<String>,
    output: MaterialOutputExpressions,
    resource_bindings: Vec<CompiledMaterialResourceBinding>,
}

#[derive(Debug, Clone)]
struct MaterialOutputExpressions {
    base_color: String,
    roughness: String,
    metallic: String,
    normal_strength: String,
    emissive: String,
    opacity: String,
    material_channel: String,
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
    fn compile(
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

fn material_program_wgsl(
    ir: &MaterialIr,
    fixture: MaterialPreviewFixture,
    program: &WgslMaterialProgram,
) -> String {
    let generated_lines = if program.lines.is_empty() {
        String::new()
    } else {
        let mut lines = program.lines.join("\n");
        lines.push('\n');
        lines
    };
    let resource_declarations = material_resource_declarations(&program.resource_bindings);
    let output = &program.output;
    format!(
        r#"// generated by runenwerk material compiler v{MATERIAL_WGSL_COMPILER_CONTRACT_VERSION}
// document: {document_id}
// output_target: {output_target}
// fixture: {fixture}

struct VsOut {{
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>,
}};

struct MaterialEvalContext {{
    position : vec3<f32>,
    normal : vec3<f32>,
    distance : f32,
    material_channel : f32,
    density : f32,
    support : f32,
    wetness : f32,
}};

struct MaterialEvalOutput {{
    base_color : vec4<f32>,
    roughness : f32,
    metallic : f32,
    normal_strength : f32,
    emissive : vec3<f32>,
    opacity : f32,
    material_channel : f32,
}};

{resource_declarations}
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {{
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var out: VsOut;
    out.clip_position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
    out.uv = pos[vertex_index] * 0.5 + vec2<f32>(0.5, 0.5);
    return out;
}}

fn rw_hash31(value: f32) -> f32 {{
    return fract(sin(value) * 43758.5453123);
}}

fn rw_noise3(position: vec3<f32>) -> f32 {{
    let seed = dot(position, vec3<f32>(12.9898, 78.233, 37.719));
    return rw_hash31(seed);
}}

fn rw_fbm3(position: vec3<f32>) -> f32 {{
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    for (var octave = 0u; octave < 4u; octave = octave + 1u) {{
        value = value + rw_noise3(position * frequency) * amplitude;
        frequency = frequency * 2.03;
        amplitude = amplitude * 0.5;
    }}
    return clamp(value, 0.0, 1.0);
}}

fn rw_triplanar_weights(normal: vec3<f32>) -> vec3<f32> {{
    let weights = pow(abs(normal), vec3<f32>(4.0));
    let total = max(weights.x + weights.y + weights.z, 0.000001);
    return weights / total;
}}

fn evaluate_material(ctx: MaterialEvalContext) -> MaterialEvalOutput {{
{generated_lines}    var out: MaterialEvalOutput;
    out.base_color = {base_color};
    out.roughness = clamp({roughness}, 0.0, 1.0);
    out.metallic = clamp({metallic}, 0.0, 1.0);
    out.normal_strength = max({normal_strength}, 0.0);
    out.emissive = {emissive};
    out.opacity = clamp({opacity}, 0.0, 1.0);
    out.material_channel = {material_channel};
    return out;
}}

fn fixture_context(uv: vec2<f32>) -> MaterialEvalContext {{
    let centered = uv - vec2<f32>(0.5, 0.5);
    let distance = length(centered) - 0.38;
    var normal = normalize(vec3<f32>(centered, 0.58));
    var density = smoothstep(0.48, 0.18, length(centered));
    var support = 1.0;
    if {fixture_id}u == 1u {{
        density = step(abs(centered.x), 0.36) * step(abs(centered.y), 0.28);
        normal = normalize(vec3<f32>(sign(centered.x) * 0.35, sign(centered.y) * 0.35, 0.75));
    }}
    if {fixture_id}u == 2u {{
        density = smoothstep(0.04, 0.0, abs(centered.y + 0.18));
        normal = vec3<f32>(0.0, 1.0, 0.0);
    }}
    if {fixture_id}u == 3u {{
        support = max(density, step(abs(centered.x), 0.32) * step(abs(centered.y), 0.32));
    }}
    if {fixture_id}u == 4u {{
        density = 0.5 + 0.5 * sin((uv.x + uv.y) * 32.0);
        support = density;
    }}
    return MaterialEvalContext(
        vec3<f32>(centered, density),
        normal,
        distance,
        f32({fixture_id}u),
        density,
        support,
        clamp(uv.y, 0.0, 1.0),
    );
}}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {{
    let ctx = fixture_context(in.uv);
    let material = evaluate_material(ctx);
    let light_dir = normalize(vec3<f32>(0.35, 0.75, 0.55));
    let ndotl = max(dot(ctx.normal, light_dir), 0.12);
    let roughness_lift = mix(0.2, 0.55, material.roughness);
    let metal_tint = mix(material.base_color.rgb, vec3<f32>(0.95, 0.92, 0.85), material.metallic * 0.35);
    let shaded = metal_tint * (ndotl + roughness_lift) + material.emissive;
    return vec4<f32>(clamp(shaded, vec3<f32>(0.0), vec3<f32>(1.0)), material.opacity);
}}
"#,
        document_id = ir.document_id.raw(),
        output_target = output_target_label(ir.output_target),
        fixture = fixture.label(),
        fixture_id = fixture_id(fixture),
        resource_declarations = resource_declarations,
        base_color = output.base_color,
        roughness = output.roughness,
        metallic = output.metallic,
        normal_strength = output.normal_strength,
        emissive = output.emissive,
        opacity = output.opacity,
        material_channel = output.material_channel,
    )
}

fn material_scene_product_wgsl(ir: &MaterialIr, program: &WgslMaterialProgram) -> String {
    let generated_lines = if program.lines.is_empty() {
        String::new()
    } else {
        let mut lines = program.lines.join("\n");
        lines.push('\n');
        lines
    };
    let resource_declarations = material_resource_declarations(&program.resource_bindings);
    let output = &program.output;
    format!(
        r#"// generated by runenwerk material compiler v{MATERIAL_WGSL_COMPILER_CONTRACT_VERSION}
// scene entrypoint generated from material document: {document_id}
// output_target: {output_target}

struct EditorViewportSceneProductUniform {{
    surface : vec4<f32>,
    viewport : vec4<f32>,
    camera_position : vec4<f32>,
    camera_forward : vec4<f32>,
    camera_right : vec4<f32>,
    camera_up : vec4<f32>,
    object_transform : vec4<f32>,
    primitive_params_a : vec4<f32>,
    primitive_params_b : vec4<f32>,
    primitive_flags : vec4<u32>,
    primitive_slot_transforms : array<vec4<f32>, 64>,
    primitive_slot_params_a : array<vec4<f32>, 64>,
    primitive_slot_params_b : array<vec4<f32>, 64>,
    primitive_slot_flags : array<vec4<u32>, 64>,
}};

const MAX_PRIMITIVES : u32 = 64u;

@group(0) @binding(0)
var<uniform> u : EditorViewportSceneProductUniform;

struct VsOut {{
    @builtin(position) clip_position : vec4<f32>,
}};

struct MaterialEvalContext {{
    position : vec3<f32>,
    normal : vec3<f32>,
    distance : f32,
    material_channel : f32,
    density : f32,
    support : f32,
    wetness : f32,
}};

struct MaterialEvalOutput {{
    base_color : vec4<f32>,
    roughness : f32,
    metallic : f32,
    normal_strength : f32,
    emissive : vec3<f32>,
    opacity : f32,
    material_channel : f32,
}};

{resource_declarations}
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {{
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var out: VsOut;
    out.clip_position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
    return out;
}}

fn rw_hash31(value: f32) -> f32 {{
    return fract(sin(value) * 43758.5453123);
}}

fn rw_noise3(position: vec3<f32>) -> f32 {{
    let seed = dot(position, vec3<f32>(12.9898, 78.233, 37.719));
    return rw_hash31(seed);
}}

fn rw_fbm3(position: vec3<f32>) -> f32 {{
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    for (var octave = 0u; octave < 4u; octave = octave + 1u) {{
        value = value + rw_noise3(position * frequency) * amplitude;
        frequency = frequency * 2.03;
        amplitude = amplitude * 0.5;
    }}
    return clamp(value, 0.0, 1.0);
}}

fn rw_triplanar_weights(normal: vec3<f32>) -> vec3<f32> {{
    let weights = pow(abs(normal), vec3<f32>(4.0));
    let total = max(weights.x + weights.y + weights.z, 0.000001);
    return weights / total;
}}

fn evaluate_material(ctx: MaterialEvalContext) -> MaterialEvalOutput {{
{generated_lines}    var out: MaterialEvalOutput;
    out.base_color = {base_color};
    out.roughness = clamp({roughness}, 0.0, 1.0);
    out.metallic = clamp({metallic}, 0.0, 1.0);
    out.normal_strength = max({normal_strength}, 0.0);
    out.emissive = {emissive};
    out.opacity = clamp({opacity}, 0.0, 1.0);
    out.material_channel = {material_channel};
    return out;
}}

fn sdf_box(sample_pos: vec3<f32>, center: vec3<f32>, half_extents: vec3<f32>) -> f32 {{
    let q = abs(sample_pos - center) - half_extents;
    let outside = length(max(q, vec3<f32>(0.0, 0.0, 0.0)));
    let inside = min(max(q.x, max(q.y, q.z)), 0.0);
    return outside + inside;
}}

fn sdf_sphere(sample_pos: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {{
    return length(sample_pos - center) - radius;
}}

fn sdf_capsule(sample_pos: vec3<f32>, center: vec3<f32>, radius: f32, half_height: f32) -> f32 {{
    let q = sample_pos - center;
    let clamped_y = clamp(q.y, -half_height, half_height);
    let closest = vec3<f32>(0.0, clamped_y, 0.0);
    return length(q - closest) - radius;
}}

fn sdf_cylinder(sample_pos: vec3<f32>, center: vec3<f32>, radius: f32, half_height: f32) -> f32 {{
    let local = sample_pos - center;
    let d = vec2<f32>(length(local.xz) - radius, abs(local.y) - half_height);
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0, 0.0)));
}}

fn sdf_torus(sample_pos: vec3<f32>, center: vec3<f32>, major_radius: f32, minor_radius: f32) -> f32 {{
    let local = sample_pos - center;
    let q = vec2<f32>(length(local.xz) - major_radius, local.y);
    return length(q) - minor_radius;
}}

fn sdf_plane_slab(sample_pos: vec3<f32>, center: vec3<f32>, half_extents: vec3<f32>) -> f32 {{
    let slab_extents = vec3<f32>(max(half_extents.x, 0.05), max(min(half_extents.y, 0.05), 0.01), max(half_extents.z, 0.05));
    return sdf_box(sample_pos, center, slab_extents);
}}

fn primitive_count() -> u32 {{
    return min(u.primitive_flags.y, MAX_PRIMITIVES);
}}

fn sdf_primitive_slot(sample_pos: vec3<f32>, primitive_index: u32) -> f32 {{
    let slot = u.primitive_slot_flags[primitive_index];
    let center = u.primitive_slot_transforms[primitive_index].xyz;
    let primitive_kind = slot.x;
    let params_a = u.primitive_slot_params_a[primitive_index];
    let params_b = u.primitive_slot_params_b[primitive_index];
    if primitive_kind == 1u {{
        return sdf_sphere(sample_pos, center, max(params_a.w, 0.05));
    }}
    if primitive_kind == 2u {{
        return sdf_capsule(sample_pos, center, max(params_b.x, 0.05), max(params_b.y, 0.05));
    }}
    if primitive_kind == 3u {{
        return sdf_cylinder(sample_pos, center, max(params_b.x, 0.05), max(params_b.y, 0.05));
    }}
    if primitive_kind == 4u {{
        return sdf_torus(sample_pos, center, max(params_a.w * 1.5, 0.05), max(params_a.w * 0.5, 0.05));
    }}
    if primitive_kind == 5u {{
        return sdf_plane_slab(sample_pos, center, max(params_a.xyz, vec3<f32>(0.05, 0.05, 0.05)));
    }}
    return sdf_box(sample_pos, center, max(params_a.xyz, vec3<f32>(0.05, 0.05, 0.05)));
}}

fn scene_sdf(sample_pos: vec3<f32>) -> f32 {{
    var distance = 1e9;
    let count = primitive_count();
    var index = 0u;
    loop {{
        if index >= count {{
            break;
        }}
        distance = min(distance, sdf_primitive_slot(sample_pos, index));
        index = index + 1u;
    }}
    return distance;
}}

fn estimate_normal(sample_pos: vec3<f32>) -> vec3<f32> {{
    let e = 0.001;
    let nx = scene_sdf(sample_pos + vec3<f32>(e, 0.0, 0.0)) - scene_sdf(sample_pos - vec3<f32>(e, 0.0, 0.0));
    let ny = scene_sdf(sample_pos + vec3<f32>(0.0, e, 0.0)) - scene_sdf(sample_pos - vec3<f32>(0.0, e, 0.0));
    let nz = scene_sdf(sample_pos + vec3<f32>(0.0, 0.0, e)) - scene_sdf(sample_pos - vec3<f32>(0.0, 0.0, e));
    return normalize(vec3<f32>(nx, ny, nz));
}}

struct RaymarchResult {{
    hit : bool,
    distance : f32,
}};

fn march_scene(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> RaymarchResult {{
    var t = 0.0;
    var hit = false;
    var steps = 0u;
    loop {{
        if steps >= 96u {{
            break;
        }}
        let sample_pos = ray_origin + ray_dir * t;
        let distance = scene_sdf(sample_pos);
        if distance < 0.001 {{
            hit = true;
            break;
        }}
        t = t + distance;
        if t > 64.0 {{
            break;
        }}
        steps = steps + 1u;
    }}
    return RaymarchResult(hit, t);
}}

fn viewport_background() -> vec4<f32> {{
    return vec4<f32>(0.09, 0.10, 0.12, 1.0);
}}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {{
    let pixel = position.xy;
    let target_size = max(u.surface.xy, vec2<f32>(1.0, 1.0));
    let viewport_local = pixel / target_size;
    let ndc = vec2<f32>(viewport_local.x * 2.0 - 1.0, 1.0 - viewport_local.y * 2.0);
    if primitive_count() == 0u {{
        return viewport_background();
    }}
    let aspect = target_size.x / target_size.y;
    let tan_half_fov = tan(u.camera_position.w * 0.5);
    let ray_origin = u.camera_position.xyz;
    let ray_dir = normalize(
        u.camera_forward.xyz
        + u.camera_right.xyz * ndc.x * aspect * tan_half_fov
        + u.camera_up.xyz * ndc.y * tan_half_fov
    );
    let march = march_scene(ray_origin, ray_dir);
    if !march.hit {{
        return viewport_background();
    }}
    let sample_pos = ray_origin + ray_dir * march.distance;
    let normal = estimate_normal(sample_pos);
    let material = evaluate_material(MaterialEvalContext(
        sample_pos,
        normal,
        march.distance,
        0.0,
        1.0,
        1.0,
        0.0,
    ));
    let light_dir = normalize(vec3<f32>(0.5, 0.8, 0.35));
    let diff = max(dot(normal, light_dir), 0.1);
    let rim = pow(max(1.0 - max(dot(normal, -ray_dir), 0.0), 0.0), 2.0);
    let lit = material.base_color.rgb * diff + vec3<f32>(0.15, 0.20, 0.28) * rim + material.emissive;
    return vec4<f32>(clamp(lit, vec3<f32>(0.0), vec3<f32>(1.0)), material.opacity);
}}
"#,
        document_id = ir.document_id.raw(),
        output_target = output_target_label(ir.output_target),
        resource_declarations = resource_declarations,
        base_color = output.base_color,
        roughness = output.roughness,
        metallic = output.metallic,
        normal_strength = output.normal_strength,
        emissive = output.emissive,
        opacity = output.opacity,
        material_channel = output.material_channel,
    )
}

fn output_variable(node_id: NodeId, output_name: &str) -> String {
    format!("n{}_{}", node_id.raw(), sanitize_identifier(output_name))
}

fn sanitize_identifier(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() {
            out.push(byte as char);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() || out.as_bytes()[0].is_ascii_digit() {
        out.insert(0, '_');
    }
    out
}

fn wgsl_type(value_type: MaterialValueType) -> &'static str {
    match value_type {
        MaterialValueType::Float => "f32",
        MaterialValueType::Vec2 => "vec2<f32>",
        MaterialValueType::Vec3 => "vec3<f32>",
        MaterialValueType::Vec4 | MaterialValueType::Color => "vec4<f32>",
        MaterialValueType::Bool => "bool",
        MaterialValueType::ResourceTexture2D | MaterialValueType::ResourceTexture3D => "u32",
    }
}

fn literal_to_wgsl(
    literal: &MaterialLiteral,
    value_type: MaterialValueType,
) -> Result<String, MaterialShaderCompileError> {
    match (literal, value_type) {
        (MaterialLiteral::Bool(value), MaterialValueType::Bool) => Ok(value.to_string()),
        (MaterialLiteral::Float(value), MaterialValueType::Float) => Ok(wgsl_float(value)?),
        (MaterialLiteral::Vec2(values), MaterialValueType::Vec2) => Ok(wgsl_vec("vec2", values)?),
        (MaterialLiteral::Vec3(values), MaterialValueType::Vec3) => Ok(wgsl_vec("vec3", values)?),
        (MaterialLiteral::Vec4(values), MaterialValueType::Vec4) => Ok(wgsl_vec("vec4", values)?),
        (MaterialLiteral::Color(values), MaterialValueType::Color) => Ok(wgsl_vec("vec4", values)?),
        _ => Err(MaterialShaderCompileError::InvalidLiteral {
            value: literal.canonical_component(),
            expected_type: value_type.label(),
        }),
    }
}

fn canonical_value_to_wgsl(
    canonical: &str,
    value_type: MaterialValueType,
) -> Result<String, MaterialShaderCompileError> {
    match value_type {
        MaterialValueType::Float => canonical_float(canonical),
        MaterialValueType::Vec2 => canonical_vector_wgsl(canonical, "vec2", 2),
        MaterialValueType::Vec3 => canonical_vector_wgsl(canonical, "vec3", 3),
        MaterialValueType::Vec4 => canonical_vector_wgsl(canonical, "vec4", 4),
        MaterialValueType::Color => canonical_vector_wgsl(canonical, "vec4", 4),
        MaterialValueType::Bool => canonical_bool(canonical),
        MaterialValueType::ResourceTexture2D | MaterialValueType::ResourceTexture3D => {
            Err(MaterialShaderCompileError::InvalidLiteral {
                value: canonical.to_string(),
                expected_type: value_type.label(),
            })
        }
    }
}

fn canonical_float(canonical: &str) -> Result<String, MaterialShaderCompileError> {
    if let Some(value) = canonical.strip_prefix("integer:") {
        return wgsl_float(value);
    }
    if let Some(value) = length_prefixed_payload(canonical, "float")
        .or_else(|| length_prefixed_payload(canonical, "decimal"))
        .or_else(|| length_prefixed_payload(canonical, "text"))
    {
        return wgsl_float(value);
    }
    Err(MaterialShaderCompileError::InvalidLiteral {
        value: canonical.to_string(),
        expected_type: "float",
    })
}

fn canonical_bool(canonical: &str) -> Result<String, MaterialShaderCompileError> {
    if let Some(value) = canonical.strip_prefix("bool:") {
        return match value {
            "true" => Ok("true".to_string()),
            "false" => Ok("false".to_string()),
            _ => Err(MaterialShaderCompileError::InvalidLiteral {
                value: canonical.to_string(),
                expected_type: "bool",
            }),
        };
    }
    Err(MaterialShaderCompileError::InvalidLiteral {
        value: canonical.to_string(),
        expected_type: "bool",
    })
}

fn canonical_vector_wgsl(
    canonical: &str,
    wgsl_constructor: &str,
    count: usize,
) -> Result<String, MaterialShaderCompileError> {
    let values = if canonical.starts_with("text:") {
        let text = length_prefixed_payload(canonical, "text").ok_or_else(|| {
            MaterialShaderCompileError::InvalidLiteral {
                value: canonical.to_string(),
                expected_type: "vector",
            }
        })?;
        parse_text_vector(text, count)?
    } else {
        parse_canonical_vector(canonical, count)?
    };
    Ok(format!(
        "{wgsl_constructor}<f32>({})",
        values
            .iter()
            .map(|value| wgsl_float(value))
            .collect::<Result<Vec<_>, _>>()?
            .join(", ")
    ))
}

fn parse_text_vector(value: &str, count: usize) -> Result<Vec<String>, MaterialShaderCompileError> {
    let values = value
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if values.len() == count {
        Ok(values)
    } else {
        Err(MaterialShaderCompileError::InvalidLiteral {
            value: value.to_string(),
            expected_type: "vector",
        })
    }
}

fn parse_canonical_vector(
    canonical: &str,
    count: usize,
) -> Result<Vec<String>, MaterialShaderCompileError> {
    let mut cursor = 0usize;
    let label_end = canonical[cursor..].find(':').ok_or_else(|| {
        MaterialShaderCompileError::InvalidLiteral {
            value: canonical.to_string(),
            expected_type: "vector",
        }
    })? + cursor;
    let label = &canonical[cursor..label_end];
    if !matches!(label, "vec2" | "vec3" | "vec4" | "color") {
        return Err(MaterialShaderCompileError::InvalidLiteral {
            value: canonical.to_string(),
            expected_type: "vector",
        });
    }
    cursor = label_end + 1;
    let encoded_count = read_decimal_until_colon(canonical, &mut cursor)?;
    if encoded_count != count {
        return Err(MaterialShaderCompileError::InvalidLiteral {
            value: canonical.to_string(),
            expected_type: "vector",
        });
    }
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(read_length_prefixed_field(canonical, &mut cursor)?.to_string());
    }
    if cursor != canonical.len() {
        return Err(MaterialShaderCompileError::InvalidLiteral {
            value: canonical.to_string(),
            expected_type: "vector",
        });
    }
    Ok(values)
}

fn length_prefixed_payload<'a>(canonical: &'a str, label: &str) -> Option<&'a str> {
    let prefix = format!("{label}:");
    let mut cursor = prefix.len();
    if !canonical.starts_with(&prefix) {
        return None;
    }
    let payload = read_length_prefixed_field(canonical, &mut cursor).ok()?;
    if cursor == canonical.len() {
        Some(payload)
    } else {
        None
    }
}

fn read_length_prefixed_field<'a>(
    value: &'a str,
    cursor: &mut usize,
) -> Result<&'a str, MaterialShaderCompileError> {
    let length = read_decimal_until_colon(value, cursor)?;
    let end =
        cursor
            .checked_add(length)
            .ok_or_else(|| MaterialShaderCompileError::InvalidLiteral {
                value: value.to_string(),
                expected_type: "length-prefixed field",
            })?;
    if end > value.len() || !value.is_char_boundary(end) {
        return Err(MaterialShaderCompileError::InvalidLiteral {
            value: value.to_string(),
            expected_type: "length-prefixed field",
        });
    }
    let payload = &value[*cursor..end];
    *cursor = end;
    if *cursor < value.len() {
        if value.as_bytes()[*cursor] != b':' {
            return Err(MaterialShaderCompileError::InvalidLiteral {
                value: value.to_string(),
                expected_type: "length-prefixed field",
            });
        }
        *cursor += 1;
    }
    Ok(payload)
}

fn read_decimal_until_colon(
    value: &str,
    cursor: &mut usize,
) -> Result<usize, MaterialShaderCompileError> {
    let start = *cursor;
    while *cursor < value.len() && value.as_bytes()[*cursor] != b':' {
        *cursor += 1;
    }
    if *cursor == value.len() || *cursor == start {
        return Err(MaterialShaderCompileError::InvalidLiteral {
            value: value.to_string(),
            expected_type: "length",
        });
    }
    let length = value[start..*cursor].parse::<usize>().map_err(|_| {
        MaterialShaderCompileError::InvalidLiteral {
            value: value.to_string(),
            expected_type: "length",
        }
    })?;
    *cursor += 1;
    Ok(length)
}

fn wgsl_vec<const N: usize>(
    constructor: &str,
    values: &[String; N],
) -> Result<String, MaterialShaderCompileError> {
    Ok(format!(
        "{constructor}<f32>({})",
        values
            .iter()
            .map(|value| wgsl_float(value))
            .collect::<Result<Vec<_>, _>>()?
            .join(", ")
    ))
}

fn wgsl_float(value: &str) -> Result<String, MaterialShaderCompileError> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| MaterialShaderCompileError::InvalidLiteral {
            value: value.to_string(),
            expected_type: "float",
        })?;
    if !parsed.is_finite() {
        return Err(MaterialShaderCompileError::InvalidLiteral {
            value: value.to_string(),
            expected_type: "finite float",
        });
    }
    if value.contains('.') || value.contains('e') || value.contains('E') {
        Ok(value.to_string())
    } else {
        Ok(format!("{value}.0"))
    }
}

fn validate_wgsl(source: &str) -> Result<(), MaterialShaderCompileError> {
    let module = naga::front::wgsl::parse_str(source)
        .map_err(|error| MaterialShaderCompileError::InvalidWgsl(error.to_string()))?;
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .map(|_| ())
    .map_err(|error| MaterialShaderCompileError::InvalidWgsl(format!("{error:?}")))
}

fn material_shader_identity(ir: &MaterialIr, fixture: MaterialPreviewFixture) -> String {
    let mut encoder = CanonicalShaderIdentityEncoder::default();
    encoder.field(
        "compiler_contract",
        &MATERIAL_WGSL_COMPILER_CONTRACT_VERSION.to_string(),
    );
    encoder.field("ir_contract", &ir.contract_version.to_string());
    encoder.field("document", &ir.document_id.raw().to_string());
    encoder.field("target", output_target_label(ir.output_target));
    encoder.field("fixture", fixture.label());
    encoder.number("node_count", ir.nodes.len() as u64);
    for node in &ir.nodes {
        encoder.field("node_id", &node.node_id.raw().to_string());
        encoder.field("node_op", node.op.label());
        for input in &node.inputs {
            encoder.field("input_name", &input.name);
            encoder.field("input_type", input.value_type.label());
            match &input.source {
                MaterialIrInputSource::Connected {
                    node_id,
                    output_name,
                } => {
                    encoder.field("input_source", "connected");
                    encoder.field("input_source_node", &node_id.raw().to_string());
                    encoder.field("input_source_output", output_name);
                }
                MaterialIrInputSource::Constant(literal) => {
                    encoder.field("input_source", "constant");
                    encoder.field("input_literal", &literal.canonical_component());
                }
                MaterialIrInputSource::NodeValue {
                    key,
                    canonical_value,
                } => {
                    encoder.field("input_source", "node_value");
                    encoder.field("input_value_key", key);
                    encoder.field("input_value", canonical_value);
                }
            }
        }
        for output in &node.outputs {
            encoder.field("output_name", &output.name);
            encoder.field("output_type", output.value_type.label());
        }
        for value in &node.values {
            encoder.field("value_key", &value.key);
            encoder.field("value_value", &value.canonical_value);
        }
    }
    encoder.number("resource_count", ir.required_resources.len() as u64);
    for resource in &ir.required_resources {
        encoder.field("resource_node", &resource.node_id.raw().to_string());
        encoder.field("resource_key", &resource.binding_key);
        encoder.field("resource_ref", &resource.reference.canonical_component());
    }
    encoder.finish_hex()
}

fn material_scene_shader_identity(ir: &MaterialIr) -> String {
    let mut encoder = CanonicalShaderIdentityEncoder::default();
    encoder.field(
        "compiler_contract",
        &MATERIAL_WGSL_COMPILER_CONTRACT_VERSION.to_string(),
    );
    encoder.field("entrypoint", "scene");
    encoder.field("ir_contract", &ir.contract_version.to_string());
    encoder.field("document", &ir.document_id.raw().to_string());
    encoder.field("target", output_target_label(ir.output_target));
    for node in &ir.nodes {
        encoder.field("node_id", &node.node_id.raw().to_string());
        encoder.field("node_op", node.op.label());
        for value in &node.values {
            encoder.field("value_key", &value.key);
            encoder.field("value_value", &value.canonical_value);
        }
    }
    encoder.finish_hex()
}

#[derive(Default)]
struct CanonicalShaderIdentityEncoder {
    bytes: Vec<u8>,
}

impl CanonicalShaderIdentityEncoder {
    fn number(&mut self, label: &str, value: u64) {
        self.field(label, &value.to_string());
    }

    fn field(&mut self, label: &str, value: &str) {
        self.bytes.extend_from_slice(label.as_bytes());
        self.bytes.push(b'=');
        self.bytes
            .extend_from_slice(value.len().to_string().as_bytes());
        self.bytes.push(b':');
        self.bytes.extend_from_slice(value.as_bytes());
        self.bytes.push(b'\n');
    }

    fn finish_hex(self) -> String {
        self.bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

fn output_target_label(output_target: MaterialOutputTarget) -> &'static str {
    match output_target {
        MaterialOutputTarget::PbrPreview => "pbr_preview",
        MaterialOutputTarget::FieldMaterialChannel => "field_material_channel",
        MaterialOutputTarget::RenderMaterial => "render_material",
    }
}

fn fixture_id(fixture: MaterialPreviewFixture) -> u32 {
    match fixture {
        MaterialPreviewFixture::Sphere => 0,
        MaterialPreviewFixture::Box => 1,
        MaterialPreviewFixture::Plane => 2,
        MaterialPreviewFixture::SdfPrimitive => 3,
        MaterialPreviewFixture::FieldMaterial => 4,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialShaderCompileError {
    UnsupportedIrVersion {
        found: u32,
        expected: u32,
    },
    MissingOutputNode,
    DuplicateOutputNode,
    MissingInput {
        node_id: u64,
        input: String,
    },
    MissingNodeValue {
        node_id: u64,
        key: String,
    },
    MissingResourceBinding {
        node_id: u64,
        key: String,
    },
    MissingConnectedOutput {
        node_id: u64,
        input: String,
        source_node_id: u64,
        output: String,
    },
    InvalidNodeContract {
        node_id: u64,
        message: String,
    },
    InvalidLiteral {
        value: String,
        expected_type: &'static str,
    },
    InvalidWgsl(String),
}

impl fmt::Display for MaterialShaderCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedIrVersion { found, expected } => write!(
                formatter,
                "unsupported material IR contract version {found}; expected {expected}"
            ),
            Self::MissingOutputNode => formatter.write_str("material IR has no pbr.output node"),
            Self::DuplicateOutputNode => {
                formatter.write_str("material IR has multiple pbr.output nodes")
            }
            Self::MissingInput { node_id, input } => {
                write!(
                    formatter,
                    "material node {node_id} is missing input '{input}'"
                )
            }
            Self::MissingNodeValue { node_id, key } => {
                write!(
                    formatter,
                    "material node {node_id} is missing value '{key}'"
                )
            }
            Self::MissingResourceBinding { node_id, key } => write!(
                formatter,
                "material node {node_id} is missing resolved resource binding '{key}'"
            ),
            Self::MissingConnectedOutput {
                node_id,
                input,
                source_node_id,
                output,
            } => write!(
                formatter,
                "material node {node_id} input '{input}' references missing output {source_node_id}.{output}"
            ),
            Self::InvalidNodeContract { node_id, message } => {
                write!(
                    formatter,
                    "material node {node_id} has invalid compiler contract: {message}"
                )
            }
            Self::InvalidLiteral {
                value,
                expected_type,
            } => write!(
                formatter,
                "material literal '{value}' cannot be compiled as {expected_type}"
            ),
            Self::InvalidWgsl(message) => write!(formatter, "generated WGSL is invalid: {message}"),
        }
    }
}

impl Error for MaterialShaderCompileError {}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::{
        CyclePolicy, GraphDefinition, GraphId, GraphMetadataEntry, GraphValue, NodeDefinition,
        NodeId, PortDefinition, PortDirection, PortId, PortTypeId,
    };
    use material_graph::{
        MaterialGraphDocument, MaterialGraphDocumentId, MaterialNodeCatalog, lower_material_graph,
    };
    use resource_ref::ResourceRef;

    fn ir() -> MaterialIr {
        let document = MaterialGraphDocument::new(
            MaterialGraphDocumentId::new(7),
            "mat",
            GraphDefinition::new(
                GraphId::new(1),
                "mat",
                CyclePolicy::RejectDirectedCycles,
                [NodeDefinition::new(
                    NodeId::new(1),
                    "pbr.output",
                    [PortDefinition::new(
                        PortId::new(1),
                        "base_color",
                        PortDirection::Input,
                        PortTypeId::new(1),
                    )],
                )],
                [],
            ),
            MaterialOutputTarget::RenderMaterial,
        );
        lower_material_graph(&document, &MaterialNodeCatalog::first_slice())
            .product
            .expect("formed")
            .executable_ir
            .expect("ir")
    }

    fn all_first_slice_ops_ir() -> MaterialIr {
        let catalog = MaterialNodeCatalog::first_slice();
        let nodes = catalog
            .descriptors()
            .enumerate()
            .map(|(index, descriptor)| {
                let mut node = NodeDefinition::new(
                    NodeId::new((index + 1) as u64),
                    descriptor.key.clone(),
                    [],
                );
                if descriptor.key == "texture.sample_2d" {
                    node = node.with_values([GraphMetadataEntry::new(
                        material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
                        GraphValue::resource(
                            ResourceRef::new("asset.catalog.texture2d", "albedo")
                                .expect("resource ref"),
                        ),
                    )]);
                }
                if descriptor.key == "texture.sample_3d" {
                    node = node.with_values([GraphMetadataEntry::new(
                        material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
                        GraphValue::resource(
                            ResourceRef::new("asset.catalog.texture3d", "volume")
                                .expect("resource ref"),
                        ),
                    )]);
                }
                node
            })
            .collect::<Vec<_>>();
        let document = MaterialGraphDocument::new(
            MaterialGraphDocumentId::new(9),
            "all_ops",
            GraphDefinition::new(
                GraphId::new(1),
                "all_ops",
                CyclePolicy::RejectDirectedCycles,
                nodes,
                [],
            ),
            MaterialOutputTarget::RenderMaterial,
        );
        let lowering = lower_material_graph(&document, &catalog);
        assert!(
            !lowering.report.has_blocking_issues(),
            "{:?}",
            lowering.report.issues()
        );
        lowering.product.expect("formed").executable_ir.expect("ir")
    }

    #[test]
    fn compiler_emits_valid_deterministic_wgsl() {
        let ir = ir();
        let first = compile_material_shader(MaterialShaderCompileRequest {
            ir: &ir,
            fixture: MaterialPreviewFixture::Sphere,
        })
        .expect("compile");
        let second = compile_material_shader(MaterialShaderCompileRequest {
            ir: &ir,
            fixture: MaterialPreviewFixture::Sphere,
        })
        .expect("compile");

        assert_eq!(first.wgsl, second.wgsl);
        assert_eq!(first.scene_wgsl, second.scene_wgsl);
        assert_eq!(first.identity, second.identity);
        assert_eq!(first.scene_identity, second.scene_identity);
        assert!(first.wgsl.contains("fn evaluate_material"));
        assert!(first.scene_wgsl.contains("fn evaluate_material"));
        assert!(
            first
                .scene_wgsl
                .contains("EditorViewportSceneProductUniform")
        );
        assert!(first.wgsl.contains("fn fs_main"));
    }

    #[test]
    fn fixture_changes_shader_identity() {
        let ir = ir();
        let sphere = compile_material_shader(MaterialShaderCompileRequest {
            ir: &ir,
            fixture: MaterialPreviewFixture::Sphere,
        })
        .expect("compile");
        let plane = compile_material_shader(MaterialShaderCompileRequest {
            ir: &ir,
            fixture: MaterialPreviewFixture::Plane,
        })
        .expect("compile");

        assert_ne!(sphere.identity, plane.identity);
    }

    #[test]
    fn compiler_generates_semantics_for_every_first_slice_node() {
        let ir = all_first_slice_ops_ir();
        let shader = compile_material_shader(MaterialShaderCompileRequest {
            ir: &ir,
            fixture: MaterialPreviewFixture::FieldMaterial,
        })
        .expect("all first-slice ops should compile");

        assert!(shader.wgsl.contains("rw_noise3"));
        assert!(shader.wgsl.contains("rw_fbm3"));
        assert!(shader.wgsl.contains("texture_2d<f32>"));
        assert!(shader.wgsl.contains("texture_3d<f32>"));
        assert!(shader.wgsl.contains("textureSample"));
        assert!(shader.wgsl.contains("rw_triplanar_weights"));
        assert!(shader.scene_wgsl.contains("texture_2d<f32>"));
        assert!(shader.scene_wgsl.contains("textureSample"));
        assert!(shader.scene_wgsl.contains("march_scene"));
    }

    #[test]
    fn compiler_rejects_missing_output_node() {
        let mut ir = ir();
        ir.nodes.retain(|node| node.op != MaterialNodeOp::PbrOutput);

        let error = compile_material_shader(MaterialShaderCompileRequest {
            ir: &ir,
            fixture: MaterialPreviewFixture::Sphere,
        })
        .expect_err("missing output should fail");

        assert_eq!(error, MaterialShaderCompileError::MissingOutputNode);
    }
}
