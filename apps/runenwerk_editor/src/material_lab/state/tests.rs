use super::*;
use crate::material_lab::{
    PreviewSceneMaterialSlot, PreviewSceneProduct, PreviewSceneProductMode,
    PreviewSceneProductRequestIdentity, PreviewSceneResourceSlot, PreviewSceneResourceSlotMapping,
    PreviewSceneShaderProductRef,
};
use asset::{
    ArtifactCacheKey, ArtifactPayloadKind, AssetArtifactDescriptor, AssetDiagnosticCode,
    AssetDiagnosticRecord, AssetDiagnosticSeverity, AssetRecord, AssetSourceDescriptor,
    ForeignMeshMaterialRegionDescriptor, SourceHash, asset_artifact_id, asset_id, asset_source_id,
};
use editor_scene::{
    SceneMaterialAssignmentState, SceneMaterialPalette, SceneMaterialSlot, SceneMaterialSlotId,
    SceneMeshMaterialRegionId, SceneModelMeshMaterialRegionSourceId, SceneModelMeshSourceId,
};
use graph::{
    CyclePolicy, EdgeDefinition, GraphDefinition, GraphId, NodeDefinition, NodeId, PortDefinition,
    PortDirection, PortId,
};
use material_graph::{
    MaterialGraphDocument, MaterialGraphEditorState, MaterialGraphNodeLayout,
    MaterialGraphViewportState, MaterialOutputTarget, MaterialValueType,
};
use resource_ref::ResourceRef;
use texture::{
    Ktx2TextureMetadata, TextureDescriptor, TextureDimension, TextureExtent, TexturePixelFormat,
    TextureProductId,
};

#[test]
fn graph_canvas_projects_source_document_without_formed_preview() {
    let asset_id = asset_id(7);
    let color_port = MaterialValueType::Color.port_type_id();
    let editor_state = MaterialGraphEditorState {
        viewport: MaterialGraphViewportState {
            pan_x: 12,
            pan_y: -6,
            zoom_milli: 1500,
        },
        node_layouts: vec![MaterialGraphNodeLayout::new(NodeId::new(3), 420, 90)],
        ..MaterialGraphEditorState::default()
    };
    let document = MaterialGraphDocument::new(
        material_graph::MaterialGraphDocumentId::new(70),
        "source-backed",
        GraphDefinition::new(
            GraphId::new(1),
            "source",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(
                    NodeId::new(3),
                    "pbr.base_color",
                    [PortDefinition::new(
                        PortId::new(30),
                        "color",
                        PortDirection::Output,
                        color_port,
                    )],
                ),
                NodeDefinition::new(
                    NodeId::new(4),
                    "pbr.output",
                    [PortDefinition::new(
                        PortId::new(40),
                        "base_color",
                        PortDirection::Input,
                        color_port,
                    )],
                ),
            ],
            [EdgeDefinition::new(
                graph::EdgeId::new(9),
                PortId::new(30),
                PortId::new(40),
            )],
        ),
        MaterialOutputTarget::RenderMaterial,
    )
    .with_editor_state(editor_state);
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_active_source_document(asset_id, document);
    let mut catalog = AssetCatalog::new();
    catalog.insert_asset_record(AssetRecord::new(
        asset_id,
        "mat.source",
        "Source Material",
        AssetKind::MaterialGraph,
    ));

    let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

    assert_eq!(
        view.graph.document_id,
        Some(material_graph::MaterialGraphDocumentId::new(70))
    );
    assert_eq!(view.graph.viewport.zoom_milli, 1500);
    assert_eq!(view.graph.nodes.len(), 2);
    let color_node = view
        .graph
        .nodes
        .iter()
        .find(|node| node.node_id == NodeId::new(3))
        .expect("source node should project");
    assert_eq!(color_node.position_x, 420);
    assert_eq!(color_node.output_ports[0].port_id, PortId::new(30));
    assert!(color_node.output_ports[0].connected);
    assert_eq!(view.graph.edges[0].from_port_id, PortId::new(30));
    assert_eq!(view.graph.edges[0].to_port_id, PortId::new(40));
}

#[test]
fn material_graph_palette_search_is_session_projection_state() {
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_node_palette_search_query("noise");
    let catalog = AssetCatalog::new();

    let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

    assert_eq!(view.palette.search_query, "noise");
    assert!(
        view.palette
            .categories
            .iter()
            .flat_map(|category| category.nodes.iter())
            .all(|node| node.label.to_ascii_lowercase().contains("noise")
                || node.descriptor_key.to_ascii_lowercase().contains("noise"))
    );
}

#[test]
fn material_graph_diagnostics_anchor_into_graph_canvas_overlays() {
    let asset_id = asset_id(8);
    let color_port = MaterialValueType::Color.port_type_id();
    let document = MaterialGraphDocument::new(
        material_graph::MaterialGraphDocumentId::new(80),
        "diagnostics",
        GraphDefinition::new(
            GraphId::new(1),
            "source",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(
                    NodeId::new(3),
                    "pbr.base_color",
                    [PortDefinition::new(
                        PortId::new(30),
                        "color",
                        PortDirection::Output,
                        color_port,
                    )],
                ),
                NodeDefinition::new(
                    NodeId::new(4),
                    "pbr.output",
                    [PortDefinition::new(
                        PortId::new(40),
                        "base_color",
                        PortDirection::Input,
                        color_port,
                    )],
                ),
            ],
            [EdgeDefinition::new(
                graph::EdgeId::new(9),
                PortId::new(30),
                PortId::new(40),
            )],
        ),
        MaterialOutputTarget::RenderMaterial,
    );
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_active_source_document(asset_id, document);
    runtime.record_diagnostics([
        AssetDiagnosticRecord::new(
            AssetDiagnosticCode::RatificationRejected,
            AssetDiagnosticSeverity::Warning,
            "node warning",
        )
        .with_subject("material_graph.node:3"),
        AssetDiagnosticRecord::new(
            AssetDiagnosticCode::RatificationRejected,
            AssetDiagnosticSeverity::Error,
            "port error",
        )
        .with_subject("material_graph.port:40"),
    ]);
    runtime.set_active_diagnostic_index(Some(1));
    let catalog = AssetCatalog::new();

    let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

    assert_eq!(view.active_diagnostic_index, Some(1));
    assert_eq!(view.validation_overlays.len(), 2);
    assert_eq!(
        view.validation_overlays[0].subject_node_id,
        Some(NodeId::new(3))
    );
    assert_eq!(
        view.validation_overlays[1].subject_port_id,
        Some(PortId::new(40))
    );
    assert!(view.validation_overlays[1].active);
    assert!(
        view.graph
            .graph_editor
            .canvas
            .overlays
            .iter()
            .any(|overlay| overlay.anchor
                == ui_graph_editor::GraphOverlayAnchor::Node(ui_graph_editor::GraphNodeKey(3),)),
        "node diagnostic must be projected into graph canvas overlays",
    );
    assert!(
        view.graph
            .graph_editor
            .canvas
            .overlays
            .iter()
            .any(|overlay| overlay.anchor
                == ui_graph_editor::GraphOverlayAnchor::Port(ui_graph_editor::GraphPortKey(40),)
                && overlay.active),
        "active port diagnostic must stay anchored and highlighted in canvas overlays",
    );
}

#[test]
fn material_graph_node_picker_projects_filtered_catalog_selection() {
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_node_picker_search_query("base");
    runtime.open_node_picker();
    let catalog = AssetCatalog::new();

    let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

    assert!(view.node_picker.open);
    assert_eq!(view.node_picker.search_query, "base");
    assert_eq!(
        view.node_picker.highlighted_descriptor_key.as_deref(),
        Some("pbr.base_color")
    );
    assert!(
        view.node_picker
            .categories
            .iter()
            .flat_map(|category| category.nodes.iter())
            .all(|node| node.label.to_ascii_lowercase().contains("base")
                || node.descriptor_key.to_ascii_lowercase().contains("base"))
    );
}

#[test]
fn material_graph_texture_picker_lists_catalog_texture_products() {
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_texture_resource_search_query("albedo");
    let mut catalog = AssetCatalog::new();
    let asset_id = asset_id(90);
    let artifact_id = asset_artifact_id(91);
    let descriptor = TextureDescriptor::new(
        TextureProductId::new(92),
        "Rock Albedo",
        TextureDimension::Texture2D,
        TextureExtent::new(4, 4, 1),
    );
    catalog.insert_asset_record(AssetRecord::new(
        asset_id,
        "rock.albedo",
        "Rock Albedo",
        AssetKind::Texture2D,
    ));
    catalog.insert_artifact(AssetArtifactDescriptor::new(
        artifact_id,
        asset_id,
        AssetKind::Texture2D,
        ArtifactPayloadKind::TextureProduct {
            descriptor_hash: descriptor.descriptor_hash().to_string(),
            descriptor,
            artifact_uri: Some(".runenwerk/artifacts/rock-albedo.ktx2".to_string()),
        },
        ArtifactCacheKey::new("rock-albedo"),
    ));

    let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

    assert_eq!(view.texture_picker.search_query, "albedo");
    assert_eq!(view.texture_picker.options.len(), 1);
    let option = &view.texture_picker.options[0];
    assert_eq!(option.stable_id, "rock.albedo");
    assert_eq!(
        option.resource_kind,
        material_graph::MaterialResourceKind::Texture2D
    );
    assert_eq!(option.product_id, 92);
    assert!(option.valid);
    assert_eq!(option.artifact_uri, ".runenwerk/artifacts/rock-albedo.ktx2");
    assert!(!option.descriptor_hash.is_empty());
}

#[test]
fn unresolved_texture_binding_reports_binding_diagnostic() {
    let asset_id = asset_id(91);
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_active_source_document(asset_id, texture_source_document(None));

    let view = runtime.graph_canvas_view_model(&AssetCatalog::new(), Vec::new());

    assert_eq!(view.resource_binding_diagnostics.len(), 1);
    let row = &view.resource_binding_diagnostics[0];
    assert_eq!(row.status, MaterialResourceBindingStatusKind::Unresolved);
    assert_eq!(row.code, "material.resource.unresolved_binding");
    assert_eq!(row.resource_key_or_slot_label, "texture_ref");
    assert_eq!(row.expected_kind_label.as_deref(), Some("texture_2d"));
}

#[test]
fn missing_texture_resource_reports_binding_diagnostic() {
    let asset_id = asset_id(92);
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_active_source_document(
        asset_id,
        texture_source_document(Some(
            ResourceRef::new("asset.catalog.texture2d", "missing.albedo").expect("ref"),
        )),
    );

    let view = runtime.graph_canvas_view_model(&AssetCatalog::new(), Vec::new());

    assert_eq!(
        view.resource_binding_diagnostics[0].status,
        MaterialResourceBindingStatusKind::Missing
    );
    assert!(
        view.resource_binding_diagnostics[0]
            .message
            .contains("missing.albedo")
    );
}

#[test]
fn ambiguous_texture_resource_reports_binding_diagnostic() {
    let material_asset_id = asset_id(93);
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_active_source_document(
        material_asset_id,
        texture_source_document(Some(
            ResourceRef::new("asset.catalog.texture2d", "rock.albedo").expect("ref"),
        )),
    );
    let mut catalog = AssetCatalog::new();
    catalog.insert_asset_record(AssetRecord::new(
        asset_id(301),
        "rock.albedo",
        "Rock Albedo A",
        AssetKind::Texture2D,
    ));
    catalog.insert_asset_record(AssetRecord::new(
        asset_id(302),
        "rock.albedo",
        "Rock Albedo B",
        AssetKind::Texture2D,
    ));

    let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

    assert_eq!(
        view.resource_binding_diagnostics[0].status,
        MaterialResourceBindingStatusKind::Ambiguous
    );
}

#[test]
fn incompatible_texture_resource_reports_binding_diagnostic() {
    let material_asset_id = asset_id(94);
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_active_source_document(
        material_asset_id,
        texture_source_document(Some(
            ResourceRef::new("asset.catalog.texture2d", "rock.volume").expect("ref"),
        )),
    );
    let mut catalog = AssetCatalog::new();
    catalog.insert_asset_record(AssetRecord::new(
        asset_id(303),
        "rock.volume",
        "Rock Volume",
        AssetKind::Texture3DVolume,
    ));

    let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

    assert_eq!(
        view.resource_binding_diagnostics[0].status,
        MaterialResourceBindingStatusKind::Incompatible
    );
}

#[test]
fn generated_texture_available_reports_status_when_observable() {
    let material_asset_id = asset_id(95);
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_active_source_document(
        material_asset_id,
        texture_source_document(Some(
            ResourceRef::new("asset.catalog.texture2d", "generated.albedo").expect("ref"),
        )),
    );
    let mut catalog = AssetCatalog::new();
    insert_texture_asset(
        &mut catalog,
        asset_id(304),
        asset_artifact_id(404),
        "generated.albedo",
        TexturePayloadFixture::Generated,
        ArtifactValidity::Valid,
    );

    let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

    assert_eq!(
        view.resource_binding_diagnostics[0].status,
        MaterialResourceBindingStatusKind::GeneratedAvailable
    );
}

#[test]
fn generated_texture_unavailable_reports_status_when_observable() {
    let material_asset_id = asset_id(96);
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_active_source_document(
        material_asset_id,
        texture_source_document(Some(
            ResourceRef::new("asset.catalog.texture2d", "generated.stale").expect("ref"),
        )),
    );
    let mut catalog = AssetCatalog::new();
    insert_texture_asset(
        &mut catalog,
        asset_id(305),
        asset_artifact_id(405),
        "generated.stale",
        TexturePayloadFixture::Generated,
        ArtifactValidity::Stale,
    );

    let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

    assert_eq!(
        view.resource_binding_diagnostics[0].status,
        MaterialResourceBindingStatusKind::GeneratedUnavailable
    );
}

#[test]
fn resource_binding_diagnostic_population_does_not_mutate_resolution_state() {
    let material_asset_id = asset_id(97);
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_active_source_document(
        material_asset_id,
        texture_source_document(Some(
            ResourceRef::new("asset.catalog.texture2d", "rock.albedo").expect("ref"),
        )),
    );
    let mut catalog = AssetCatalog::new();
    insert_texture_asset(
        &mut catalog,
        asset_id(306),
        asset_artifact_id(406),
        "rock.albedo",
        TexturePayloadFixture::Imported,
        ArtifactValidity::Valid,
    );
    let before = runtime.graph_canvas_view_model(&catalog, Vec::new());
    let after = runtime.graph_canvas_view_model(&catalog, Vec::new());

    assert_eq!(
        before.resource_binding_diagnostics,
        after.resource_binding_diagnostics
    );
    assert_eq!(
        runtime.selected_material_asset_id(),
        Some(material_asset_id)
    );
    assert_eq!(catalog.assets().count(), 1);
}

#[test]
fn material_diagnostic_rows_preserve_code_subject_and_severity() {
    let mut runtime = MaterialLabRuntime::default();
    runtime.record_diagnostic(
        AssetDiagnosticRecord::new(
            AssetDiagnosticCode::RatificationRejected,
            AssetDiagnosticSeverity::Warning,
            "roughness input is disconnected",
        )
        .with_subject("material_graph.node:3"),
    );

    let view = runtime.graph_canvas_view_model(&AssetCatalog::new(), Vec::new());

    assert_eq!(view.diagnostic_rows.len(), 1);
    let row = &view.diagnostic_rows[0];
    assert_eq!(row.severity, MaterialDiagnosticSeverity::Warning);
    assert_eq!(row.code, "asset.ratification.rejected");
    assert_eq!(row.subject_label.as_deref(), Some("material_graph.node:3"));
    assert_eq!(row.category_label.as_deref(), Some("material workflow"));
    assert_eq!(row.message, "roughness input is disconnected");
}

#[test]
fn material_inspector_view_model_exposes_structured_diagnostics() {
    let mut runtime = MaterialLabRuntime::default();
    runtime.record_diagnostic(AssetDiagnosticRecord::error(
        AssetDiagnosticCode::ImportProfileRejected,
        "material import profile is invalid",
    ));

    let view = runtime.inspector_view_model(&AssetCatalog::new());

    assert_eq!(view.diagnostic_rows.len(), 1);
    assert_eq!(
        view.diagnostic_rows[0].code,
        "asset.import.profile_rejected"
    );
    assert!(
        view.diagnostic_lines
            .iter()
            .any(|line| line.contains("material import profile is invalid")),
        "legacy string diagnostics remain available during ML-A",
    );
}

#[test]
fn material_preview_status_reports_no_selection() {
    let runtime = MaterialLabRuntime::default();

    let view = runtime.preview_view_model(&AssetCatalog::new());

    assert_eq!(
        view.preview_status.status,
        MaterialPreviewStatusKind::NoSelection
    );
    assert_eq!(view.preview_status.headline, "No material asset selected");
    assert!(!view.preview_status.last_good_available);
}

#[test]
fn material_preview_model_mesh_projection_uses_source_backed_regions() {
    let runtime = MaterialLabRuntime::default();
    let mut catalog = AssetCatalog::new();
    let model_asset_id = asset_id(401);
    let model_source_id = asset_source_id(402);
    insert_foreign_mesh_reference_artifact(
        &mut catalog,
        model_asset_id,
        model_source_id,
        asset_artifact_id(403),
        "hero.model",
        "Hero Model",
        "source_material_slot:0",
        "Body",
    );
    let assigned_slot = SceneMaterialSlotId::new(2);
    let material_region = SceneModelMeshMaterialRegionSourceId::new(
        SceneModelMeshSourceId::new(model_asset_id, model_source_id)
            .with_source_revision_id(asset::asset_source_revision_id(1))
            .with_source_revision("sha256:mesh"),
        SceneMeshMaterialRegionId::new("source_material_slot:0")
            .expect("source-backed material region key should form"),
    );
    let assignments = SceneMaterialAssignmentState::new_with_model_mesh_assignments(
        SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(assigned_slot, "Hero Body").with_material_asset(asset_id(404)),
        ])
        .expect("scene material palette should form"),
        [],
        [editor_scene::SceneModelMeshMaterialSlotAssignment::new(
            material_region,
            assigned_slot,
        )],
    )
    .expect("scene material assignments should form");

    let view =
        runtime.preview_view_model_with_scene_material_assignments(&catalog, Some(&assignments));
    let preview = view.model_mesh_preview;

    assert_eq!(preview.status, MaterialModelMeshPreviewStatusKind::Ready);
    assert_eq!(preview.source_backed_region_count, 1);
    assert_eq!(preview.assignable_region_count, 1);
    assert_eq!(preview.prepared_region_count, 1);
    assert_eq!(preview.assigned_region_count, 1);
    assert_eq!(preview.diagnostic_count, 0);
    assert_eq!(preview.regions.len(), 1);
    let region = &preview.regions[0];
    assert_eq!(region.asset_id, model_asset_id);
    assert_eq!(region.source_id, Some(model_source_id));
    assert_eq!(region.source_revision.as_deref(), Some("sha256:mesh"));
    assert_eq!(region.assigned_slot_label.as_deref(), Some("Hero Body"));
    assert_eq!(region.requested_slot_id, Some(assigned_slot));
    assert_eq!(region.resolved_slot_id, Some(assigned_slot));
    assert_eq!(region.material_table_index, Some(1));
    assert!(!region.used_default_fallback);
    assert_eq!(region.diagnostic, None);
}

#[test]
fn material_preview_status_reports_no_source_document() {
    let mut runtime = MaterialLabRuntime::default();
    runtime.select_material_asset(Some(asset_id(12)));

    let view = runtime.preview_view_model(&AssetCatalog::new());

    assert_eq!(
        view.preview_status.status,
        MaterialPreviewStatusKind::NoSourceDocument
    );
    assert_eq!(
        view.preview_status.headline,
        "No material source document is loaded"
    );
}

#[test]
fn material_preview_status_reports_published_when_existing_state_has_preview() {
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_active_preview(test_preview_product(asset_id(20)));

    let view = runtime.preview_view_model(&AssetCatalog::new());

    assert_eq!(
        view.preview_status.status,
        MaterialPreviewStatusKind::Published
    );
    assert!(view.preview_status.last_good_available);
    assert_eq!(
        view.preview_status.active_preview_label.as_deref(),
        Some("material product 30")
    );
    assert_eq!(
        view.preview_status.publication_status,
        MaterialPreviewPublicationStatusKind::NoPublication
    );
    assert_eq!(
        view.preview_status.product_status_label.as_deref(),
        Some("active material preview product ready")
    );
    assert_eq!(
        view.preview_status.active_product_label.as_deref(),
        Some("material product 30")
    );
    assert_eq!(
        view.preview_status.material_artifact_label.as_deref(),
        Some("material artifact 32")
    );
    assert_eq!(
        view.preview_status.shader_artifact_label.as_deref(),
        Some("shader artifact 33")
    );
    assert_eq!(
        view.preview_status.scene_shader_artifact_label.as_deref(),
        Some("scene shader artifact 34")
    );
    assert_eq!(
        view.preview_status.viewport_product_label.as_deref(),
        Some("viewport product 10030")
    );
}

#[test]
fn material_preview_status_reports_failed_preserved_last_good_when_existing_state_has_preserved_failure()
 {
    let mut runtime = MaterialLabRuntime::default();
    runtime.select_material_asset(Some(asset_id(21)));
    runtime.record_publication(EditorMaterialPreviewPublicationJournalEntry {
        artifact_id: asset_artifact_id(91),
        product_id: None,
        status: ProductPublicationStatus::FailedPreserved,
    });

    let view = runtime.preview_view_model(&AssetCatalog::new());

    assert_eq!(
        view.preview_status.status,
        MaterialPreviewStatusKind::FailedPreservedLastGood
    );
    assert!(view.preview_status.last_good_available);
    assert!(view.preview_status.failed_preserved_last_good);
    assert_eq!(
        view.preview_status.publication_status,
        MaterialPreviewPublicationStatusKind::FailedPreserved
    );
    assert_eq!(
        view.preview_status.product_status_label.as_deref(),
        Some("prior valid material artifact preserved")
    );
    assert_eq!(
        view.preview_status.last_publication_label.as_deref(),
        Some("FailedPreserved artifact 91 product none")
    );
    assert_eq!(
        view.preview_status.last_good_reason.as_deref(),
        Some("last publication preserved a prior valid material artifact")
    );
    assert_eq!(
        view.preview_status.material_artifact_label.as_deref(),
        Some("last publication artifact 91")
    );
    assert!(
        view.preview_status
            .detail_lines
            .iter()
            .any(|line| line.contains("FailedPreserved artifact 91")),
    );
}

#[test]
fn preview_failure_without_prior_valid_reports_no_last_good() {
    let mut runtime = MaterialLabRuntime::default();
    runtime.select_material_asset(Some(asset_id(22)));
    runtime.set_workflow_status("preview build blocked");

    let view = runtime.preview_view_model(&AssetCatalog::new());

    assert_eq!(
        view.preview_status.status,
        MaterialPreviewStatusKind::Blocked
    );
    assert!(!view.preview_status.last_good_available);
    assert!(!view.preview_status.failed_preserved_last_good);
    assert_eq!(
        view.preview_status.publication_status,
        MaterialPreviewPublicationStatusKind::NoPublication
    );
    assert_eq!(view.preview_status.last_good_reason, None);
    assert_eq!(
        view.preview_status.product_status_label.as_deref(),
        Some("preview status: Blocked")
    );
}

#[test]
fn material_preview_view_model_reports_product_or_artifact_labels_when_available() {
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_active_preview(test_preview_product(asset_id(23)));

    let view = runtime.preview_view_model(&AssetCatalog::new());

    assert_eq!(
        view.preview_status.active_product_label.as_deref(),
        Some("material product 30")
    );
    assert_eq!(
        view.preview_status.material_artifact_label.as_deref(),
        Some("material artifact 32")
    );
    assert_eq!(
        view.preview_status.shader_artifact_label.as_deref(),
        Some("shader artifact 33")
    );
}

#[test]
fn preview_status_population_does_not_mutate_material_lab_state() {
    let mut runtime = MaterialLabRuntime::default();
    runtime.set_active_preview(test_preview_product(asset_id(24)));
    runtime.record_diagnostic(AssetDiagnosticRecord::warning(
        AssetDiagnosticCode::RatificationRejected,
        "existing diagnostic",
    ));

    let before = runtime.preview_view_model(&AssetCatalog::new());
    let after = runtime.preview_view_model(&AssetCatalog::new());

    assert_eq!(before, after);
    assert_eq!(runtime.diagnostics().len(), 1);
    assert_eq!(
        runtime.selected_material_asset_id(),
        Some(asset_id(24)),
        "preview status projection must not change selected asset"
    );
}

#[test]
fn preview_status_includes_current_preview_scene_product_identity() {
    let mut runtime = MaterialLabRuntime::default();
    let product = preview_scene_product("active", "table-a", "layout-a");
    runtime
        .record_preview_scene_product(&product.request_identity(), product.clone())
        .expect("fresh preview scene product should record");

    let view = runtime.preview_view_model(&AssetCatalog::new());

    assert_eq!(
        view.preview_status
            .preview_scene_product_identity
            .as_deref(),
        Some(product.product_identity.as_str())
    );
    assert!(
        view.preview_status
            .preview_scene_product_status_label
            .as_deref()
            .is_some_and(|label| label.contains("current preview scene product available"))
    );
}

#[test]
fn preview_status_includes_scene_material_table_mode() {
    let mut runtime = MaterialLabRuntime::default();
    let product = preview_scene_product("active", "table-a", "layout-a");
    runtime
        .record_preview_scene_product(&product.request_identity(), product)
        .expect("fresh preview scene product should record");

    let view = runtime.preview_view_model(&AssetCatalog::new());

    assert_eq!(
        view.preview_status
            .preview_scene_product_mode_label
            .as_deref(),
        Some("scene material table")
    );
}

#[test]
fn preview_status_includes_material_table_and_resource_layout_identity() {
    let mut runtime = MaterialLabRuntime::default();
    let product = preview_scene_product("active", "table-a", "layout-a");
    runtime
        .record_preview_scene_product(&product.request_identity(), product)
        .expect("fresh preview scene product should record");

    let view = runtime.preview_view_model(&AssetCatalog::new());

    assert_eq!(
        view.preview_status.material_table_identity_label.as_deref(),
        Some("material table table-a")
    );
    assert_eq!(
        view.preview_status
            .resource_layout_identity_label
            .as_deref(),
        Some("resource layout layout-a")
    );
    assert_eq!(
        view.preview_status
            .preview_scene_product_shader_identity_label
            .as_deref(),
        Some("shader identity shader-identity-active")
    );
    assert_eq!(
        view.preview_status
            .preview_scene_product_shader_artifact_label
            .as_deref(),
        Some("shader artifact shader-artifact-active cache shader-cache-active")
    );
    assert_eq!(view.preview_status.slot_count, Some(2));
    assert_eq!(view.preview_status.resource_slot_count, Some(2));
}

#[test]
fn preview_status_includes_last_valid_preview_scene_product() {
    let mut runtime = MaterialLabRuntime::default();
    let product = preview_scene_product("active", "table-a", "layout-a");
    let request_identity = product.request_identity();
    runtime
        .record_preview_scene_product(&request_identity, product.clone())
        .expect("fresh preview scene product should record");
    assert!(runtime.record_preview_scene_product_failure(Some(&request_identity)));

    let view = runtime.preview_view_model(&AssetCatalog::new());

    assert_eq!(
        view.preview_status
            .last_valid_preview_scene_product_identity
            .as_deref(),
        Some(product.product_identity.as_str())
    );
    assert_eq!(
        view.preview_status
            .preview_scene_product_identity
            .as_deref(),
        Some(product.product_identity.as_str())
    );
    assert!(
        view.preview_status
            .preview_scene_product_status_label
            .as_deref()
            .is_some_and(|label| label.contains("last-valid preview scene product preserved"))
    );
}

#[test]
fn preview_status_reports_preview_scene_product_failure_reason() {
    let mut runtime = MaterialLabRuntime::default();
    runtime.record_preview_scene_product_failure(None);
    runtime.record_diagnostic(AssetDiagnosticRecord::error(
        AssetDiagnosticCode::RatificationRejected,
        "current preview scene product is stale for the active material preview request",
    ));

    let view = runtime.preview_view_model(&AssetCatalog::new());

    assert_eq!(
        view.preview_status
            .preview_scene_product_failure_reason
            .as_deref(),
        Some("current preview scene product is stale for the active material preview request")
    );
}

#[test]
fn diagnostics_rows_include_stale_preview_scene_product_state() {
    let mut runtime = MaterialLabRuntime::default();
    runtime.record_preview_scene_product_failure(None);
    runtime.record_diagnostic(AssetDiagnosticRecord::error(
            AssetDiagnosticCode::RatificationRejected,
            "scene material table generated shader bundle is stale for the current material table/resource layout",
        ));

    let view = runtime.preview_view_model(&AssetCatalog::new());

    assert!(view.diagnostic_rows.iter().any(|row| {
        row.code == "material.preview_scene.generated_bundle_stale"
            && row.category_label.as_deref() == Some("preview scene product")
            && row.message.contains("generated shader bundle is stale")
    }));
}

#[test]
fn dto_remains_presentation_only() {
    let dto_source =
        include_str!("../../../../../domain/editor/editor_shell/src/surfaces/material.rs");

    assert!(dto_source.contains("preview_scene_product_identity: Option<String>"));
    assert!(!dto_source.contains("PreviewSceneProduct"));
    assert!(!dto_source.contains("PreparedSceneMaterialBundle"));
    assert!(!dto_source.contains("PreparedMaterialFeatureContribution"));
}

#[test]
fn material_runtime_records_current_preview_scene_product() {
    let mut runtime = MaterialLabRuntime::default();
    let product = preview_scene_product("active", "table-a", "layout-a");
    let request_identity = product.request_identity();

    runtime
        .record_preview_scene_product(&request_identity, product.clone())
        .expect("fresh product should record");

    assert_eq!(runtime.current_preview_scene_product(), Some(&product));
    assert_eq!(runtime.last_valid_preview_scene_product(), Some(&product));
    assert_eq!(
        runtime.preview_scene_product_status(),
        PreviewSceneProductRuntimeStatus::Current {
            product_identity: product.product_identity
        }
    );
}

#[test]
fn material_runtime_preserves_last_valid_preview_scene_product_for_same_request() {
    let mut runtime = MaterialLabRuntime::default();
    let product = preview_scene_product("active", "table-a", "layout-a");
    let request_identity = product.request_identity();
    runtime
        .record_preview_scene_product(&request_identity, product.clone())
        .expect("fresh product should record");

    let preserved = runtime.record_preview_scene_product_failure(Some(&request_identity));

    assert!(preserved);
    assert_eq!(runtime.current_preview_scene_product(), None);
    assert_eq!(runtime.last_valid_preview_scene_product(), Some(&product));
    assert_eq!(
        runtime.preview_scene_product_status(),
        PreviewSceneProductRuntimeStatus::LastValidPreserved {
            product_identity: product.product_identity
        }
    );
}

#[test]
fn material_runtime_rejects_last_valid_preview_scene_product_for_different_resource_layout() {
    let mut runtime = MaterialLabRuntime::default();
    let product = preview_scene_product("active", "table-a", "layout-a");
    let changed_layout = preview_scene_product("active", "table-a", "layout-b");
    runtime
        .record_preview_scene_product(&product.request_identity(), product)
        .expect("fresh product should record");

    let preserved =
        runtime.record_preview_scene_product_failure(Some(&changed_layout.request_identity()));

    assert!(!preserved);
    assert_eq!(runtime.current_preview_scene_product(), None);
    assert_eq!(runtime.last_valid_preview_scene_product(), None);
    assert_eq!(
        runtime.preview_scene_product_status(),
        PreviewSceneProductRuntimeStatus::Empty
    );
}

#[test]
fn material_runtime_rejects_last_valid_preview_scene_product_for_different_material_table() {
    let mut runtime = MaterialLabRuntime::default();
    let product = preview_scene_product("active", "table-a", "layout-a");
    let changed_table = preview_scene_product("active", "table-b", "layout-a");
    runtime
        .record_preview_scene_product(&product.request_identity(), product)
        .expect("fresh product should record");

    let preserved =
        runtime.record_preview_scene_product_failure(Some(&changed_table.request_identity()));

    assert!(!preserved);
    assert_eq!(runtime.current_preview_scene_product(), None);
    assert_eq!(runtime.last_valid_preview_scene_product(), None);
}

#[test]
fn failed_scene_table_bundle_preserves_last_valid_only_for_same_request() {
    let mut runtime = MaterialLabRuntime::default();
    let product = preview_scene_product("active", "table-a", "layout-a");
    let same_request_without_shader = request_identity_without_shader(&product);
    runtime
        .record_preview_scene_product(&product.request_identity(), product.clone())
        .expect("fresh product should record");

    assert!(runtime.record_preview_scene_product_failure(Some(&same_request_without_shader)));
    assert_eq!(runtime.last_valid_preview_scene_product(), Some(&product));

    let changed_layout = preview_scene_product("active", "table-a", "layout-b");
    let changed_request_without_shader = request_identity_without_shader(&changed_layout);
    assert!(!runtime.record_preview_scene_product_failure(Some(&changed_request_without_shader)));
    assert_eq!(runtime.last_valid_preview_scene_product(), None);
}

#[test]
fn unresolved_explicit_scene_slot_clears_current_product_and_preserves_only_valid_same_request() {
    let mut runtime = MaterialLabRuntime::default();
    let product = preview_scene_product("active", "table-a", "layout-a");
    runtime
        .record_preview_scene_product(&product.request_identity(), product.clone())
        .expect("fresh product should record");

    assert!(!runtime.record_preview_scene_product_failure(None));
    assert_eq!(runtime.current_preview_scene_product(), None);
    assert_eq!(runtime.last_valid_preview_scene_product(), None);

    runtime
        .record_preview_scene_product(&product.request_identity(), product.clone())
        .expect("fresh product should record");
    assert!(runtime.record_preview_scene_product_failure(Some(&product.request_identity())));
    assert_eq!(runtime.last_valid_preview_scene_product(), Some(&product));
}

#[test]
fn no_v5_or_asset_persistence_for_preview_scene_product_state() {
    let runtime_source = include_str!("runtime.rs");
    let workspace_layout_source = include_str!("../../persistence/workspace_layout.rs");

    assert!(runtime_source.contains("current_preview_scene_product"));
    assert!(runtime_source.contains("last_valid_preview_scene_product"));
    assert!(!workspace_layout_source.contains("PreviewSceneProduct"));
    assert!(!workspace_layout_source.contains("current_preview_scene_product"));
    assert!(!workspace_layout_source.contains("last_valid_preview_scene_product"));
}

#[derive(Debug, Clone, Copy)]
enum TexturePayloadFixture {
    Imported,
    Generated,
}

fn texture_source_document(reference: Option<ResourceRef>) -> MaterialGraphDocument {
    let mut texture_node = NodeDefinition::new(NodeId::new(11), "texture.sample_2d", []);
    if let Some(reference) = reference {
        texture_node = texture_node.with_values([graph::GraphMetadataEntry::new(
            material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
            graph::GraphValue::resource(reference),
        )]);
    }
    MaterialGraphDocument::new(
        material_graph::MaterialGraphDocumentId::new(901),
        "texture-diagnostics",
        GraphDefinition::new(
            GraphId::new(901),
            "texture-diagnostics",
            CyclePolicy::RejectDirectedCycles,
            [texture_node],
            [],
        ),
        MaterialOutputTarget::RenderMaterial,
    )
}

fn insert_texture_asset(
    catalog: &mut AssetCatalog,
    asset_id: AssetId,
    artifact_id: AssetArtifactId,
    stable_name: &str,
    payload_fixture: TexturePayloadFixture,
    validity: ArtifactValidity,
) {
    catalog.insert_asset_record(AssetRecord::new(
        asset_id,
        stable_name,
        stable_name,
        AssetKind::Texture2D,
    ));
    let descriptor = texture_descriptor(
        artifact_id.raw(),
        TextureDimension::Texture2D,
        TextureExtent::new(4, 4, 1),
    );
    let payload_kind = match payload_fixture {
        TexturePayloadFixture::Imported => ArtifactPayloadKind::TextureProduct {
            descriptor_hash: descriptor.descriptor_hash().to_string(),
            descriptor,
            artifact_uri: Some(format!(
                ".runenwerk/artifacts/texture-{}.ktx2",
                artifact_id.raw()
            )),
        },
        TexturePayloadFixture::Generated => ArtifactPayloadKind::GeneratedTextureProduct {
            descriptor_hash: descriptor.descriptor_hash().to_string(),
            descriptor,
            artifact_uri: Some(format!(
                ".runenwerk/artifacts/generated-texture-{}.ktx2",
                artifact_id.raw()
            )),
        },
    };
    catalog.insert_artifact(
        AssetArtifactDescriptor::new(
            artifact_id,
            asset_id,
            AssetKind::Texture2D,
            payload_kind,
            ArtifactCacheKey::new(format!("texture-cache-{}", artifact_id.raw())),
        )
        .with_artifact_path(format!(
            ".runenwerk/artifacts/texture-{}.ktx2",
            artifact_id.raw()
        ))
        .with_validity(validity),
    );
}

#[allow(clippy::too_many_arguments)]
fn insert_foreign_mesh_reference_artifact(
    catalog: &mut AssetCatalog,
    asset_id: AssetId,
    source_id: asset::AssetSourceId,
    artifact_id: AssetArtifactId,
    stable_name: &str,
    display_name: &str,
    material_region_key: &str,
    material_region_label: &str,
) {
    catalog.insert_asset_record(
        AssetRecord::new(
            asset_id,
            stable_name,
            display_name,
            AssetKind::ForeignMeshReferenceSource,
        )
        .with_primary_source(source_id),
    );
    catalog.insert_source(
        AssetSourceDescriptor::new(
            source_id,
            asset_id,
            AssetKind::ForeignMeshReferenceSource,
            format!("assets/models/{stable_name}.gltf"),
        )
        .with_hash(SourceHash::new("sha256", "mesh")),
    );
    catalog.insert_artifact(
        AssetArtifactDescriptor::new(
            artifact_id,
            asset_id,
            AssetKind::ForeignMeshReferenceArtifact,
            ArtifactPayloadKind::ForeignReference {
                format: "gltf".to_string(),
                material_regions: vec![
                    ForeignMeshMaterialRegionDescriptor::importer_authored(
                        material_region_key,
                        material_region_label,
                    )
                    .expect("test material region key should be stable"),
                ],
            },
            ArtifactCacheKey::new(format!("{stable_name}-mesh-cache")),
        )
        .with_source(source_id, asset::asset_source_revision_id(1)),
    );
}

fn texture_descriptor(
    product_id: u64,
    dimension: TextureDimension,
    extent: TextureExtent,
) -> TextureDescriptor {
    let descriptor = TextureDescriptor::new(
        TextureProductId::new(product_id),
        format!("texture.{product_id}"),
        dimension,
        extent,
    );
    let mip_count = descriptor.mip_count;
    let descriptor_hash = descriptor.descriptor_hash().to_string();
    descriptor.with_ktx2_metadata(
        Ktx2TextureMetadata::new(
            TexturePixelFormat::Rgba8Unorm,
            mip_count,
            descriptor_hash,
            "1",
        )
        .with_byte_layout(128, [64]),
    )
}

fn test_preview_product(asset_id: AssetId) -> EditorMaterialPreviewProduct {
    let source_id = asset_source_id(22);
    let product = FormedMaterialProduct::new(
        MaterialProductId::new(30),
        material_graph::MaterialGraphDocumentId::new(31),
        MaterialOutputTarget::RenderMaterial,
        material_graph::MaterialCacheKey::new("material-preview-cache"),
    );
    EditorMaterialPreviewProduct::new(
        asset_id,
        source_id,
        asset_artifact_id(32),
        ArtifactCacheKey::new("artifact-cache"),
        product,
        MaterialRendererParameterProfile::RenderMaterial,
        asset_artifact_id(33),
        ArtifactCacheKey::new("shader-cache"),
        ".runenwerk/artifacts/material.wgsl",
        "material-shader",
        asset_artifact_id(34),
        ArtifactCacheKey::new("scene-shader-cache"),
        ".runenwerk/artifacts/scene-material.wgsl",
        "scene-material-shader",
        [],
    )
}

fn preview_scene_product(
    label: &str,
    material_table_identity: &str,
    resource_layout_identity: &str,
) -> PreviewSceneProduct {
    let shader = PreviewSceneShaderProductRef::new(
        format!("shader-artifact-{label}"),
        ArtifactCacheKey::new(format!("shader-cache-{label}")),
        format!("shader-identity-{label}"),
        format!(".runenwerk/artifacts/material-scene-shader-{label}.wgsl"),
        material_table_identity,
        resource_layout_identity,
    );
    PreviewSceneProduct::new(
        PreviewSceneProductMode::SceneMaterialTable,
        ExpressionProductId(10030),
        MaterialProductId::new(30),
        ArtifactCacheKey::new("active-material-cache"),
        material_table_identity,
        resource_layout_identity,
        shader,
        [
            PreviewSceneMaterialSlot::new(
                0,
                "default",
                MaterialProductId::new(30),
                ArtifactCacheKey::new("material-cache-default"),
                "scene-shader-default",
                [PreviewSceneResourceSlotMapping::new(0, 0)],
            ),
            PreviewSceneMaterialSlot::new(
                1,
                "assigned",
                MaterialProductId::new(31),
                ArtifactCacheKey::new("material-cache-assigned"),
                "scene-shader-assigned",
                [PreviewSceneResourceSlotMapping::new(0, 1)],
            ),
        ],
        [
            preview_scene_resource_slot(0, "albedo"),
            preview_scene_resource_slot(1, "normal"),
        ],
    )
}

fn request_identity_without_shader(
    product: &PreviewSceneProduct,
) -> PreviewSceneProductRequestIdentity {
    PreviewSceneProductRequestIdentity::new(
        product.mode,
        product.viewport_product_id,
        product.active_material_product_id,
        &product.active_material_artifact_cache_key,
        &product.material_table_identity,
        &product.resource_layout_identity,
        None,
        &product.slots,
        &product.resources,
    )
}

fn preview_scene_resource_slot(table_resource_slot: u32, label: &str) -> PreviewSceneResourceSlot {
    PreviewSceneResourceSlot::new(
        table_resource_slot,
        format!("texture-product-{label}"),
        "Texture2D",
        "2d",
        "rgba8_unorm_srgb|sampled",
        "linear_repeat",
        format!("texture-artifact-{label}"),
        ArtifactCacheKey::new(format!("texture-cache-{label}")),
    )
}
