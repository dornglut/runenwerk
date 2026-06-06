use super::*;
use asset::{
    AssetDiagnosticCode, AssetDiagnosticRecord, AssetProjectCatalogDescriptor, AssetRecord,
    AssetSourceDescriptor, AssetSourceRoot, AssetSourceRootKind,
    ForeignMeshMaterialRegionDescriptor, ImportSettings, SourceHash, asset_artifact_id, asset_id,
    asset_source_id, asset_source_root_id,
};
use editor_persistence::{
    ProjectFileV3, ProjectImportProfileDefaultV3, ProjectImportProfileDefinitionV3,
};
use editor_scene::{
    SceneMaterialAssignmentState, SceneMaterialPalette, SceneMaterialSlot, SceneMaterialSlotId,
    SceneMeshMaterialRegionId, SceneModelMeshMaterialRegionSourceId, SceneModelMeshSourceId,
};
use texture::{TextureDescriptor, TextureDimension, TextureExtent, TextureProductId};

#[test]
fn generated_default_material_graph_is_colored_sdf_first_source_graph() {
    let asset_id = asset_id(91);
    let source = AssetSourceDescriptor::new(
        asset_source_id(92),
        asset_id,
        AssetKind::MaterialGraph,
        "assets/materials/default.material.ron",
    );

    let document = default_material_graph_document_for_source_with_target(
        asset_id,
        &source,
        "Default",
        MaterialOutputTarget::RenderMaterial,
    );

    assert_eq!(document.graph.nodes.len(), 2);
    assert_eq!(document.graph.edges.len(), 1);
    assert_eq!(
        document.editor_state.selected_fixture,
        material_graph::MaterialGraphPreviewFixture::SdfPrimitive
    );
    assert_eq!(
        document.editor_state.selected_preview,
        material_graph::MaterialGraphPreviewSelection::SceneProduct
    );
    let base_color = document
        .graph
        .nodes
        .iter()
        .find(|node| node.name == "pbr.base_color")
        .expect("default graph should own an explicit color node");
    assert_eq!(
        base_color.value("color"),
        Some(&graph::GraphValue::Text("0.08 0.62 0.95 1.0".to_string()))
    );
    let lowering = lower_material_graph(&document, &MaterialNodeCatalog::first_slice());
    assert!(
        lowering.report.is_accepted(),
        "{:?}",
        lowering.report.issues()
    );
}

#[test]
fn material_graph_node_move_persists_source_layout() {
    let mut document = command_test_document();

    let changed = apply_material_document_action(
        &mut document,
        &[],
        &[],
        &MaterialSurfaceAction::MoveGraphNode {
            node_id: graph::NodeId::new(1),
            delta_x: 40,
            delta_y: -12,
        },
    )
    .expect("node move should be accepted");

    assert!(changed);
    let restored = material_graph::MaterialGraphSourceFileV2::from_document(&document)
        .into_document()
        .expect("source should round-trip");
    assert_eq!(
        restored.editor_state.node_layouts,
        vec![material_graph::MaterialGraphNodeLayout::new(
            graph::NodeId::new(1),
            40,
            -12
        )]
    );
}

#[test]
fn material_source_edit_records_load_failure_diagnostic() {
    let mut app = RunenwerkEditorApp::new();
    crate::material_lab::ensure_default_material_source_document(&mut app);

    let result = app.apply_material_surface_action(MaterialSurfaceAction::MoveGraphNode {
        node_id: graph::NodeId::new(1),
        delta_x: 8,
        delta_y: -4,
    });

    assert!(result.is_err());
    assert!(
        app.material_lab_runtime()
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.message.contains(
                "failed to load material graph source for edit: cannot load material graph document: no asset project session"
            )),
        "source-backed edit load failures should preserve the concrete cause"
    );
}

#[test]
fn material_graph_connect_ports_mutates_source_graph() {
    let mut document = command_test_document();

    apply_material_document_action(
        &mut document,
        &[],
        &[],
        &MaterialSurfaceAction::ConnectPorts {
            from_port_id: graph::PortId::new(1),
            to_port_id: graph::PortId::new(2),
        },
    )
    .expect("compatible ports should connect");

    assert_eq!(
        document.graph.edges,
        vec![graph::EdgeDefinition::new(
            graph::EdgeId::new(1),
            graph::PortId::new(1),
            graph::PortId::new(2)
        )]
    );
}

#[test]
fn material_graph_disconnect_edge_mutates_source_graph() {
    let mut document = command_test_document();
    material_graph::connect_ports(&mut document, graph::PortId::new(1), graph::PortId::new(2))
        .expect("setup edge");

    apply_material_document_action(
        &mut document,
        &[],
        &[],
        &MaterialSurfaceAction::DisconnectEdge {
            edge_id: graph::EdgeId::new(1),
        },
    )
    .expect("edge disconnect should be accepted");

    assert!(document.graph.edges.is_empty());
}

#[test]
fn material_graph_property_edit_mutates_source_graph() {
    let mut document = command_test_document();

    apply_material_document_action(
        &mut document,
        &[],
        &[],
        &MaterialSurfaceAction::SetNodeValue {
            node_id: graph::NodeId::new(1),
            key: "roughness".to_string(),
            value: "0.25".to_string(),
        },
    )
    .expect("property edit should be accepted");

    let value = document.graph.nodes[0]
        .value("roughness")
        .expect("node value should be present");
    assert_eq!(value, &graph::GraphValue::Text("0.25".to_string()));
}

#[test]
fn material_graph_texture_ref_edit_mutates_source_graph() {
    let mut document = texture_edit_command_document();

    apply_material_document_action(
        &mut document,
        &[],
        &[],
        &MaterialSurfaceAction::PickTextureResource {
            node_id: graph::NodeId::new(1),
            key: material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF.to_string(),
            stable_id: "rock.albedo".to_string(),
        },
    )
    .expect("texture ref edit should be accepted");

    let value = document.graph.nodes[0]
        .value(material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF)
        .expect("texture ref should be present");
    let graph::GraphValue::Resource(reference) = value else {
        panic!("texture ref should be stored as a resource value");
    };
    assert_eq!(reference.kind.as_str(), "asset.catalog.texture2d");
    assert_eq!(reference.stable_id.as_str(), "rock.albedo");
}

#[test]
fn material_graph_node_drag_is_one_undo_transaction() {
    let root = unique_temp_dir("material_graph_node_drag_undo");
    let asset_id = asset_id(41);
    let source_id = asset_source_id(42);
    let mut app = material_edit_app(&root, asset_id, source_id, command_test_document());

    app.apply_material_surface_action(MaterialSurfaceAction::MoveGraphNode {
        node_id: graph::NodeId::new(1),
        delta_x: 10,
        delta_y: 4,
    })
    .expect("drag commit should apply as one material edit");

    assert!(app.material_lab_runtime().can_undo());
    app.apply_material_surface_action(MaterialSurfaceAction::UndoMaterialEdit)
        .expect("one undo should roll back the drag commit");

    let document = app
        .load_material_graph_document_for_asset(asset_id)
        .expect("document should reload after undo");
    assert!(document.editor_state.node_layouts.is_empty());
    assert!(
        !app.material_lab_runtime().can_undo(),
        "node drag commit must create exactly one undo snapshot"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn material_graph_property_edit_groups_commit_correctly() {
    let root = unique_temp_dir("material_graph_property_edit_undo");
    let asset_id = asset_id(51);
    let source_id = asset_source_id(52);
    let mut app = material_edit_app(&root, asset_id, source_id, command_test_document());

    app.apply_material_surface_action(MaterialSurfaceAction::SetNodeValue {
        node_id: graph::NodeId::new(1),
        key: "roughness".to_string(),
        value: "0.25".to_string(),
    })
    .expect("property commit should apply as one material edit");

    assert!(app.material_lab_runtime().can_undo());
    app.apply_material_surface_action(MaterialSurfaceAction::UndoMaterialEdit)
        .expect("one undo should roll back the property commit");

    let document = app
        .load_material_graph_document_for_asset(asset_id)
        .expect("document should reload after undo");
    assert_eq!(document.graph.nodes[0].value("roughness"), None);
    assert!(
        !app.material_lab_runtime().can_undo(),
        "property commit must create exactly one undo snapshot"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn material_graph_texture_picker_selection_updates_source_ref() {
    let root = unique_temp_dir("material_graph_texture_picker_select");
    let material_asset_id = asset_id(61);
    let source_id = asset_source_id(62);
    let mut app = material_edit_app(
        &root,
        material_asset_id,
        source_id,
        texture_edit_command_document(),
    );
    insert_texture_product(
        app.asset_catalog_runtime_mut().catalog_mut(),
        asset_id(63),
        asset_artifact_id(64),
        "rock.albedo",
        "Rock Albedo",
        65,
        TextureDimension::Texture2D,
    );

    app.apply_material_surface_action(MaterialSurfaceAction::PickTextureResource {
        node_id: graph::NodeId::new(1),
        key: material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF.to_string(),
        stable_id: "rock.albedo".to_string(),
    })
    .expect("catalog-backed texture picker selection should update source ref");

    let document = app
        .load_material_graph_document_for_asset(material_asset_id)
        .expect("edited material document should reload");
    let value = document.graph.nodes[0]
        .value(material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF)
        .expect("texture ref should be present");
    let graph::GraphValue::Resource(reference) = value else {
        panic!("texture picker should write a resource ref");
    };
    assert_eq!(reference.kind.as_str(), "asset.catalog.texture2d");
    assert_eq!(reference.stable_id.as_str(), "rock.albedo");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn material_graph_texture_picker_rejects_missing_texture_product() {
    let root = unique_temp_dir("material_graph_texture_picker_missing");
    let asset_id = asset_id(66);
    let source_id = asset_source_id(67);
    let mut app = material_edit_app(&root, asset_id, source_id, texture_edit_command_document());

    let result = app.apply_material_surface_action(MaterialSurfaceAction::PickTextureResource {
        node_id: graph::NodeId::new(1),
        key: material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF.to_string(),
        stable_id: "missing.albedo".to_string(),
    });

    assert!(result.is_err());
    assert!(
        app.material_lab_runtime()
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.message.contains("not a catalog texture asset") })
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn model_mesh_material_region_assignment_uses_catalog_source_identity() {
    let mut app = RunenwerkEditorApp::new();
    let model_asset_id = asset_id(301);
    let model_source_id = asset_source_id(302);
    let model_artifact_id = asset_artifact_id(303);
    insert_foreign_mesh_reference_artifact(
        app.asset_catalog_runtime_mut().catalog_mut(),
        model_asset_id,
        model_source_id,
        model_artifact_id,
        "hero.model",
        "Hero Model",
        "source_material_slot:0",
        "Body",
    );
    let palette = SceneMaterialPalette::new([
        SceneMaterialSlot::default_generated(),
        SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Hero Body")
            .with_material_asset(asset_id(304)),
    ])
    .expect("palette should be valid");
    app.runtime_mut().replace_scene_material_assignments(
        SceneMaterialAssignmentState::new_with_model_mesh_assignments(palette, [], [])
            .expect("scene material assignments should form"),
    );

    app.apply_material_surface_action(MaterialSurfaceAction::AssignModelMeshMaterialSlot {
        model_asset_id,
        material_region_key: "source_material_slot:0".to_string(),
        slot_id: SceneMaterialSlotId::new(2),
    })
    .expect("catalog-backed model mesh material region should assign");

    let material_region = SceneModelMeshMaterialRegionSourceId::new(
        SceneModelMeshSourceId::new(model_asset_id, model_source_id)
            .with_source_revision_id(asset::asset_source_revision_id(1))
            .with_source_revision("sha256:mesh"),
        SceneMeshMaterialRegionId::new("source_material_slot:0")
            .expect("stable material region key"),
    );
    let resolution = app
        .runtime()
        .scene_material_assignments()
        .resolve_material_slot_for_model_mesh_region(&material_region);
    assert_eq!(resolution.resolved_slot_id, SceneMaterialSlotId::new(2));
    assert!(
        app.runtime()
            .scene_material_assignments()
            .material_table_identity()
            .contains("source_revision=sha256:mesh")
    );
}

#[test]
fn model_mesh_material_region_assignment_rejects_missing_region() {
    let mut app = RunenwerkEditorApp::new();
    let model_asset_id = asset_id(311);
    insert_foreign_mesh_reference_artifact(
        app.asset_catalog_runtime_mut().catalog_mut(),
        model_asset_id,
        asset_source_id(312),
        asset_artifact_id(313),
        "hero.model",
        "Hero Model",
        "source_material_slot:0",
        "Body",
    );

    let result =
        app.apply_material_surface_action(MaterialSurfaceAction::AssignModelMeshMaterialSlot {
            model_asset_id,
            material_region_key: "source_material_slot:99".to_string(),
            slot_id: SceneMaterialSlotId::new(1),
        });

    assert!(result.is_err());
    assert!(
        app.material_lab_runtime()
            .diagnostics()
            .iter()
            .any(|diagnostic| {
                diagnostic
                    .message
                    .contains("does not expose material region 'source_material_slot:99'")
            })
    );
}

#[test]
fn model_mesh_assignment_survives_asset_rebuild() {
    let mut app = RunenwerkEditorApp::new();
    let model_asset_id = asset_id(321);
    let model_source_id = asset_source_id(322);
    let initial_artifact_id = asset_artifact_id(323);
    let rebuilt_artifact_id = asset_artifact_id(324);
    insert_foreign_mesh_reference_artifact(
        app.asset_catalog_runtime_mut().catalog_mut(),
        model_asset_id,
        model_source_id,
        initial_artifact_id,
        "rebuilt.hero",
        "Rebuilt Hero",
        "source_material_slot:0",
        "Body",
    );
    let assigned_slot = SceneMaterialSlotId::new(2);
    let palette = SceneMaterialPalette::new([
        SceneMaterialSlot::default_generated(),
        SceneMaterialSlot::new(assigned_slot, "Hero Body").with_material_asset(asset_id(325)),
    ])
    .expect("palette should be valid");
    app.runtime_mut().replace_scene_material_assignments(
        SceneMaterialAssignmentState::new_with_model_mesh_assignments(palette, [], [])
            .expect("scene material assignments should form"),
    );
    app.apply_material_surface_action(MaterialSurfaceAction::AssignModelMeshMaterialSlot {
        model_asset_id,
        material_region_key: "source_material_slot:0".to_string(),
        slot_id: assigned_slot,
    })
    .expect("initial source-backed region should assign");

    insert_foreign_mesh_reference_artifact_descriptor(
        app.asset_catalog_runtime_mut().catalog_mut(),
        model_asset_id,
        model_source_id,
        rebuilt_artifact_id,
        "rebuilt.hero",
        "source_material_slot:0",
        "Body",
    );

    let view = app
        .material_lab_runtime()
        .graph_canvas_view_model_with_scene_material_assignments(
            app.asset_catalog_runtime().catalog(),
            Vec::new(),
            Some(app.runtime().scene_material_assignments()),
        );

    assert_eq!(view.model_mesh_regions.len(), 1);
    assert_eq!(view.model_mesh_regions[0].artifact_id, rebuilt_artifact_id);
    assert_eq!(
        view.model_mesh_regions[0].assigned_slot_id,
        Some(assigned_slot)
    );
}

#[test]
fn model_mesh_material_region_assignment_rejects_missing_catalog_source() {
    let mut app = RunenwerkEditorApp::new();
    let model_asset_id = asset_id(331);
    let model_source_id = asset_source_id(332);
    let model_artifact_id = asset_artifact_id(333);
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(
            AssetRecord::new(
                model_asset_id,
                "orphan.model",
                "Orphan Model",
                AssetKind::ForeignMeshReferenceSource,
            )
            .with_primary_source(model_source_id),
        );
    insert_foreign_mesh_reference_artifact_descriptor(
        app.asset_catalog_runtime_mut().catalog_mut(),
        model_asset_id,
        model_source_id,
        model_artifact_id,
        "orphan.model",
        "source_material_slot:0",
        "Body",
    );

    let result =
        app.apply_material_surface_action(MaterialSurfaceAction::AssignModelMeshMaterialSlot {
            model_asset_id,
            material_region_key: "source_material_slot:0".to_string(),
            slot_id: SceneMaterialSlotId::new(1),
        });

    assert!(result.is_err());
    assert!(
        app.material_lab_runtime()
            .diagnostics()
            .iter()
            .any(|diagnostic| {
                diagnostic
                    .message
                    .contains("references missing catalog source 332")
            })
    );
}

#[test]
fn model_mesh_region_projection_keeps_unassigned_region_unassigned() {
    let mut app = RunenwerkEditorApp::new();
    let model_asset_id = asset_id(341);
    insert_foreign_mesh_reference_artifact(
        app.asset_catalog_runtime_mut().catalog_mut(),
        model_asset_id,
        asset_source_id(342),
        asset_artifact_id(343),
        "unassigned.model",
        "Unassigned Model",
        "source_material_slot:0",
        "Body",
    );
    let assignments = SceneMaterialAssignmentState::default();

    let view = app
        .material_lab_runtime()
        .graph_canvas_view_model_with_scene_material_assignments(
            app.asset_catalog_runtime().catalog(),
            Vec::new(),
            Some(&assignments),
        );

    assert_eq!(view.model_mesh_regions.len(), 1);
    assert_eq!(view.model_mesh_regions[0].assigned_slot_id, None);
    assert_eq!(view.model_mesh_regions[0].assigned_slot_label, None);
}

#[test]
fn material_graph_node_picker_confirm_adds_highlighted_node_to_source() {
    let root = unique_temp_dir("material_graph_node_picker_confirm");
    let asset_id = asset_id(68);
    let source_id = asset_source_id(69);
    let mut app = material_edit_app(&root, asset_id, source_id, command_test_document());

    app.apply_material_surface_action(MaterialSurfaceAction::OpenNodePicker)
        .expect("node picker should open");
    app.apply_material_surface_action(MaterialSurfaceAction::HighlightNodePickerNode {
        descriptor_key: "pbr.base_color".to_string(),
    })
    .expect("catalog node should highlight");
    app.apply_material_surface_action(MaterialSurfaceAction::ConfirmNodePickerSelection)
        .expect("highlighted catalog node should be added through source-backed workflow");

    let document = app
        .load_material_graph_document_for_asset(asset_id)
        .expect("edited material document should reload");
    assert!(
        document
            .graph
            .nodes
            .iter()
            .any(|node| node.name == "pbr.base_color"),
        "modal confirmation must route through AddGraphNode and persist the source file",
    );
    assert!(
        !app.material_lab_runtime()
            .graph_canvas_view_model(&AssetCatalog::new(), Vec::new())
            .node_picker
            .open,
        "successful modal confirmation should close the node picker",
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn material_graph_navigate_diagnostic_selects_and_centers_subject() {
    let root = unique_temp_dir("material_graph_navigate_diagnostic");
    let asset_id = asset_id(70);
    let source_id = asset_source_id(71);
    let mut app = material_edit_app(&root, asset_id, source_id, command_test_document());
    app.material_lab_runtime_mut().record_diagnostic(
        AssetDiagnosticRecord::error(
            AssetDiagnosticCode::RatificationRejected,
            "roughness input is missing",
        )
        .with_subject("material_graph.port:2"),
    );

    app.apply_material_surface_action(MaterialSurfaceAction::NavigateDiagnostic {
        diagnostic_index: 0,
    })
    .expect("diagnostic navigation should load and persist source viewport state");

    let document = app
        .load_material_graph_document_for_asset(asset_id)
        .expect("material document should reload after navigation");
    assert_eq!(
        app.material_lab_runtime().active_diagnostic_index(),
        Some(0)
    );
    assert!(
        app.material_lab_runtime()
            .selected_graph_nodes()
            .contains(&graph::NodeId::new(2)),
        "port diagnostic navigation should select the owning node",
    );
    assert_eq!(document.editor_state.viewport.pan_x, 100);
    assert_eq!(document.editor_state.viewport.pan_y, 220);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn material_preview_build_lowers_source_document_and_queues_publication() {
    let root = unique_temp_dir("material_preview_build");
    let asset_id = asset_id(1);
    let source_id = asset_source_id(2);
    let mut project = ProjectFileV3::new("project.material", "Material");
    project
        .import_profile_definitions
        .push(ProjectImportProfileDefinitionV3::new(
            AssetKind::MaterialGraph,
            "render",
            ImportSettings::MaterialGraph {
                lowering_target: "render_material".to_string(),
            },
            AssetKind::Material,
        ));
    project
        .import_profile_defaults
        .push(ProjectImportProfileDefaultV3::new(
            AssetKind::MaterialGraph,
            "render",
        ));
    let mut session =
        crate::asset_pipeline::EditorAssetProjectSession::from_project_file(&root, &project)
            .expect("project session should form");
    let source = AssetSourceDescriptor::new(
        source_id,
        asset_id,
        AssetKind::MaterialGraph,
        "assets/materials/rock.material.ron",
    )
    .with_hash(SourceHash::new("sha256", "abc"));
    let document = default_material_graph_document_for_source_with_target(
        asset_id,
        &source,
        "Rock",
        MaterialOutputTarget::RenderMaterial,
    );
    write_material_graph_document(&root.join(&source.relative_path), &document)
        .expect("source document should write");
    let mut catalog = AssetCatalog::new();
    catalog.insert_asset_record(
        AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
            .with_primary_source(source_id),
    );
    catalog.insert_source(source);

    let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

    assert!(outcome.diagnostics.is_empty());
    let publication = outcome.publication.expect("preview should queue");
    assert_eq!(publication.status, product::ProductPublicationStatus::Ready);
    assert_eq!(
        publication.preview.as_ref().map(|preview| preview.asset_id),
        Some(asset_id)
    );
    assert!(
        publication
            .preview
            .as_ref()
            .expect("preview")
            .shader_path
            .starts_with(
                &root
                    .to_string_lossy()
                    .replace(std::path::MAIN_SEPARATOR, "/")
            ),
        "runtime shader registry path must be project-root aware"
    );
    assert_eq!(session.import_ledger().len(), 3);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn material_recipe_rejection_blocks_before_ledger_allocation() {
    let root = unique_temp_dir("material_recipe_rejected");
    let asset_id = asset_id(1);
    let source_id = asset_source_id(2);
    let mut project = ProjectFileV3::new("project.material", "Material");
    project
        .import_profile_definitions
        .push(ProjectImportProfileDefinitionV3::new(
            AssetKind::MaterialGraph,
            "bad",
            ImportSettings::MaterialGraph {
                lowering_target: String::new(),
            },
            AssetKind::Material,
        ));
    project
        .import_profile_defaults
        .push(ProjectImportProfileDefaultV3::new(
            AssetKind::MaterialGraph,
            "bad",
        ));
    let mut session =
        crate::asset_pipeline::EditorAssetProjectSession::from_project_file(&root, &project)
            .expect("project session should form");
    let source = AssetSourceDescriptor::new(
        source_id,
        asset_id,
        AssetKind::MaterialGraph,
        "assets/materials/rock.material.ron",
    )
    .with_hash(SourceHash::new("sha256", "abc"));
    let mut catalog = AssetCatalog::new();
    catalog.insert_asset_record(
        AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
            .with_primary_source(source_id),
    );
    catalog.insert_source(source);

    let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

    assert!(outcome.publication.is_none());
    assert_eq!(session.import_ledger().len(), 0);
    assert!(
        outcome
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == AssetDiagnosticCode::ImportProfileRejected })
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn multiple_material_sources_without_primary_source_block_before_ledger_allocation() {
    let root = unique_temp_dir("material_source_ambiguous");
    let asset_id = asset_id(1);
    let first_source_id = asset_source_id(2);
    let second_source_id = asset_source_id(3);
    let mut session = crate::asset_pipeline::EditorAssetProjectSession::from_project_file(
        &root,
        &material_project_with_recipe("render", "render_material"),
    )
    .expect("project session should form");
    let mut catalog = AssetCatalog::new();
    catalog.insert_asset_record(AssetRecord::new(
        asset_id,
        "rock",
        "Rock",
        AssetKind::MaterialGraph,
    ));
    catalog.insert_source(
        AssetSourceDescriptor::new(
            first_source_id,
            asset_id,
            AssetKind::MaterialGraph,
            "assets/materials/rock-a.material.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc")),
    );
    catalog.insert_source(
        AssetSourceDescriptor::new(
            second_source_id,
            asset_id,
            AssetKind::MaterialGraph,
            "assets/materials/rock-b.material.ron",
        )
        .with_hash(SourceHash::new("sha256", "def")),
    );

    let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

    assert!(outcome.publication.is_none());
    assert_eq!(session.import_ledger().len(), 0);
    assert!(outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("multiple material graph sources and no primary source")
    }));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn missing_texture_resource_artifact_blocks_before_ledger_allocation() {
    let root = unique_temp_dir("material_texture_resource_missing");
    let asset_id = asset_id(1);
    let source_id = asset_source_id(2);
    let source = AssetSourceDescriptor::new(
        source_id,
        asset_id,
        AssetKind::MaterialGraph,
        "assets/materials/rock.material.ron",
    )
    .with_hash(SourceHash::new("sha256", "abc"));
    let mut session = crate::asset_pipeline::EditorAssetProjectSession::from_project_file(
        &root,
        &material_project_with_recipe("render", "render_material"),
    )
    .expect("project session should form");
    let document = texture_material_graph_document(asset_id, &source);
    write_material_graph_document(&root.join(&source.relative_path), &document)
        .expect("source document should write");
    let mut catalog = AssetCatalog::new();
    catalog.insert_asset_record(
        AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
            .with_primary_source(source_id),
    );
    catalog.insert_source(source);

    let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

    assert!(outcome.publication.is_none());
    assert_eq!(session.import_ledger().len(), 0);
    assert!(outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("references missing texture asset 'rock.albedo'")
    }));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn changed_material_recipe_allocates_new_ledger_entry() {
    let root = unique_temp_dir("material_recipe_cache_split");
    let asset_id = asset_id(1);
    let source_id = asset_source_id(2);
    let source = AssetSourceDescriptor::new(
        source_id,
        asset_id,
        AssetKind::MaterialGraph,
        "assets/materials/rock.material.ron",
    )
    .with_hash(SourceHash::new("sha256", "abc"));
    let mut catalog = AssetCatalog::new();
    catalog.insert_asset_record(
        AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
            .with_primary_source(source_id),
    );
    catalog.insert_source(source.clone());

    let mut preview_session = crate::asset_pipeline::EditorAssetProjectSession::from_project_file(
        &root,
        &material_project_with_recipe("preview", "preview"),
    )
    .expect("preview project session should form");
    let preview_document = default_material_graph_document_for_source_with_target(
        asset_id,
        &source,
        "Rock",
        MaterialOutputTarget::PbrPreview,
    );
    write_material_graph_document(&root.join(&source.relative_path), &preview_document)
        .expect("preview source document should write");
    let preview = rebuild_material_preview_for_asset(&catalog, &mut preview_session, asset_id);
    assert!(preview.diagnostics.is_empty(), "{:?}", preview.diagnostics);
    assert_eq!(preview_session.import_ledger().len(), 3);

    let mut render_session = crate::asset_pipeline::EditorAssetProjectSession::from_project_file(
        &root,
        &material_project_with_recipe("render", "render_material"),
    )
    .expect("render project session should form");
    let render_document = default_material_graph_document_for_source_with_target(
        asset_id,
        &source,
        "Rock",
        MaterialOutputTarget::RenderMaterial,
    );
    write_material_graph_document(&root.join(&source.relative_path), &render_document)
        .expect("render source document should write");
    let render = rebuild_material_preview_for_asset(&catalog, &mut render_session, asset_id);

    assert!(render.diagnostics.is_empty(), "{:?}", render.diagnostics);
    assert_eq!(render_session.import_ledger().len(), 6);
    let cache_keys = render_session
        .import_ledger()
        .entries()
        .iter()
        .map(|entry| entry.cache_key.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(cache_keys.len(), 6);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn invalid_material_graph_preserves_prior_valid_artifact() {
    let root = unique_temp_dir("material_preview_preserve");
    let asset_id = asset_id(1);
    let source_id = asset_source_id(2);
    let mut session = crate::asset_pipeline::EditorAssetProjectSession::new(
        &root,
        AssetProjectCatalogDescriptor::new(
            [AssetSourceRoot::new(
                asset_source_root_id(1),
                AssetSourceRootKind::ProjectAssets,
                "Project assets",
                "assets",
            )],
            ".runenwerk/artifacts",
            ".runenwerk/field-products",
            "assets/catalog.ron",
        ),
    );
    let source = AssetSourceDescriptor::new(
        source_id,
        asset_id,
        AssetKind::MaterialGraph,
        "assets/materials/missing.material.ron",
    )
    .with_hash(SourceHash::new("sha256", "abc"));
    let mut catalog = AssetCatalog::new();
    catalog.insert_asset_record(
        AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
            .with_primary_source(source_id),
    );
    catalog.insert_source(source);
    catalog.insert_artifact(
        AssetArtifactDescriptor::new(
            asset_artifact_id(9),
            asset_id,
            AssetKind::Material,
            ArtifactPayloadKind::FormedMaterialProduct {
                product_id: "3".to_string(),
            },
            ArtifactCacheKey::new("prior"),
        )
        .with_validity(asset::ArtifactValidity::Valid),
    );

    let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

    assert!(!outcome.diagnostics.is_empty());
    let publication = outcome.publication.expect("prior valid should preserve");
    assert_eq!(
        publication.artifact.validity,
        asset::ArtifactValidity::FailedPreserved
    );
    assert_eq!(publication.artifact.artifact_id, asset_artifact_id(9));
    let _ = std::fs::remove_dir_all(root);
}

fn material_project_with_recipe(profile_name: &str, lowering_target: &str) -> ProjectFileV3 {
    let mut project = ProjectFileV3::new("project.material", "Material");
    project
        .import_profile_definitions
        .push(ProjectImportProfileDefinitionV3::new(
            AssetKind::MaterialGraph,
            profile_name,
            ImportSettings::MaterialGraph {
                lowering_target: lowering_target.to_string(),
            },
            AssetKind::Material,
        ));
    project
        .import_profile_defaults
        .push(ProjectImportProfileDefaultV3::new(
            AssetKind::MaterialGraph,
            profile_name,
        ));
    project
}

fn command_test_document() -> MaterialGraphDocument {
    use graph::{
        CyclePolicy, GraphDefinition, GraphId, NodeDefinition, NodeId, PortDefinition,
        PortDirection, PortId,
    };
    let float = material_graph::MaterialValueType::Float.port_type_id();
    MaterialGraphDocument::new(
        material_graph::MaterialGraphDocumentId::new(77),
        "Command Test",
        GraphDefinition::new(
            GraphId::new(1),
            "material.command",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(
                    NodeId::new(1),
                    "pbr.roughness",
                    [PortDefinition::new(
                        PortId::new(1),
                        "value",
                        PortDirection::Output,
                        float,
                    )],
                ),
                NodeDefinition::new(
                    NodeId::new(2),
                    "pbr.output",
                    [PortDefinition::new(
                        PortId::new(2),
                        "roughness",
                        PortDirection::Input,
                        float,
                    )],
                ),
            ],
            [],
        ),
        MaterialOutputTarget::RenderMaterial,
    )
}

fn material_edit_app(
    root: &std::path::Path,
    asset_id: AssetId,
    source_id: asset::AssetSourceId,
    document: MaterialGraphDocument,
) -> RunenwerkEditorApp {
    let source = AssetSourceDescriptor::new(
        source_id,
        asset_id,
        AssetKind::MaterialGraph,
        "assets/materials/edit.material.ron",
    )
    .with_hash(SourceHash::new("sha256", "edit"));
    write_material_graph_document(&root.join(&source.relative_path), &document)
        .expect("material graph source should write");
    let session = crate::asset_pipeline::EditorAssetProjectSession::from_project_file(
        root,
        &material_project_with_recipe("render", "render_material"),
    )
    .expect("project session should form");
    let mut catalog = AssetCatalog::new();
    catalog.insert_asset_record(
        AssetRecord::new(asset_id, "edit", "Edit", AssetKind::MaterialGraph)
            .with_primary_source(source_id),
    );
    catalog.insert_source(source);

    let mut app = RunenwerkEditorApp::new();
    app.set_asset_project_session(session);
    app.asset_catalog_runtime_mut().replace_catalog(catalog);
    app.material_lab_runtime_mut()
        .select_material_asset(Some(asset_id));
    app.material_lab_runtime_mut()
        .set_active_source_document(asset_id, document);
    app
}

fn texture_edit_command_document() -> MaterialGraphDocument {
    use graph::{CyclePolicy, GraphDefinition, GraphId, NodeDefinition, NodeId};
    MaterialGraphDocument::new(
        material_graph::MaterialGraphDocumentId::new(78),
        "Texture Edit",
        GraphDefinition::new(
            GraphId::new(1),
            "material.texture_edit",
            CyclePolicy::RejectDirectedCycles,
            [NodeDefinition::new(NodeId::new(1), "texture.sample_2d", [])],
            [],
        ),
        MaterialOutputTarget::RenderMaterial,
    )
}

fn insert_texture_product(
    catalog: &mut AssetCatalog,
    asset_id: AssetId,
    artifact_id: asset::AssetArtifactId,
    stable_name: &str,
    display_name: &str,
    product_id: u64,
    dimension: TextureDimension,
) {
    let depth = match dimension {
        TextureDimension::Texture2D => 1,
        TextureDimension::Texture3DVolume => 2,
    };
    let descriptor = TextureDescriptor::new(
        TextureProductId::new(product_id),
        display_name,
        dimension,
        TextureExtent::new(4, 4, depth),
    );
    let asset_kind = match dimension {
        TextureDimension::Texture2D => AssetKind::Texture2D,
        TextureDimension::Texture3DVolume => AssetKind::Texture3DVolume,
    };
    catalog.insert_asset_record(AssetRecord::new(
        asset_id,
        stable_name,
        display_name,
        asset_kind,
    ));
    catalog.insert_artifact(AssetArtifactDescriptor::new(
        artifact_id,
        asset_id,
        asset_kind,
        ArtifactPayloadKind::TextureProduct {
            descriptor_hash: descriptor.descriptor_hash().to_string(),
            descriptor,
            artifact_uri: Some(format!(".runenwerk/artifacts/{stable_name}.ktx2")),
        },
        ArtifactCacheKey::new(format!("{stable_name}-cache")),
    ));
}

#[allow(clippy::too_many_arguments)]
fn insert_foreign_mesh_reference_artifact(
    catalog: &mut AssetCatalog,
    asset_id: AssetId,
    source_id: asset::AssetSourceId,
    artifact_id: asset::AssetArtifactId,
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
    let source = AssetSourceDescriptor::new(
        source_id,
        asset_id,
        AssetKind::ForeignMeshReferenceSource,
        format!("assets/models/{stable_name}.gltf"),
    )
    .with_hash(SourceHash::new("sha256", "mesh"));
    catalog.insert_source(source);
    insert_foreign_mesh_reference_artifact_descriptor(
        catalog,
        asset_id,
        source_id,
        artifact_id,
        stable_name,
        material_region_key,
        material_region_label,
    );
}

fn insert_foreign_mesh_reference_artifact_descriptor(
    catalog: &mut AssetCatalog,
    asset_id: AssetId,
    source_id: asset::AssetSourceId,
    artifact_id: asset::AssetArtifactId,
    stable_name: &str,
    material_region_key: &str,
    material_region_label: &str,
) {
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

fn texture_material_graph_document(
    asset_id: AssetId,
    source: &AssetSourceDescriptor,
) -> MaterialGraphDocument {
    use graph::{
        CyclePolicy, GraphDefinition, GraphId, GraphMetadataEntry, GraphValue, NodeDefinition,
        NodeId, PortDefinition, PortDirection, PortId, PortTypeId,
    };
    use resource_ref::ResourceRef;

    MaterialGraphDocument::new(
        material_document_id_for_source(asset_id, source.source_id),
        "Rock",
        GraphDefinition::new(
            GraphId::new(1),
            "material.texture",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(
                    NodeId::new(1),
                    "pbr.output",
                    [PortDefinition::new(
                        PortId::new(1),
                        "base_color",
                        PortDirection::Input,
                        PortTypeId::new(1),
                    )],
                ),
                NodeDefinition::new(NodeId::new(2), "texture.sample_2d", []).with_values([
                    GraphMetadataEntry::new(
                        material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
                        GraphValue::resource(
                            ResourceRef::new("asset.catalog.texture2d", "rock.albedo")
                                .expect("resource ref"),
                        ),
                    ),
                ]),
            ],
            [],
        ),
        MaterialOutputTarget::RenderMaterial,
    )
}

fn unique_temp_dir(label: &str) -> std::path::PathBuf {
    let mut root = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    root.push(format!("{label}_{nanos}"));
    std::fs::create_dir_all(&root).expect("temp dir should be creatable");
    root
}
