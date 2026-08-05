use engine::plugins::gpu;
use std::num::NonZeroU64;

const WGSL: &str = r#"
struct ViewUniform {
    transform: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> view: ViewUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> @builtin(position) vec4<f32> {
    return view.transform * vec4<f32>(input.position, 1.0);
}

@fragment
fn fragment_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

fn main() {
    let owner = gpu::GpuProgramSourceOwnerId::allocate().expect("source owner should allocate");
    let identity = gpu::GpuProgramSourceIdentity::new(
        owner,
        gpu::GpuProgramSourceKey::new("examples.render.triangle")
            .expect("source key should be valid"),
        gpu::GpuProgramSourceRevision::try_from_raw(1).expect("source revision should be nonzero"),
    );
    let mut registry =
        gpu::GpuProgramSourceRegistry::new(4, 16 * 1024).expect("registry should construct");
    let source = registry
        .admit_wgsl(
            identity,
            WGSL,
            gpu::GpuProgramSourceProvenance::new(
                "gpu-render-contract-example",
                Some("render example".to_owned()),
            )
            .expect("source provenance should be valid"),
        )
        .expect("source should admit");

    let visibility =
        gpu::GpuShaderStages::new([gpu::GpuShaderStage::Vertex, gpu::GpuShaderStage::Fragment])
            .expect("uniform visibility should be nonempty");
    let uniform = gpu::GpuBindingDeclaration::new(
        gpu::GpuBindingKey::try_new(0, 0).expect("binding key should fit u32"),
        visibility,
        gpu::GpuBindingKind::uniform_buffer(false, NonZeroU64::new(64)),
        None,
        "view-uniform",
        gpu::GpuBindingProvenance::new("gpu-render-contract-example", None)
            .expect("binding provenance should be valid"),
    )
    .expect("uniform declaration should construct");
    let interface =
        gpu::GpuProgramInterfaceDescriptor::new([uniform]).expect("interface should construct");
    let program = gpu::GpuProgramDescriptor::new(
        source,
        interface.clone(),
        [
            gpu::GpuEntryPointDescriptor::new(
                entry_point("vertex_main"),
                gpu::GpuShaderStage::Vertex,
                interface.clone(),
            ),
            gpu::GpuEntryPointDescriptor::new(
                entry_point("fragment_main"),
                gpu::GpuShaderStage::Fragment,
                interface.clone(),
            ),
        ],
    )
    .expect("program should construct");

    let vertex_layout = gpu::GpuVertexBufferLayoutDescriptor::new(
        0,
        12,
        gpu::GpuVertexStepMode::Vertex,
        [gpu::GpuVertexAttribute::new(
            0,
            0,
            gpu::GpuVertexFormat::Float32x3,
        )],
    )
    .expect("vertex layout should construct");
    let vertex_input = gpu::GpuVertexInputStateDescriptor::new([vertex_layout])
        .expect("vertex input state should construct");
    let color_target = gpu::GpuColorTargetStateDescriptor::new(
        gpu::GpuTextureFormat::Rgba8UnormSrgb,
        gpu::GpuBlendMode::Replace,
        gpu::GpuColorWriteMask::ALL,
    )
    .expect("color target should construct");
    let state = gpu::GpuRenderPipelineStateDescriptor::new(
        vertex_input,
        Some(gpu::GpuFragmentOutputStateDescriptor::new([color_target])),
        gpu::GpuPrimitiveStateDescriptor::default(),
        None,
        gpu::GpuMultisampleStateDescriptor::default(),
    )
    .expect("render state should construct");
    let layout = gpu::GpuPipelineLayoutDescriptor::from_interface(&interface)
        .expect("layout should derive from the interface");
    let pipeline = gpu::GpuRenderPipelineDescriptor::new(
        program,
        gpu::GpuRenderEntryPoints::new(
            entry_point("vertex_main"),
            Some(entry_point("fragment_main")),
        ),
        state,
        layout,
        empty_specialization(),
        gpu::GpuCapabilityRequirements::new(),
    )
    .expect("render pipeline should construct");

    compare_stage_io(&pipeline);
}

fn compare_stage_io(pipeline: &gpu::GpuRenderPipelineDescriptor) {
    let expected_vertex = pipeline
        .expected_vertex_input_signature()
        .expect("expected vertex signature should derive");
    let observed_vertex = gpu::GpuObservedVertexInputSignature::new(
        entry_point("vertex_main"),
        [location(0, gpu::GpuShaderIoScalarClass::Float, 3)],
        [],
    )
    .expect("observed vertex signature should normalize");
    gpu::compare_vertex_input_signatures(&expected_vertex, &observed_vertex)
        .expect("vertex stage IO should agree");

    let expected_fragment = pipeline
        .expected_fragment_output_signature()
        .expect("expected fragment signature should derive")
        .expect("pipeline selects a fragment stage");
    let observed_fragment = gpu::GpuObservedFragmentOutputSignature::new(
        entry_point("fragment_main"),
        [location(0, gpu::GpuShaderIoScalarClass::Float, 4)],
        [],
    )
    .expect("observed fragment signature should normalize");
    gpu::compare_fragment_output_signatures(&expected_fragment, &observed_fragment)
        .expect("fragment stage IO should agree");
}

fn location(
    location: u32,
    scalar_class: gpu::GpuShaderIoScalarClass,
    vector_width: u8,
) -> gpu::GpuShaderIoLocation {
    gpu::GpuShaderIoLocation::new(
        location,
        gpu::GpuShaderIoValueType::try_new(scalar_class, vector_width)
            .expect("shader IO type should be valid"),
    )
}

fn entry_point(name: &str) -> gpu::GpuEntryPointName {
    gpu::GpuEntryPointName::new(name).expect("entry-point name should be valid")
}

fn empty_specialization() -> gpu::GpuSpecializationValueSet {
    let schema =
        gpu::GpuSpecializationSchema::new([]).expect("empty specialization schema should be valid");
    gpu::GpuSpecializationValueSet::new(schema, [])
        .expect("empty specialization value set should be valid")
}
