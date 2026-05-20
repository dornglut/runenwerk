use asset::{
    ArtifactCacheKey, ArtifactPayloadKind, AssetArtifactDescriptor, AssetCatalog, AssetKind,
    AssetRecord, AssetSourceDescriptor, SourceHash, asset_artifact_id, asset_id, asset_source_id,
};
use editor_core::{DocumentId, DocumentKind};
use editor_persistence::{
    ProjectFileV3, SceneEntityRecordV2, SceneFileV2, SceneMaterialAssignmentsRecord,
    SceneMaterialSlotRecord, SceneMaterialSourceRefRecord, ScenePrimitiveKind,
    ScenePrimitiveRecord, SceneTransformRecord, SdfPrimitiveMaterialSlotAssignmentRecord,
};
use editor_viewport::ViewportPresentationState;
use engine::App;
use engine::plugins::render::inspect::{
    CaptureStage, CaptureTextureClass, RenderCaptureSelector, RenderCaptureTerminalCode,
    RenderCapturedTextureState, RenderDebugFrameReportState, RenderPassProvenanceRecord,
    RenderPassProvenanceState, RenderPixelCoordinate, deterministic_capture_filename,
};
use engine::plugins::render::{
    Gfx, MaterialPreviewFixture, MaterialShaderCompileRequest, compile_material_shader,
};
use graph::{
    CyclePolicy, GraphDefinition, GraphId, GraphMetadataEntry, GraphValue, NodeDefinition, NodeId,
    PortDefinition, PortDirection, PortId, PortTypeId,
};
use material_graph::{
    FormedMaterialProduct, MaterialGraphDocument, MaterialGraphDocumentId,
    MaterialGraphSourceFileV2, MaterialNodeCatalog, MaterialOutputTarget, MaterialProductId,
    lower_material_graph,
};
use resource_ref::ResourceRef;
use runenwerk_editor::asset_pipeline::EditorAssetProjectSession;
use runenwerk_editor::editor_app::RunenwerkEditorApp;
use runenwerk_editor::editor_runtime::register_mvp_component_types;
use runenwerk_editor::material_lab::{
    EditorMaterialPreviewProduct, MaterialRendererParameterProfile,
    prepared_material_contribution_for_preview, resolve_material_resources,
};
use runenwerk_editor::persistence::apply_scene_file_to_runtime;
use runenwerk_editor::runtime::app::EDITOR_MATERIAL_PREVIEW_SHADER_ID;
use runenwerk_editor::runtime::resources::EditorHostResource;
use runenwerk_editor::runtime::viewport::{
    MAIN_VIEWPORT_ID, SCENE_COLOR_PRODUCT_ID, ViewportPresentationStateResource,
};
use runenwerk_editor::shell::{
    EditorSurfaceProviderRegistry, RunenwerkEditorShellState, SurfaceProviderBuildContext,
    SurfaceSessionState,
};
use runenwerk_editor::texture_preview::{TexturePreviewProofMetadata, prepare_texture_preview};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use texture::{
    Ktx2TextureMetadata, TextureDescriptor, TextureDimension, TextureExtent, TexturePixelFormat,
    TextureProductId,
};
use ui_render_data::ProductSurfaceTextureBindingSource;
use ui_theme::ThemeTokens;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::{ActiveEventLoop, EventLoop};
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::window::Window;

const SURFACE_RESOURCE_ID: &str = "surface.color";
const WR028_CLOSEOUT_DIR: &str = "docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding";
const WR028_TEXTURE_2D_PIXEL: [u8; 4] = [108, 210, 162, 255];
const WR028_TEXTURE_3D_PIXEL: [u8; 4] = [153, 103, 173, 255];

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
    configure_wr028_sdf_two_slot_scene(&mut app);
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
        debug_control.artifact_export_enabled = true;
        debug_control.artifact_output_dir = wr028_closeout_artifact_path("artifacts/captures");
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
    let sdf_samples = assert_wr028_sdf_two_slot_scene_pixels(viewport_after);

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
    app = assert_wr028_texture_viewer_product_surface_gpu_truth(app);
    eprintln!(
        "WR-028 SDF pixel proof: left_entity=201 slot=0 pixel={:?} rgba={:?}; right_entity=202 slot=1 pixel={:?} rgba={:?}; background pixel={:?} rgba={:?}",
        sdf_samples.left_pixel,
        sdf_samples.left_rgba,
        sdf_samples.right_pixel,
        sdf_samples.right_rgba,
        sdf_samples.background_pixel,
        sdf_samples.background_rgba
    );
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

#[derive(Debug, Clone, Copy)]
struct Wr028SdfPixelSamples {
    left_pixel: RenderPixelCoordinate,
    left_rgba: [u8; 4],
    right_pixel: RenderPixelCoordinate,
    right_rgba: [u8; 4],
    background_pixel: RenderPixelCoordinate,
    background_rgba: [u8; 4],
}

fn configure_wr028_sdf_two_slot_scene(app: &mut App) {
    let left_texture_path = write_gpu_truth_ktx2_texture(
        "wr028-scene-table-source-texture-left.ktx2",
        4,
        4,
        1,
        [38, 164, 92, 255],
    );
    let left_proof = source_backed_texture_material_preview_product(
        72,
        &left_texture_path,
        TextureDimension::Texture2D,
        asset::AssetKind::Texture2D,
        "texture.sample_2d",
        "wr028.scene_table_texture_left",
        "SceneTableLeft",
    );
    let right_texture_path = write_gpu_truth_ktx2_texture(
        "wr028-scene-table-source-texture-right.ktx2",
        4,
        4,
        1,
        [168, 72, 220, 255],
    );
    let right_proof = source_backed_texture_material_preview_product(
        73,
        &right_texture_path,
        TextureDimension::Texture2D,
        asset::AssetKind::Texture2D,
        "texture.sample_2d",
        "wr028.scene_table_texture_right",
        "SceneTableRight",
    );
    let scene_file = wr028_two_sdf_primitive_scene_file(&left_proof.preview, &right_proof.preview);
    let host = app
        .world_mut()
        .resource_mut::<EditorHostResource>()
        .expect("editor host should exist");
    register_mvp_component_types(host.app.runtime_mut());
    apply_scene_file_to_runtime(host.app.runtime_mut(), &scene_file)
        .expect("WR-028 SDF proof scene should load into the empty editor runtime");
    let material_runtime = host.app.material_lab_runtime_mut();
    material_runtime.set_active_preview(right_proof.preview);
    material_runtime.set_active_preview(left_proof.preview);
}

fn wr028_two_sdf_primitive_scene_file(
    default_preview: &EditorMaterialPreviewProduct,
    assigned_preview: &EditorMaterialPreviewProduct,
) -> SceneFileV2 {
    let left_transform = SceneTransformRecord {
        translation: [-1.25, 0.0, 0.0],
        ..Default::default()
    };
    let right_transform = SceneTransformRecord {
        translation: [1.25, 0.0, 0.0],
        ..Default::default()
    };
    let sphere = ScenePrimitiveRecord {
        kind: ScenePrimitiveKind::Sphere,
        sphere_radius: 0.75,
        ..Default::default()
    };
    let mut default_slot = SceneMaterialSlotRecord::default_generated();
    default_slot.material_asset_id = Some(default_preview.asset_id.raw());
    default_slot.source_ref = Some(SceneMaterialSourceRefRecord::new(
        default_preview.asset_id.raw(),
        default_preview.source_id.raw(),
    ));
    let mut second_slot = SceneMaterialSlotRecord::default_generated();
    second_slot.slot_id = 2;
    second_slot.palette_entry_id = 2;
    second_slot.display_name = "WR-028 Slot 1".to_string();
    second_slot.is_default = false;
    second_slot.material_asset_id = Some(assigned_preview.asset_id.raw());
    second_slot.source_ref = Some(SceneMaterialSourceRefRecord::new(
        assigned_preview.asset_id.raw(),
        assigned_preview.source_id.raw(),
    ));

    SceneFileV2::new(vec![
        SceneEntityRecordV2::new(201, "WR-028 Left SDF Slot 0", None, left_transform, sphere),
        SceneEntityRecordV2::new(
            202,
            "WR-028 Right SDF Slot 1",
            None,
            right_transform,
            sphere,
        ),
    ])
    .with_material_assignments(SceneMaterialAssignmentsRecord::new(
        [default_slot, second_slot],
        [SdfPrimitiveMaterialSlotAssignmentRecord::new(202, 2)],
    ))
}

fn assert_wr028_sdf_two_slot_scene_pixels(
    capture: &engine::plugins::render::inspect::RenderCapturedTexture,
) -> Wr028SdfPixelSamples {
    let left = dominant_pixel_in_region(capture, 0, capture.width / 2, |rgba| {
        rgba[1] as i32 - rgba[0].max(rgba[2]) as i32
    })
    .expect("left SDF primitive should produce a green source-texture slot-0 pixel");
    let right = dominant_pixel_in_region(capture, capture.width / 2, capture.width, |rgba| {
        rgba[2] as i32 - rgba[1] as i32
    })
    .expect("right SDF primitive should produce a blue source-texture slot-1 pixel");
    let background_pixel = RenderPixelCoordinate {
        x: 16_u32.min(capture.width.saturating_sub(1)),
        y: 16_u32.min(capture.height.saturating_sub(1)),
    };
    let background_rgba = capture
        .sample_pixel_rgba8(background_pixel)
        .expect("background point should be sampleable");

    assert!(
        left.1[1] > left.1[0].saturating_add(12) && left.1[1] > left.1[2].saturating_add(12),
        "left SDF sample must be green-dominant from source-backed material slot 0: {:?}",
        left
    );
    assert!(
        right.1[2] > right.1[1].saturating_add(12) && right.1[0] > right.1[1].saturating_add(12),
        "right SDF sample must be purple/blue-dominant from source-backed material slot 1: {:?}",
        right
    );
    assert_ne!(
        left.1, right.1,
        "two SDF material slots must produce different scene pixels"
    );

    Wr028SdfPixelSamples {
        left_pixel: left.0,
        left_rgba: left.1,
        right_pixel: right.0,
        right_rgba: right.1,
        background_pixel,
        background_rgba,
    }
}

fn dominant_pixel_in_region(
    capture: &engine::plugins::render::inspect::RenderCapturedTexture,
    x_start: u32,
    x_end: u32,
    score: impl Fn([u8; 4]) -> i32,
) -> Option<(RenderPixelCoordinate, [u8; 4], i32)> {
    let mut best = None;
    let y_start = capture.height / 4;
    let y_end = capture.height.saturating_mul(3) / 4;
    for y in y_start..y_end.max(y_start + 1) {
        for x in x_start..x_end.max(x_start + 1) {
            let pixel = RenderPixelCoordinate { x, y };
            let rgba = capture.sample_pixel_rgba8(pixel)?;
            if rgba[3] == 0 {
                continue;
            }
            let pixel_score = score(rgba);
            if best
                .as_ref()
                .map(|(_, _, best_score)| pixel_score > *best_score)
                .unwrap_or(true)
            {
                best = Some((pixel, rgba, pixel_score));
            }
        }
    }
    best
}

fn assert_material_scene_bundle_and_texture_group_gpu_truth(mut app: App) -> App {
    let texture_path = write_gpu_truth_ktx2_texture(
        "wr028-material-source-edit-texture-2d.ktx2",
        4,
        4,
        1,
        [38, 164, 92, 255],
    );
    let proof = source_backed_texture_material_preview_product(
        70,
        &texture_path,
        TextureDimension::Texture2D,
        asset::AssetKind::Texture2D,
        "texture.sample_2d",
        "wr028.source_edit_texture_2d",
        "Texture2D",
    );
    app = assert_source_backed_material_preview_product_pixels(app, proof, "Texture2D");

    let texture_path = write_gpu_truth_ktx2_texture(
        "wr028-material-source-edit-texture-3d.ktx2",
        4,
        4,
        2,
        [168, 72, 220, 255],
    );
    let proof = source_backed_texture_material_preview_product(
        71,
        &texture_path,
        TextureDimension::Texture3DVolume,
        asset::AssetKind::Texture3DVolume,
        "texture.sample_3d",
        "wr028.source_edit_texture_3d",
        "Texture3D",
    );
    assert_source_backed_material_preview_product_pixels(app, proof, "Texture3D")
}

#[derive(Debug, Clone)]
struct SourceBackedMaterialGpuProof {
    preview: EditorMaterialPreviewProduct,
    source_path: String,
    product_identity_before: String,
    product_identity_after: String,
    preview_shader_identity_before: String,
    preview_shader_identity_after: String,
    scene_shader_identity_before: String,
    scene_shader_identity_after: String,
    material_table_identity_before: String,
    material_table_identity_after: String,
}

fn source_backed_texture_material_preview_product(
    product_id: u64,
    texture_path: &str,
    texture_dimension: TextureDimension,
    asset_kind: asset::AssetKind,
    texture_descriptor_key: &str,
    stable_texture_id: &str,
    label: &str,
) -> SourceBackedMaterialGpuProof {
    let before_texture_id = format!("{stable_texture_id}.before");
    let mut document = texture_material_source_document(
        MaterialGraphDocumentId::new(product_id + 900),
        format!("WR-028 {label} source-backed material"),
        texture_descriptor_key,
        &before_texture_id,
    );
    let source_path = write_gpu_truth_material_source(
        format!("wr028-material-source-edit-{label}.material.ron").as_str(),
        &MaterialGraphSourceFileV2::from_document(&document),
    );
    let source_payload = std::fs::read_to_string(&source_path)
        .expect("source-backed material proof should reload source file bytes");
    let loaded_source: MaterialGraphSourceFileV2 = ron::de::from_str(&source_payload)
        .expect("source-backed material proof should parse MaterialGraphSourceFileV2");
    document = loaded_source
        .into_document()
        .expect("source-backed material proof should form a source document");

    let before_product = lower_texture_material_source_product(&document, product_id);
    let before_compiled = compile_material_shader(MaterialShaderCompileRequest {
        ir: before_product
            .executable_ir
            .as_ref()
            .expect("before source material should include IR"),
        fixture: MaterialPreviewFixture::Sphere,
    })
    .expect("before source material shader should compile");

    let material_asset_id = asset_id(product_id + 301);
    let material_source_id = asset_source_id(product_id + 302);
    let mut catalog = source_backed_texture_catalog(
        product_id,
        stable_texture_id,
        texture_path,
        texture_dimension,
        asset_kind,
    );
    catalog.insert_asset_record(
        AssetRecord::new(
            material_asset_id,
            format!("wr028_source_backed_material_{product_id}"),
            format!("WR-028 source-backed material {product_id}"),
            AssetKind::MaterialGraph,
        )
        .with_primary_source(material_source_id),
    );
    catalog.insert_source(
        AssetSourceDescriptor::new(
            material_source_id,
            material_asset_id,
            AssetKind::MaterialGraph,
            repo_relative_path(Path::new(&source_path)),
        )
        .with_hash(SourceHash::new(
            "manual",
            format!("wr028-source-edit-{product_id}"),
        )),
    );
    let mut workflow_app =
        material_graph_source_edit_workflow_app(catalog, material_asset_id, document.clone());
    workflow_app
        .apply_material_surface_action(editor_shell::MaterialSurfaceAction::PickTextureResource {
            node_id: graph::NodeId::new(7),
            key: material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF.to_string(),
            stable_id: stable_texture_id.to_string(),
        })
        .expect("source-backed texture pick should flow through material workflow");
    document = workflow_app
        .load_material_graph_document_for_asset(material_asset_id)
        .expect("source-backed material workflow edit should persist to source file");

    let after_product = lower_texture_material_source_product(&document, product_id);
    let after_ir = after_product
        .executable_ir
        .as_ref()
        .expect("after source material should include IR");
    let after_compiled = compile_material_shader(MaterialShaderCompileRequest {
        ir: after_ir,
        fixture: MaterialPreviewFixture::Sphere,
    })
    .expect("after source material shader should compile");
    assert_ne!(
        before_product.cache_key, after_product.cache_key,
        "MaterialGraphSourceFileV2 texture edit must change formed material product identity"
    );
    assert_ne!(
        before_compiled.identity, after_compiled.identity,
        "MaterialGraphSourceFileV2 texture edit must change generated preview shader identity"
    );
    assert_ne!(
        before_compiled.scene_identity, after_compiled.scene_identity,
        "MaterialGraphSourceFileV2 texture edit must change generated scene shader identity"
    );

    let preview_shader_path = write_gpu_truth_shader(
        format!("wr028-material-source-edit-preview-{label}.wgsl").as_str(),
        after_compiled.wgsl.as_str(),
    );
    let scene_shader_path = write_gpu_truth_shader(
        format!("wr028-material-source-edit-scene-{label}.wgsl").as_str(),
        after_compiled.scene_wgsl.as_str(),
    );
    let resolved_resources =
        resolve_material_resources(workflow_app.asset_catalog_runtime().catalog(), after_ir)
            .expect("source-backed material proof should resolve catalog texture resources");
    let preview = EditorMaterialPreviewProduct::new(
        asset_id(product_id + 101),
        asset_source_id(product_id + 102),
        asset_artifact_id(product_id + 103),
        ArtifactCacheKey::new(format!("wr028-source-material-artifact-cache-{product_id}")),
        after_product,
        MaterialRendererParameterProfile::RenderMaterial,
        asset_artifact_id(product_id + 104),
        ArtifactCacheKey::new(format!(
            "wr028-source-material-preview-shader-cache:{}",
            after_compiled.identity
        )),
        preview_shader_path,
        after_compiled.identity.clone(),
        asset_artifact_id(product_id + 105),
        ArtifactCacheKey::new(format!(
            "wr028-source-material-scene-shader-cache:{}",
            after_compiled.scene_identity
        )),
        scene_shader_path,
        after_compiled.scene_identity.clone(),
        resolved_resources,
    );
    let before_preview = EditorMaterialPreviewProduct::new(
        asset_id(product_id + 201),
        asset_source_id(product_id + 202),
        asset_artifact_id(product_id + 203),
        ArtifactCacheKey::new(format!(
            "wr028-source-material-before-artifact-cache-{product_id}"
        )),
        before_product.clone(),
        MaterialRendererParameterProfile::RenderMaterial,
        asset_artifact_id(product_id + 204),
        ArtifactCacheKey::new(format!(
            "wr028-source-material-before-preview-shader-cache:{}",
            before_compiled.identity
        )),
        "",
        before_compiled.identity.clone(),
        asset_artifact_id(product_id + 205),
        ArtifactCacheKey::new(format!(
            "wr028-source-material-before-scene-shader-cache:{}",
            before_compiled.scene_identity
        )),
        "",
        before_compiled.scene_identity.clone(),
        [],
    );
    let material_table_identity_before =
        prepared_material_contribution_for_preview(&before_preview)
            .scene_bundle
            .expect("before source material contribution should include scene bundle")
            .material_table_identity;
    let material_table_identity_after = prepared_material_contribution_for_preview(&preview)
        .scene_bundle
        .expect("after source material contribution should include scene bundle")
        .material_table_identity;
    assert_ne!(
        material_table_identity_before, material_table_identity_after,
        "MaterialGraphSourceFileV2 texture edit must change material table identity"
    );
    let product_identity_before = before_product.cache_key.as_str().to_string();
    let product_identity_after = preview.product.cache_key.as_str().to_string();

    SourceBackedMaterialGpuProof {
        preview,
        source_path,
        product_identity_before,
        product_identity_after,
        preview_shader_identity_before: before_compiled.identity,
        preview_shader_identity_after: after_compiled.identity,
        scene_shader_identity_before: before_compiled.scene_identity,
        scene_shader_identity_after: after_compiled.scene_identity,
        material_table_identity_before,
        material_table_identity_after,
    }
}

fn texture_material_source_document(
    document_id: MaterialGraphDocumentId,
    label: impl Into<String>,
    texture_descriptor_key: &str,
    stable_texture_id: &str,
) -> MaterialGraphDocument {
    let color = PortTypeId::new(1);
    let resource_kind = if texture_descriptor_key == "texture.sample_3d" {
        "asset.catalog.texture3d"
    } else {
        "asset.catalog.texture2d"
    };
    let texture_ref = ResourceRef::new(resource_kind, stable_texture_id)
        .expect("source-backed material proof texture ref should be valid");
    let texture_node = NodeDefinition::new(
        NodeId::new(7),
        texture_descriptor_key,
        [PortDefinition::new(
            PortId::new(70),
            "color",
            PortDirection::Output,
            color,
        )],
    )
    .with_values([GraphMetadataEntry::new(
        material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
        GraphValue::resource(texture_ref),
    )]);
    MaterialGraphDocument::new(
        document_id,
        label,
        GraphDefinition::new(
            GraphId::new(document_id.raw()),
            format!("wr028_source_backed_texture_{}", document_id.raw()),
            CyclePolicy::RejectDirectedCycles,
            [
                texture_node,
                NodeDefinition::new(
                    NodeId::new(8),
                    "pbr.output",
                    [PortDefinition::new(
                        PortId::new(80),
                        "base_color",
                        PortDirection::Input,
                        color,
                    )],
                ),
            ],
            [graph::EdgeDefinition::new(
                graph::EdgeId::new(7),
                PortId::new(70),
                PortId::new(80),
            )],
        ),
        MaterialOutputTarget::RenderMaterial,
    )
}

fn lower_texture_material_source_product(
    document: &MaterialGraphDocument,
    product_id: u64,
) -> FormedMaterialProduct {
    let lowering = lower_material_graph(document, &MaterialNodeCatalog::first_slice());
    assert!(
        !lowering.report.has_blocking_issues(),
        "{:?}",
        lowering.report.issues()
    );
    lowering
        .product
        .expect("source-backed texture material should form")
        .with_product_id(MaterialProductId::new(product_id))
}

fn source_backed_texture_catalog(
    product_id: u64,
    stable_texture_id: &str,
    texture_path: &str,
    texture_dimension: TextureDimension,
    asset_kind: asset::AssetKind,
) -> AssetCatalog {
    let depth = match texture_dimension {
        TextureDimension::Texture2D => 1,
        TextureDimension::Texture3DVolume => 2,
    };
    let texture_length = std::fs::metadata(texture_path)
        .expect("source-backed material proof texture metadata should exist")
        .len();
    let descriptor = TextureDescriptor::new(
        TextureProductId::new(product_id + 5),
        format!("WR-028 source-backed material texture {product_id}"),
        texture_dimension,
        TextureExtent::new(4, 4, depth),
    );
    let mip_count = descriptor.mip_count;
    let descriptor_hash = descriptor.descriptor_hash().to_string();
    let descriptor = descriptor.with_ktx2_metadata(
        Ktx2TextureMetadata::new(
            TexturePixelFormat::Rgba8Unorm,
            mip_count,
            descriptor_hash.clone(),
            "wr028-source-backed-material-edit",
        )
        .with_byte_layout(texture_length, [4 * 4 * depth as u64 * 4]),
    );
    let asset_id = asset_id(product_id + 401);
    let artifact_id = asset_artifact_id(product_id + 402);
    let mut catalog = AssetCatalog::new();
    catalog.insert_asset_record(AssetRecord::new(
        asset_id,
        stable_texture_id,
        format!("WR-028 source-backed texture {product_id}"),
        asset_kind,
    ));
    catalog.insert_artifact(
        AssetArtifactDescriptor::new(
            artifact_id,
            asset_id,
            asset_kind,
            ArtifactPayloadKind::TextureProduct {
                descriptor,
                descriptor_hash,
                artifact_uri: Some(texture_path.to_string()),
            },
            ArtifactCacheKey::new(format!("wr028-source-backed-texture-cache-{product_id}")),
        )
        .with_artifact_path(texture_path.to_string()),
    );
    catalog
}

fn material_graph_source_edit_workflow_app(
    catalog: AssetCatalog,
    material_asset_id: asset::AssetId,
    document: MaterialGraphDocument,
) -> RunenwerkEditorApp {
    let project = ProjectFileV3::new("wr028-source-backed-material-proof", "WR-028 GPU Proof");
    let session = EditorAssetProjectSession::from_project_file(&repo_root(), &project)
        .expect("source-backed material proof project session should form");
    let mut app = RunenwerkEditorApp::new();
    app.set_asset_project_session(session);
    app.asset_catalog_runtime_mut().replace_catalog(catalog);
    app.material_lab_runtime_mut()
        .select_material_asset(Some(material_asset_id));
    app.material_lab_runtime_mut()
        .set_active_source_document(material_asset_id, document);
    app
}

fn write_gpu_truth_material_source(file_name: &str, source: &MaterialGraphSourceFileV2) -> String {
    let directory = wr028_closeout_artifact_path("artifacts/fixtures/material-source-edits");
    std::fs::create_dir_all(&directory)
        .expect("source-backed material proof directory should be writable");
    let path = directory.join(file_name);
    let payload = ron::ser::to_string_pretty(source, ron::ser::PrettyConfig::new())
        .expect("source-backed material source should serialize");
    std::fs::write(&path, payload).expect("source-backed material source should write");
    path.to_string_lossy().into_owned()
}

#[derive(Debug, Clone)]
struct Wr028TextureViewerProofFixture {
    surface: editor_shell::TextureViewerSurfaceKind,
    tool_surface_kind: editor_shell::ToolSurfaceKind,
    label: &'static str,
    artifact_id: u64,
    texture_product_id: u64,
    artifact_uri: String,
    expected_rgba: [u8; 4],
    proof: TexturePreviewProofMetadata,
}

fn assert_wr028_texture_viewer_product_surface_gpu_truth(mut app: App) -> App {
    let texture2d = install_wr028_texture_viewer_catalog_fixture(
        &mut app,
        editor_shell::TextureViewerSurfaceKind::Texture2D,
    );
    assert_wr028_texture_viewer_provider_product_surface(&app, &texture2d);
    let (next_app, texture2d_capture) =
        capture_wr028_texture_viewer_dynamic_product_surface(app, &texture2d);
    app = next_app;
    write_wr028_texture_viewer_capture_metadata(&texture2d, &texture2d_capture);

    let texture3d = install_wr028_texture_viewer_catalog_fixture(
        &mut app,
        editor_shell::TextureViewerSurfaceKind::VolumeTexture3D,
    );
    assert_wr028_texture_viewer_provider_product_surface(&app, &texture3d);
    let (next_app, texture3d_capture) =
        capture_wr028_texture_viewer_dynamic_product_surface(app, &texture3d);
    app = next_app;
    write_wr028_texture_viewer_capture_metadata(&texture3d, &texture3d_capture);

    eprintln!(
        "WR-028 texture viewer GPU proof: Texture2D product={} target={} pixel={:?}; Texture3D product={} target={} pixel={:?}",
        texture2d.texture_product_id,
        texture2d.proof.target_key.label(),
        texture2d_capture.sampled_rgba,
        texture3d.texture_product_id,
        texture3d.proof.target_key.label(),
        texture3d_capture.sampled_rgba
    );
    app
}

#[derive(Debug, Clone)]
struct Wr028TextureViewerCapture {
    sampled_rgba: [u8; 4],
    capture_hash: String,
    capture_image_path: String,
}

fn install_wr028_texture_viewer_catalog_fixture(
    app: &mut App,
    surface: editor_shell::TextureViewerSurfaceKind,
) -> Wr028TextureViewerProofFixture {
    let (
        tool_surface_kind,
        label,
        asset_id_raw,
        artifact_id_raw,
        product_id,
        file_name,
        dimension,
        asset_kind,
        expected_rgba,
        generated,
    ) = match surface {
        editor_shell::TextureViewerSurfaceKind::Texture2D => (
            editor_shell::ToolSurfaceKind::TextureViewer,
            "texture2d",
            19_028,
            29_028,
            9_028,
            "wr028-texture-viewer-2d.ktx2",
            TextureDimension::Texture2D,
            AssetKind::Texture2D,
            WR028_TEXTURE_2D_PIXEL,
            false,
        ),
        editor_shell::TextureViewerSurfaceKind::VolumeTexture3D => (
            editor_shell::ToolSurfaceKind::VolumeTextureViewer,
            "texture3d",
            19_029,
            29_029,
            9_029,
            "wr028-volume-texture-viewer-3d.ktx2",
            TextureDimension::Texture3DVolume,
            AssetKind::Texture3DVolume,
            WR028_TEXTURE_3D_PIXEL,
            true,
        ),
    };
    let depth = if dimension == TextureDimension::Texture3DVolume {
        2
    } else {
        1
    };
    let fixture_dir = wr028_closeout_artifact_path("artifacts/fixtures");
    std::fs::create_dir_all(&fixture_dir).expect("WR-028 fixture directory should be writable");
    let fixture_path = fixture_dir.join(file_name);
    let bytes = build_rgba8_ktx2(4, 4, depth, expected_rgba);
    std::fs::write(&fixture_path, &bytes).expect("WR-028 durable KTX2 fixture should write");
    let artifact_uri = format!("{WR028_CLOSEOUT_DIR}/artifacts/fixtures/{file_name}");
    let descriptor = TextureDescriptor::new(
        TextureProductId::new(product_id),
        format!("WR-028 {label} catalog texture viewer proof"),
        dimension,
        TextureExtent::new(4, 4, depth),
    )
    .with_ktx2_metadata(
        Ktx2TextureMetadata::new(
            TexturePixelFormat::Rgba8Unorm,
            1,
            "",
            "wr028-closeout-catalog-preview-proof",
        )
        .with_byte_layout(bytes.len() as u64, [4 * 4 * depth as u64 * 4]),
    );
    let descriptor_hash = descriptor.descriptor_hash().to_string();
    let payload_kind = if generated {
        ArtifactPayloadKind::GeneratedTextureProduct {
            descriptor,
            descriptor_hash,
            artifact_uri: Some(artifact_uri.clone()),
        }
    } else {
        ArtifactPayloadKind::TextureProduct {
            descriptor,
            descriptor_hash,
            artifact_uri: Some(artifact_uri.clone()),
        }
    };
    let asset_id = asset_id(asset_id_raw);
    let artifact_id = asset_artifact_id(artifact_id_raw);
    let proof = {
        let host = app
            .world_mut()
            .resource_mut::<EditorHostResource>()
            .expect("editor host should exist");
        host.app
            .asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                format!("wr028_{label}_catalog_texture"),
                format!("WR-028 {label} catalog texture"),
                asset_kind,
            ));
        host.app
            .asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    artifact_id,
                    asset_id,
                    asset_kind,
                    payload_kind,
                    ArtifactCacheKey::new(format!("wr028-{label}-catalog-texture")),
                )
                .with_artifact_path(fixture_path.to_string_lossy().to_string()),
            );
        host.app
            .asset_catalog_runtime_mut()
            .select_asset(Some(asset_id));
        if surface == editor_shell::TextureViewerSurfaceKind::VolumeTexture3D {
            host.app.apply_texture_surface_action(
                editor_shell::TextureSurfaceAction::SetPreviewSlice {
                    surface,
                    slice_index: 0,
                },
            );
            host.app.apply_texture_surface_action(
                editor_shell::TextureSurfaceAction::SetPreviewChannel {
                    surface,
                    channel: editor_shell::TexturePreviewChannelSelection::All,
                },
            );
        }
        prepare_texture_preview(
            host.app.asset_catalog_runtime().catalog(),
            host.app.asset_catalog_runtime().selected_asset_id(),
            host.app.texture_preview_runtime(),
            surface,
        )
        .expect("WR-028 catalog-backed texture viewer preview should prepare")
        .proof
    };
    assert_eq!(proof.texture_product_id, product_id);
    assert_eq!(proof.artifact_uri, artifact_uri);
    assert_eq!(proof.residency_class, "engine.material_ktx2_upload");

    Wr028TextureViewerProofFixture {
        surface,
        tool_surface_kind,
        label,
        artifact_id: artifact_id_raw,
        texture_product_id: product_id,
        artifact_uri,
        expected_rgba,
        proof,
    }
}

fn assert_wr028_texture_viewer_provider_product_surface(
    app: &App,
    fixture: &Wr028TextureViewerProofFixture,
) {
    let host = app
        .world()
        .resource::<EditorHostResource>()
        .expect("editor host should exist");
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let request = wr028_texture_surface_request(fixture.tool_surface_kind);
    let frame = registry.resolve_frame(
        &SurfaceProviderBuildContext {
            app: &host.app,
            shell_state: &shell_state,
            theme: &theme,
            frame_metrics: None,
            viewport_observations: None,
            tool_surface_bindings: None,
            viewport_instances: None,
        },
        &request,
        &SurfaceSessionState::default(),
    );

    assert_eq!(
        frame.availability,
        editor_shell::SurfaceProviderAvailability::Available
    );
    assert!(
        product_surface_targets(&frame.artifact.root).iter().any(
            |(namespace, target_id)| namespace == &fixture.proof.target_key.namespace
                && target_id == &fixture.proof.target_key.target_id
        ),
        "WR-028 texture viewer provider must emit ProductSurfaceNode for target {}; root={:?}",
        fixture.proof.target_key.label(),
        frame.artifact.root
    );
    let frame_text = format!("{:?}", frame.artifact.root);
    assert!(frame_text.contains(&format!(
        "texture product id: {}",
        fixture.texture_product_id
    )));
    assert!(frame_text.contains(&fixture.proof.bind_group_identity));
}

fn product_surface_targets(node: &editor_shell::UiNode) -> Vec<(String, String)> {
    let mut targets = Vec::new();
    if let editor_shell::UiNodeKind::ProductSurface(surface) = &node.kind {
        let ProductSurfaceTextureBindingSource::DynamicTexture {
            namespace,
            target_id,
        } = &surface.source;
        targets.push((namespace.clone(), target_id.clone()));
    }
    for child in &node.children {
        targets.extend(product_surface_targets(child));
    }
    targets
}

fn wr028_texture_surface_request(
    tool_surface_kind: editor_shell::ToolSurfaceKind,
) -> editor_shell::SurfaceProviderRequest {
    let document_kind = match tool_surface_kind {
        editor_shell::ToolSurfaceKind::VolumeTextureViewer => DocumentKind::VolumeTexture,
        _ => DocumentKind::ProceduralTexture,
    };
    editor_shell::SurfaceProviderRequest {
        workspace_profile_id: editor_shell::TEXTURE_WORKSPACE_PROFILE_ID,
        document_context: editor_shell::SurfaceDocumentContext::Resolved {
            document_id: DocumentId(28),
            document_kind,
        },
        panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(28).unwrap(),
        tab_stack_id: editor_shell::TabStackId::try_from_raw(28).unwrap(),
        tool_surface_instance_id: editor_shell::ToolSurfaceInstanceId::try_from_raw(28).unwrap(),
        stable_surface_key: editor_shell::stable_key_for_tool_surface_kind(tool_surface_kind)
            .expect("WR-028 texture surface fixture should have a stable key"),
        provider_family_id: None,
        surface_route: None,
        surface_definition_id: editor_shell::tool_surface_definition_id(tool_surface_kind),
        capabilities: editor_shell::tool_surface_capability_set(tool_surface_kind),
    }
}

fn capture_wr028_texture_viewer_dynamic_product_surface(
    app: App,
    fixture: &Wr028TextureViewerProofFixture,
) -> (App, Wr028TextureViewerCapture) {
    let mut app = app
        .run_for_frames(4)
        .expect("WR-028 texture viewer dynamic product surface warmup should render frames");
    let ui_record = ui_composite_record(&app).clone();
    let target_label = fixture.proof.target_key.label();
    app.update_render_debug_config(|debug_config| {
        debug_config.clear();
        debug_config.capture_selectors = vec![RenderCaptureSelector {
            flow_id: Some(ui_record.flow_id.clone()),
            pass_id: Some(ui_record.pass_id.clone()),
            stage: CaptureStage::Before,
            resource_id: target_label.clone(),
            texture_class: CaptureTextureClass::ColorTarget,
        }];
    });
    app.update_render_debug_control(|debug_control| {
        debug_control.provenance_enabled = true;
        debug_control.capture_enabled = true;
        debug_control.readback_enabled = true;
        debug_control.artifact_export_enabled = true;
        debug_control.artifact_output_dir = wr028_closeout_artifact_path("artifacts/captures");
    });

    let app = app
        .run_for_frames(3)
        .expect("WR-028 texture viewer dynamic product surface capture should render frames");
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
    let capture = captures
        .find(
            ui_record.flow_id.as_str(),
            ui_record.pass_id.as_str(),
            CaptureStage::Before,
            target_label.as_str(),
        )
        .unwrap_or_else(|| {
            panic!(
                "WR-028 texture viewer product-surface capture should exist for {}; captures={:?}; report={:?}",
                target_label, captures.captures, debug_report
            )
        });
    assert_eq!(
        capture.terminal.code,
        RenderCaptureTerminalCode::Completed,
        "WR-028 texture viewer product-surface capture must complete: {:?}",
        capture.terminal
    );
    let sampled_rgba = capture
        .sample_center_rgba8()
        .expect("WR-028 texture viewer capture should include sampleable pixels");
    assert_eq!(
        sampled_rgba, fixture.expected_rgba,
        "WR-028 {} product-surface pixels must come from the catalog-backed texture viewer target",
        fixture.label
    );
    let image_path = wr028_closeout_artifact_path("artifacts/captures")
        .join(deterministic_capture_filename(&capture.identity, "png"));
    assert!(
        image_path.exists(),
        "WR-028 texture viewer capture image should be exported at {:?}",
        image_path
    );
    let capture_hash = capture
        .bytes_rgba8
        .as_ref()
        .map(|bytes| format!("blake3:{}", blake3::hash(bytes).to_hex()))
        .expect("WR-028 texture viewer capture should include rgba bytes");

    (
        app,
        Wr028TextureViewerCapture {
            sampled_rgba,
            capture_hash,
            capture_image_path: repo_relative_path(&image_path),
        },
    )
}

fn ui_composite_record(app: &App) -> &RenderPassProvenanceRecord {
    app.world()
        .resource::<RenderPassProvenanceState>()
        .expect("pass provenance state should exist")
        .records
        .iter()
        .find(|record| record.shader_id == "builtin:ui_composite")
        .expect("ui composite pass provenance should exist")
}

fn write_wr028_texture_viewer_capture_metadata(
    fixture: &Wr028TextureViewerProofFixture,
    capture: &Wr028TextureViewerCapture,
) {
    let metadata_dir = wr028_closeout_artifact_path("artifacts/metadata");
    std::fs::create_dir_all(&metadata_dir).expect("WR-028 metadata directory should be writable");
    let metadata_path = metadata_dir.join(format!("wr028-{}-proof.ron", fixture.label));
    let payload = format!(
        "(
  proof: \"wr028_texture_viewer_product_surface\",
  surface: \"{:?}\",
  provider_path: \"TextureViewerProvider/VolumeTextureViewerProvider -> catalog TextureProduct/GeneratedTextureProduct -> prepare_texture_preview -> ProductSurfaceNode -> dynamic texture upload -> captured pixels\",
  artifact_id: {},
  texture_product_id: {},
  artifact_uri: \"{}\",
  descriptor_hash: \"{}\",
  upload_format: \"{}\",
  mip_count: {},
  selected_mip: {},
  selected_slice: {},
  selected_channel: \"{}\",
  sampler_identity: \"{}\",
  bind_group_identity: \"{}\",
  residency_state: \"{}\",
  residency_class: \"{}\",
  preview_target_key: \"{}\",
  product_surface_key: \"{}\",
  capture_image: \"{}\",
  capture_hash: \"{}\",
  sampled_rgba: {:?},
)
",
        fixture.surface,
        fixture.artifact_id,
        fixture.texture_product_id,
        fixture.artifact_uri,
        fixture.proof.descriptor_hash,
        fixture.proof.upload_format,
        fixture.proof.mip_count,
        fixture.proof.selected_mip,
        fixture.proof.selected_slice,
        fixture.proof.selected_channel,
        fixture.proof.sampler_identity,
        fixture.proof.bind_group_identity,
        fixture.proof.residency_state,
        fixture.proof.residency_class,
        fixture.proof.target_key.label(),
        fixture.proof.target_key.label(),
        capture.capture_image_path,
        capture.capture_hash,
        capture.sampled_rgba
    );
    std::fs::write(&metadata_path, payload)
        .expect("WR-028 texture viewer proof metadata should write");
}

fn wr028_closeout_artifact_path(relative: &str) -> PathBuf {
    repo_root().join(WR028_CLOSEOUT_DIR).join(relative)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("runenwerk_editor manifest should be under apps/runenwerk_editor")
        .to_path_buf()
}

fn repo_relative_path(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn assert_source_backed_material_preview_product_pixels(
    app: App,
    proof: SourceBackedMaterialGpuProof,
    label: &str,
) -> App {
    let source_path = proof.source_path.clone();
    let product_identity_before = proof.product_identity_before.clone();
    let product_identity_after = proof.product_identity_after.clone();
    let preview_shader_identity_before = proof.preview_shader_identity_before.clone();
    let preview_shader_identity_after = proof.preview_shader_identity_after.clone();
    let scene_shader_identity_before = proof.scene_shader_identity_before.clone();
    let scene_shader_identity_after = proof.scene_shader_identity_after.clone();
    let material_table_identity_before = proof.material_table_identity_before.clone();
    let material_table_identity_after = proof.material_table_identity_after.clone();
    let app = assert_material_preview_product_pixels(app, proof.preview, label);
    eprintln!(
        "WR-028 source-backed material identity proof {label}: source={source_path}; product_identity_hash={}->{}; preview_shader_identity_hash={}->{}; scene_shader_identity_hash={}->{}; material_table_identity_hash={}->{}",
        short_identity_hash(&product_identity_before),
        short_identity_hash(&product_identity_after),
        short_identity_hash(&preview_shader_identity_before),
        short_identity_hash(&preview_shader_identity_after),
        short_identity_hash(&scene_shader_identity_before),
        short_identity_hash(&scene_shader_identity_after),
        short_identity_hash(&material_table_identity_before),
        short_identity_hash(&material_table_identity_after),
    );
    app
}

fn short_identity_hash(identity: &str) -> String {
    blake3::hash(identity.as_bytes()).to_hex().as_str()[..16].to_string()
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
        .find(|record| {
            record.shader_id == scene_shader_path || scene_product_record_targets_scene_color(record)
        })
        .unwrap_or_else(|| {
            panic!(
                "scene pass should consume either the active preview scene shader or the generated scene table shader; records={:?}",
                provenance
                    .records
                    .iter()
                    .map(|record| record.shader_id.as_str())
                    .collect::<Vec<_>>()
            )
        })
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
        debug_control.artifact_export_enabled = true;
        debug_control.artifact_output_dir = wr028_closeout_artifact_path("artifacts/captures");
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
    let scene_sample = dominant_pixel_in_region(scene_capture, 0, scene_capture.width, |rgba| {
        let max = rgba[0].max(rgba[1]).max(rgba[2]) as i32;
        let min = rgba[0].min(rgba[1]).min(rgba[2]) as i32;
        max - min
    })
    .expect("generated scene capture should include material-colored pixels");
    let scene_pixel = scene_sample.1;
    let preview_pixel = material_preview_capture
        .sample_center_rgba8()
        .expect("generated material preview capture should include pixels");
    let scene_image_path = wr028_closeout_artifact_path("artifacts/captures").join(
        deterministic_capture_filename(&scene_capture.identity, "png"),
    );
    let material_preview_image_path = wr028_closeout_artifact_path("artifacts/captures").join(
        deterministic_capture_filename(&material_preview_capture.identity, "png"),
    );
    assert!(
        scene_image_path.exists() && material_preview_image_path.exists(),
        "source-backed material proof captures should be exported: scene={:?}, preview={:?}",
        scene_image_path,
        material_preview_image_path
    );
    let scene_capture_hash = scene_capture
        .bytes_rgba8
        .as_ref()
        .map(|bytes| format!("blake3:{}", blake3::hash(bytes).to_hex()))
        .expect("generated scene capture should include rgba bytes");
    let preview_capture_hash = material_preview_capture
        .bytes_rgba8
        .as_ref()
        .map(|bytes| format!("blake3:{}", blake3::hash(bytes).to_hex()))
        .expect("generated material preview capture should include rgba bytes");
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
    assert!(
        scene_sample.2 > 24,
        "generated {label} scene shader should produce a visible material-colored scene region: {:?}",
        scene_sample
    );
    let scene_capture_path = repo_relative_path(&scene_image_path);
    let preview_capture_path = repo_relative_path(&material_preview_image_path);
    let scene_sample_rgba = scene_sample.1;
    let scene_sample_coord = scene_sample.0;
    eprintln!(
        "WR-028 source-backed material GPU proof {label}: scene_shader={scene_shader_path}, preview_shader={preview_shader_path}, textures={texture_artifact_paths:?}, scene_capture={scene_capture_path} scene_hash={scene_capture_hash} scene_pixel={scene_sample_rgba:?}@{scene_sample_coord:?}, preview_capture={preview_capture_path} preview_hash={preview_capture_hash} preview_pixel={preview_pixel:?}"
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
    let directory = wr028_closeout_artifact_path("artifacts/generated/material-source-edits");
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
    let directory = wr028_closeout_artifact_path("artifacts/fixtures/material-source-edits");
    std::fs::create_dir_all(&directory).expect("gpu proof texture directory should be writable");
    let path = directory.join(file_name);
    let bytes = build_rgba8_ktx2(width, height, depth, texel);
    std::fs::write(&path, bytes).expect("gpu proof KTX2 texture should be writable");
    path.to_string_lossy().into_owned()
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
