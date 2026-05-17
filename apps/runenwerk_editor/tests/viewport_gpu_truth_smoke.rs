use asset::{ArtifactCacheKey, asset_artifact_id, asset_id, asset_source_id};
use editor_viewport::ViewportPresentationState;
use engine::App;
use engine::plugins::render::Gfx;
use engine::plugins::render::inspect::{
    CaptureStage, CaptureTextureClass, RenderCaptureSelector, RenderCaptureTerminalCode,
    RenderCapturedTextureState, RenderDebugFrameReportState, RenderPassProvenanceState,
    RenderPixelCoordinate,
};
use material_graph::{
    FormedMaterialProduct, MaterialCacheKey, MaterialGraphDocumentId, MaterialOutputTarget,
    MaterialProductId,
};
use resource_ref::ResourceRef;
use runenwerk_editor::material_lab::{
    EditorMaterialPreviewProduct, MaterialRendererParameterProfile, ResolvedMaterialResource,
};
use runenwerk_editor::runtime::app::EDITOR_MATERIAL_PREVIEW_SHADER_ID;
use runenwerk_editor::runtime::resources::EditorHostResource;
use runenwerk_editor::runtime::viewport::{
    MAIN_VIEWPORT_ID, SCENE_COLOR_PRODUCT_ID, ViewportPresentationStateResource,
};
use std::sync::Arc;
use texture::{
    Ktx2TextureMetadata, TextureDescriptor, TextureDimension, TextureExtent, TexturePixelFormat,
    TextureProductId,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::{ActiveEventLoop, EventLoop};
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::window::Window;

const SURFACE_RESOURCE_ID: &str = "surface.color";

#[test]
#[cfg_attr(
    target_os = "macos",
    ignore = "requires macOS main-thread-safe windowed GPU harness; set RUNENWERK_ENABLE_GPU_SMOKE=1 and RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE=1 for manual runs"
)]
#[cfg_attr(
    not(target_os = "macos"),
    ignore = "requires a real windowed GPU device"
)]
fn viewport_gpu_truth_smoke() {
    if !gpu_smoke_enabled() {
        eprintln!("RUNENWERK_ENABLE_GPU_SMOKE is not enabled; skipping windowed GPU smoke test");
        return;
    }
    if cfg!(target_os = "macos") && !macos_main_thread_smoke_enabled() {
        eprintln!(
            "RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE is not enabled; skipping macOS main-thread GPU smoke path"
        );
        return;
    }
    if cfg!(target_os = "macos") && !running_on_macos_main_thread() {
        eprintln!("macOS GPU smoke requires the test body to run on the process main thread");
        return;
    }

    let window = create_hidden_window();
    let gfx = Gfx::new(window).expect("gfx should initialize for smoke test");

    let mut app = runenwerk_editor::runtime::build_headless_app();
    app.world_mut().insert_resource(gfx);

    app.update_render_debug_control(|debug_control| {
        debug_control.provenance_enabled = true;
        debug_control.capture_enabled = false;
        debug_control.readback_enabled = false;
        debug_control.artifact_export_enabled = false;
    });

    let mut app = app
        .run_for_frames(3)
        .expect("windowed GPU smoke warmup should render frames");

    let provenance = app
        .world()
        .resource::<RenderPassProvenanceState>()
        .expect("pass provenance state should exist");
    let viewport_record = provenance
        .records
        .iter()
        .find(|record| scene_product_record_targets_scene_color(record))
        .unwrap_or_else(|| {
            panic!(
                "viewport pass provenance should exist before capture; records: {:?}",
                provenance
                    .records
                    .iter()
                    .map(|record| (
                        record.pass_label.as_str(),
                        record.pass_id.as_str(),
                        record.shader_id.as_str()
                    ))
                    .collect::<Vec<_>>()
            )
        })
        .clone();
    let ui_record = provenance
        .records
        .iter()
        .find(|record| record.shader_id == "builtin:ui_composite")
        .expect("ui pass provenance should exist before capture")
        .clone();
    let scene_target_id = viewport_record
        .render_targets
        .first()
        .cloned()
        .expect("viewport pass should expose the resolved scene-color target id");

    app.update_render_debug_config(|debug_config| {
        debug_config.clear();
        debug_config.capture_selectors = vec![
            RenderCaptureSelector {
                flow_id: Some(viewport_record.flow_id.clone()),
                pass_id: Some(viewport_record.pass_id.clone()),
                stage: CaptureStage::After,
                resource_id: scene_target_id.clone(),
                texture_class: CaptureTextureClass::ColorTarget,
            },
            RenderCaptureSelector {
                flow_id: Some(ui_record.flow_id.clone()),
                pass_id: Some(ui_record.pass_id.clone()),
                stage: CaptureStage::Before,
                resource_id: SURFACE_RESOURCE_ID.to_string(),
                texture_class: CaptureTextureClass::ImportedTexture,
            },
            RenderCaptureSelector {
                flow_id: Some(ui_record.flow_id.clone()),
                pass_id: Some(ui_record.pass_id.clone()),
                stage: CaptureStage::After,
                resource_id: SURFACE_RESOURCE_ID.to_string(),
                texture_class: CaptureTextureClass::ImportedTexture,
            },
        ];
    });
    app.update_render_debug_control(|debug_control| {
        debug_control.provenance_enabled = true;
        debug_control.capture_enabled = true;
        debug_control.readback_enabled = true;
        debug_control.artifact_export_enabled = false;
    });

    let mut app = app
        .run_for_frames(3)
        .expect("windowed GPU smoke test app should render frames");

    let captures = app
        .world()
        .resource::<RenderCapturedTextureState>()
        .expect("captured texture state should exist");
    let debug_report = app
        .world()
        .resource::<RenderDebugFrameReportState>()
        .expect("render debug report state should exist")
        .latest
        .clone();

    let viewport_after = captures
        .find(
            viewport_record.flow_id.as_str(),
            viewport_record.pass_id.as_str(),
            CaptureStage::After,
            scene_target_id.as_str(),
        )
        .unwrap_or_else(|| {
            panic!(
                "viewport pass after-capture should exist; captures={:?}; report={:?}",
                captures.captures, debug_report
            )
        });
    let ui_before = captures
        .find(
            ui_record.flow_id.as_str(),
            ui_record.pass_id.as_str(),
            CaptureStage::Before,
            SURFACE_RESOURCE_ID,
        )
        .expect("ui pass before-capture should exist");
    let ui_after = captures
        .find(
            ui_record.flow_id.as_str(),
            ui_record.pass_id.as_str(),
            CaptureStage::After,
            SURFACE_RESOURCE_ID,
        )
        .expect("ui pass after-capture should exist");

    assert!(
        viewport_after.terminal.code == RenderCaptureTerminalCode::Completed
            && ui_before.terminal.code == RenderCaptureTerminalCode::Completed
            && ui_after.terminal.code == RenderCaptureTerminalCode::Completed,
        "all required captures must complete without readback errors: viewport_after={:?}, ui_before={:?}, ui_after={:?}, debug_report={:?}",
        viewport_after.terminal,
        ui_before.terminal,
        ui_after.terminal,
        debug_report
    );
    assert!(
        viewport_after.bytes_rgba8.is_some()
            && ui_before.bytes_rgba8.is_some()
            && ui_after.bytes_rgba8.is_some(),
        "all required captures must include rgba pixels"
    );

    let center_viewport = viewport_after
        .sample_center_rgba8()
        .expect("viewport after capture should provide center pixel");
    let center_ui_before = ui_before
        .sample_center_rgba8()
        .expect("ui before capture should provide center pixel");
    let center_ui_after = ui_after
        .sample_center_rgba8()
        .expect("ui after capture should provide center pixel");

    let inside_point = RenderPixelCoordinate {
        x: (viewport_after.width / 2).max(1),
        y: (viewport_after.height / 2).max(1),
    };
    let outside_point = RenderPixelCoordinate {
        x: 16_u32.min(viewport_after.width.saturating_sub(1)),
        y: 16_u32.min(viewport_after.height.saturating_sub(1)),
    };
    let inside_before = ui_before
        .sample_pixel_rgba8(inside_point)
        .expect("inside point should be sampleable before ui pass");
    let inside_after = ui_after
        .sample_pixel_rgba8(inside_point)
        .expect("inside point should be sampleable after ui pass");
    let outside_before = ui_before
        .sample_pixel_rgba8(outside_point)
        .expect("outside point should be sampleable before ui pass");
    let outside_after = ui_after
        .sample_pixel_rgba8(outside_point)
        .expect("outside point should be sampleable after ui pass");

    assert!(
        center_viewport != center_ui_before
            || center_ui_before != center_ui_after
            || inside_before != inside_after
            || outside_before != outside_after,
        "expected viewport product and ui composite passes to affect sampled points"
    );

    let provenance = app
        .world()
        .resource::<RenderPassProvenanceState>()
        .expect("pass provenance state should exist");
    let viewport_record = provenance
        .records
        .iter()
        .find(|record| scene_product_record_targets_scene_color(record))
        .expect("viewport pass provenance should exist");
    let ui_record = provenance
        .records
        .iter()
        .find(|record| record.shader_id == "builtin:ui_composite")
        .expect("ui pass provenance should exist");

    assert!(!viewport_record.shader_id.is_empty());
    assert!(!ui_record.shader_id.is_empty());
    let scene_color_product_id = SCENE_COLOR_PRODUCT_ID.0.to_string();
    assert!(
        viewport_record
            .render_targets
            .iter()
            .any(|target| target.contains(scene_color_product_id.as_str())),
        "viewport pass should render the scene-color product target"
    );
    app = assert_material_scene_bundle_and_texture_group_gpu_truth(app);
    drop(app);
}

fn scene_product_record_targets_scene_color(
    record: &engine::plugins::render::inspect::RenderPassProvenanceRecord,
) -> bool {
    let scene_color_product_id = SCENE_COLOR_PRODUCT_ID.0.to_string();
    record
        .render_targets
        .iter()
        .any(|target| target.contains(scene_color_product_id.as_str()))
}

fn assert_material_scene_bundle_and_texture_group_gpu_truth(mut app: App) -> App {
    let preview_shader_path = write_gpu_truth_shader(
        "wr021-material-preview-2d.wgsl",
        material_texture_sample_shader(),
    );
    let scene_shader_path = write_gpu_truth_shader(
        "wr021-material-scene-2d.wgsl",
        material_texture_sample_shader(),
    );
    let texture_path = write_gpu_truth_ktx2_texture(
        "wr021-material-texture-2d.ktx2",
        4,
        4,
        1,
        [38, 164, 92, 255],
    );
    let preview = gpu_truth_material_preview_product(
        70,
        &preview_shader_path,
        &scene_shader_path,
        &texture_path,
        TextureDimension::Texture2D,
        asset::AssetKind::Texture2D,
        "asset.catalog.texture_2d",
        "texture_2d",
    );
    app = assert_material_preview_product_pixels(app, preview, "Texture2D");

    let preview_shader_path = write_gpu_truth_shader(
        "wr021-material-preview-3d.wgsl",
        material_texture_3d_sample_shader(),
    );
    let scene_shader_path = write_gpu_truth_shader(
        "wr021-material-scene-3d.wgsl",
        material_texture_3d_sample_shader(),
    );
    let texture_path = write_gpu_truth_ktx2_texture(
        "wr021-material-texture-3d.ktx2",
        4,
        4,
        2,
        [168, 72, 220, 255],
    );
    let preview = gpu_truth_material_preview_product(
        71,
        &preview_shader_path,
        &scene_shader_path,
        &texture_path,
        TextureDimension::Texture3DVolume,
        asset::AssetKind::Texture3DVolume,
        "asset.catalog.texture_3d",
        "texture_3d",
    );
    assert_material_preview_product_pixels(app, preview, "Texture3D")
}

fn assert_material_preview_product_pixels(
    mut app: App,
    preview: EditorMaterialPreviewProduct,
    label: &str,
) -> App {
    let preview_product_id = preview.viewport_product_id;
    let preview_shader_path = preview.shader_path.clone();
    let scene_shader_path = preview.scene_shader_path.clone();
    let texture_artifact_paths = preview
        .resolved_resources
        .iter()
        .map(|resource| resource.artifact_path.clone())
        .collect::<Vec<_>>();

    {
        let host = app
            .world_mut()
            .resource_mut::<EditorHostResource>()
            .expect("editor host should exist");
        host.app
            .material_lab_runtime_mut()
            .set_active_preview(preview);
    }
    app.update_render_debug_control(|debug_control| {
        debug_control.provenance_enabled = true;
        debug_control.capture_enabled = false;
        debug_control.readback_enabled = false;
        debug_control.artifact_export_enabled = false;
    });

    let mut app = app
        .run_for_frames(6)
        .expect("material gpu smoke warmup should render frames");
    {
        let presentations = app
            .world_mut()
            .resource_mut::<ViewportPresentationStateResource>()
            .expect("viewport presentation state should exist");
        let mut state = presentations
            .state_for(MAIN_VIEWPORT_ID)
            .cloned()
            .unwrap_or_else(|| {
                ViewportPresentationState::new(MAIN_VIEWPORT_ID, preview_product_id)
            });
        state.select_primary_product(preview_product_id);
        presentations.upsert_state(state);
    }

    app = app
        .run_for_frames(4)
        .expect("selected material preview should produce provenance");
    let provenance = app
        .world()
        .resource::<RenderPassProvenanceState>()
        .expect("pass provenance state should exist");
    let scene_record = provenance
        .records
        .iter()
        .find(|record| record.shader_id == scene_shader_path)
        .expect("scene pass should consume the generated scene material bundle shader")
        .clone();
    let material_preview_record = provenance
        .records
        .iter()
        .find(|record| record.shader_id == EDITOR_MATERIAL_PREVIEW_SHADER_ID)
        .expect("material preview producer should consume the generated preview shader")
        .clone();
    let scene_target_id = scene_record
        .render_targets
        .first()
        .cloned()
        .expect("scene material pass should expose its product target");
    let material_preview_target_id = material_preview_record
        .render_targets
        .first()
        .cloned()
        .expect("material preview pass should expose its product target");

    app.update_render_debug_config(|debug_config| {
        debug_config.clear();
        debug_config.capture_selectors = vec![
            RenderCaptureSelector {
                flow_id: Some(scene_record.flow_id.clone()),
                pass_id: Some(scene_record.pass_id.clone()),
                stage: CaptureStage::After,
                resource_id: scene_target_id.clone(),
                texture_class: CaptureTextureClass::ColorTarget,
            },
            RenderCaptureSelector {
                flow_id: Some(material_preview_record.flow_id.clone()),
                pass_id: Some(material_preview_record.pass_id.clone()),
                stage: CaptureStage::After,
                resource_id: material_preview_target_id.clone(),
                texture_class: CaptureTextureClass::ColorTarget,
            },
        ];
    });
    app.update_render_debug_control(|debug_control| {
        debug_control.provenance_enabled = true;
        debug_control.capture_enabled = true;
        debug_control.readback_enabled = true;
        debug_control.artifact_export_enabled = false;
    });

    let app = app
        .run_for_frames(3)
        .expect("material gpu smoke capture should render frames");
    let captures = app
        .world()
        .resource::<RenderCapturedTextureState>()
        .expect("captured texture state should exist");
    let debug_report = app
        .world()
        .resource::<RenderDebugFrameReportState>()
        .expect("render debug report state should exist")
        .latest
        .clone();
    let scene_capture = captures
        .find(
            scene_record.flow_id.as_str(),
            scene_record.pass_id.as_str(),
            CaptureStage::After,
            scene_target_id.as_str(),
        )
        .expect("generated scene material capture should exist");
    let material_preview_capture = captures
        .find(
            material_preview_record.flow_id.as_str(),
            material_preview_record.pass_id.as_str(),
            CaptureStage::After,
            material_preview_target_id.as_str(),
        )
        .expect("generated material preview capture should exist");
    assert!(
        scene_capture.terminal.code == RenderCaptureTerminalCode::Completed
            && material_preview_capture.terminal.code == RenderCaptureTerminalCode::Completed,
        "material scene and preview captures must complete: scene={:?}, preview={:?}, debug_report={:?}",
        scene_capture.terminal,
        material_preview_capture.terminal,
        debug_report
    );
    let scene_pixel = scene_capture
        .sample_center_rgba8()
        .expect("generated scene capture should include pixels");
    let preview_pixel = material_preview_capture
        .sample_center_rgba8()
        .expect("generated material preview capture should include pixels");
    assert_eq!(
        scene_pixel[3], 255,
        "scene material output should be opaque"
    );
    assert_eq!(
        preview_pixel[3], 255,
        "material preview output should be opaque"
    );
    assert!(
        scene_pixel[..3].iter().any(|channel| *channel > 0)
            && preview_pixel[..3].iter().any(|channel| *channel > 0),
        "generated {label} material shaders should visibly sample GPU-resident group-1 texture bindings: scene={:?}, preview={:?}",
        scene_pixel,
        preview_pixel
    );
    eprintln!(
        "WR-021 GPU proof {label}: scene_shader={scene_shader_path}, preview_shader={preview_shader_path}, textures={texture_artifact_paths:?}, scene_pixel={scene_pixel:?}, preview_pixel={preview_pixel:?}"
    );
    app
}

fn gpu_smoke_enabled() -> bool {
    std::env::var("RUNENWERK_ENABLE_GPU_SMOKE")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn macos_main_thread_smoke_enabled() -> bool {
    std::env::var("RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn write_gpu_truth_shader(file_name: &str, source: &str) -> String {
    let directory = std::env::temp_dir()
        .join("runenwerk-wr021-gpu-proof")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory).expect("gpu proof shader directory should be writable");
    let path = directory.join(file_name);
    std::fs::write(&path, source).expect("gpu proof shader should be writable");
    path.to_string_lossy().into_owned()
}

fn write_gpu_truth_ktx2_texture(
    file_name: &str,
    width: u32,
    height: u32,
    depth: u32,
    texel: [u8; 4],
) -> String {
    let directory = std::env::temp_dir()
        .join("runenwerk-wr021-gpu-proof")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory).expect("gpu proof texture directory should be writable");
    let path = directory.join(file_name);
    let bytes = build_rgba8_ktx2(width, height, depth, texel);
    std::fs::write(&path, bytes).expect("gpu proof KTX2 texture should be writable");
    path.to_string_lossy().into_owned()
}

fn material_texture_sample_shader() -> &'static str {
    r#"
struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>,
};

@group(1) @binding(0)
var rw_material_texture_7 : texture_2d<f32>;

@group(1) @binding(1)
var rw_material_sampler_7 : sampler;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var out: VsOut;
    out.clip_position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
    out.uv = out.clip_position.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let color = textureSample(rw_material_texture_7, rw_material_sampler_7, input.uv);
    return vec4<f32>(max(color.rgb, vec3<f32>(0.02, 0.02, 0.02)), 1.0);
}
"#
}

fn material_texture_3d_sample_shader() -> &'static str {
    r#"
struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>,
};

@group(1) @binding(0)
var rw_material_texture_7 : texture_3d<f32>;

@group(1) @binding(1)
var rw_material_sampler_7 : sampler;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var out: VsOut;
    out.clip_position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
    out.uv = out.clip_position.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

fn rw_triplanar_weights(normal: vec3<f32>) -> vec3<f32> {
    let n = max(abs(normal), vec3<f32>(0.0001, 0.0001, 0.0001));
    return n / max(n.x + n.y + n.z, 0.0001);
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let weights = rw_triplanar_weights(normalize(vec3<f32>(0.35, 0.5, 0.8)));
    let sample = textureSample(rw_material_texture_7, rw_material_sampler_7, vec3<f32>(input.uv, weights.z));
    return vec4<f32>(max(sample.rgb * weights.z, vec3<f32>(0.02, 0.02, 0.02)), 1.0);
}
"#
}

fn gpu_truth_material_preview_product(
    product_id: u64,
    preview_shader_path: &str,
    scene_shader_path: &str,
    texture_path: &str,
    texture_dimension: TextureDimension,
    asset_kind: asset::AssetKind,
    resource_kind: &str,
    dimension_label: &str,
) -> EditorMaterialPreviewProduct {
    let product = FormedMaterialProduct::new(
        MaterialProductId::new(product_id),
        MaterialGraphDocumentId::new(11),
        MaterialOutputTarget::RenderMaterial,
        MaterialCacheKey::new(format!("gpu-truth-material-cache-{product_id}")),
    );
    let depth = match texture_dimension {
        TextureDimension::Texture2D => 1,
        TextureDimension::Texture3DVolume => 2,
    };
    let descriptor = TextureDescriptor::new(
        TextureProductId::new(product_id + 5),
        "GPU truth texture",
        texture_dimension,
        TextureExtent::new(4, 4, depth),
    );
    let texture_length = std::fs::metadata(texture_path)
        .expect("gpu proof texture metadata should exist")
        .len();
    let mip_count = descriptor.mip_count;
    let descriptor_hash = descriptor.descriptor_hash().to_string();
    let descriptor = descriptor.with_ktx2_metadata(
        Ktx2TextureMetadata::new(
            TexturePixelFormat::Rgba8Unorm,
            mip_count,
            descriptor_hash,
            "rev-1",
        )
        .with_byte_layout(texture_length, [4 * 4 * depth as u64 * 4]),
    );
    let resource = ResolvedMaterialResource {
        node_id: graph::NodeId::new(7),
        binding_key: "albedo".to_string(),
        reference: ResourceRef::new(resource_kind, format!("gpu-truth-texture-{product_id}"))
            .expect("test resource ref should be valid"),
        artifact_id: asset_artifact_id(product_id + 501),
        artifact_path: texture_path.to_string(),
        kind: asset_kind,
        cache_key: ArtifactCacheKey::new(format!("gpu-truth-texture-cache-{product_id}")),
        descriptor,
        artifact_revision: "rev-1".to_string(),
        dimension: dimension_label.to_string(),
        color_space: "linear".to_string(),
        sampler_policy: "linear_repeat".to_string(),
        residency_identity: format!("gpu-truth-texture-residency-{product_id}"),
    };
    EditorMaterialPreviewProduct::new(
        asset_id(product_id + 101),
        asset_source_id(product_id + 102),
        asset_artifact_id(product_id + 103),
        ArtifactCacheKey::new(format!("gpu-truth-artifact-cache-{product_id}")),
        product,
        MaterialRendererParameterProfile::RenderMaterial,
        asset_artifact_id(product_id + 104),
        ArtifactCacheKey::new(format!("gpu-truth-preview-shader-cache-{product_id}")),
        preview_shader_path,
        "gpu-truth-preview-shader",
        asset_artifact_id(product_id + 105),
        ArtifactCacheKey::new(format!("gpu-truth-scene-shader-cache-{product_id}")),
        scene_shader_path,
        "gpu-truth-scene-shader",
        [resource],
    )
}

fn build_rgba8_ktx2(width: u32, height: u32, depth: u32, texel: [u8; 4]) -> Vec<u8> {
    let format = ktx2::Format::R8G8B8A8_UNORM;
    let (basic, type_size) = ktx2::dfd::Basic::from_format(format).expect("rgba8 dfd should build");
    let dfd_block = ktx2::dfd::Block::Basic(basic);
    let dfd_block_bytes = dfd_block.to_vec();
    let dfd_total_size = 4 + dfd_block_bytes.len();
    let level_index_offset = ktx2::Header::LENGTH;
    let dfd_offset = level_index_offset + ktx2::LevelIndex::LENGTH;
    let after_dfd = dfd_offset + dfd_total_size;
    let level_data_offset = (after_dfd + 3) / 4 * 4;
    let texel_count = width as usize * height as usize * depth.max(1) as usize;
    let level_data_size = texel_count * 4;
    let mut bytes = vec![0u8; level_data_offset + level_data_size];

    let header = ktx2::Header {
        format: Some(format),
        type_size,
        pixel_width: width,
        pixel_height: height,
        pixel_depth: if depth > 1 { depth } else { 0 },
        layer_count: 0,
        face_count: 1,
        level_count: 1,
        supercompression_scheme: None,
        index: ktx2::Index {
            dfd_byte_offset: dfd_offset as u32,
            dfd_byte_length: dfd_total_size as u32,
            kvd_byte_offset: 0,
            kvd_byte_length: 0,
            sgd_byte_offset: 0,
            sgd_byte_length: 0,
        },
    };
    bytes[..ktx2::Header::LENGTH].copy_from_slice(&header.as_bytes());
    let level_index = ktx2::LevelIndex {
        byte_offset: level_data_offset as u64,
        byte_length: level_data_size as u64,
        uncompressed_byte_length: level_data_size as u64,
    };
    bytes[level_index_offset..level_index_offset + ktx2::LevelIndex::LENGTH]
        .copy_from_slice(&level_index.as_bytes());
    bytes[dfd_offset..dfd_offset + 4].copy_from_slice(&(dfd_total_size as u32).to_le_bytes());
    bytes[dfd_offset + 4..dfd_offset + 4 + dfd_block_bytes.len()].copy_from_slice(&dfd_block_bytes);
    for index in 0..texel_count {
        let start = level_data_offset + index * 4;
        bytes[start..start + 4].copy_from_slice(&texel);
    }
    bytes
}

#[cfg(target_os = "macos")]
fn running_on_macos_main_thread() -> bool {
    unsafe extern "C" {
        fn pthread_main_np() -> std::os::raw::c_int;
    }

    unsafe { pthread_main_np() != 0 }
}

#[cfg(not(target_os = "macos"))]
fn running_on_macos_main_thread() -> bool {
    true
}

fn create_hidden_window() -> Arc<Window> {
    struct HiddenWindowBootstrap {
        attrs: winit::window::WindowAttributes,
        window: Option<Arc<Window>>,
        error: Option<String>,
    }

    impl ApplicationHandler for HiddenWindowBootstrap {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            match event_loop.create_window(self.attrs.clone()) {
                Ok(window) => self.window = Some(Arc::new(window)),
                Err(err) => self.error = Some(err.to_string()),
            }
            event_loop.exit();
        }

        fn window_event(
            &mut self,
            _event_loop: &ActiveEventLoop,
            _window_id: winit::window::WindowId,
            _event: winit::event::WindowEvent,
        ) {
        }
    }

    let event_loop = create_smoke_event_loop();
    let attrs = Window::default_attributes()
        .with_title("runenwerk viewport gpu smoke")
        .with_visible(false)
        .with_inner_size(PhysicalSize::new(1280, 720));
    let mut bootstrap = HiddenWindowBootstrap {
        attrs,
        window: None,
        error: None,
    };
    event_loop
        .run_app(&mut bootstrap)
        .expect("window bootstrap loop should run");

    if let Some(error) = bootstrap.error {
        panic!("hidden smoke-test window should be created: {error}");
    }

    bootstrap
        .window
        .expect("window bootstrap should capture the created window")
}

fn create_smoke_event_loop() -> EventLoop<()> {
    #[cfg(target_os = "windows")]
    {
        let mut builder = EventLoop::builder();
        builder.with_any_thread(true);
        return builder.build().expect("event loop should initialize");
    }

    #[cfg(not(target_os = "windows"))]
    {
        EventLoop::new().expect("event loop should initialize")
    }
}
