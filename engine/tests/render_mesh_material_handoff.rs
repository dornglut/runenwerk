use engine::plugins::render::inspect::{
    RenderMeshMaterialHandoffDiagnosticSeverity, RenderMeshMaterialHandoffInspectionRequest,
    RenderPassMaterialBindingEvidence, RenderPassModelMeshMaterialSelectionEvidence,
    inspect_render_mesh_material_handoff,
};
use engine::plugins::render::{
    PreparedMaterialBindingSlot, PreparedMaterialBindingTable, PreparedMaterialFeatureContribution,
    PreparedMaterialInstanceInput, PreparedMaterialOutputTarget, PreparedMaterialParameterInput,
    PreparedMaterialParameterKind, PreparedMaterialParameterPayloadV1,
    PreparedMaterialParameterProfile, PreparedModelMeshMaterialRegionIdentity,
    PreparedModelMeshMaterialSelection, PreparedModelMeshMaterialSourceIdentity,
    PreparedSceneMaterialBundle,
};

#[test]
fn render_mesh_material_handoff_reports_ready_source_backed_pass_chain() {
    let material = prepared_material("source_material_slot:body", false);
    let pass = material_pass_evidence(&material);

    let inspection =
        inspect_render_mesh_material_handoff(RenderMeshMaterialHandoffInspectionRequest {
            prepared_material: &material,
            material_passes: &[pass],
            require_model_mesh_selection: true,
            require_material_consuming_pass: true,
        });

    assert!(inspection.is_ready(), "{:?}", inspection.diagnostics);
    assert_eq!(inspection.counts.material_instance_count, 1);
    assert_eq!(inspection.counts.material_binding_slot_count, 1);
    assert_eq!(inspection.counts.model_mesh_selection_count, 1);
    assert_eq!(inspection.counts.material_consuming_pass_count, 1);
    assert_eq!(
        inspection.scene_shader_identity.as_deref(),
        Some("shader.identity.scene")
    );
    assert_eq!(
        inspection.material_table_identity.as_deref(),
        Some("scene.material.table:v1")
    );
}

#[test]
fn render_mesh_material_handoff_fails_without_consuming_pass_evidence() {
    let material = prepared_material("source_material_slot:body", false);

    let inspection =
        inspect_render_mesh_material_handoff(RenderMeshMaterialHandoffInspectionRequest {
            prepared_material: &material,
            material_passes: &[],
            require_model_mesh_selection: true,
            require_material_consuming_pass: true,
        });

    assert!(!inspection.is_ready());
    assert!(has_error(
        &inspection.diagnostics,
        "missing_material_consuming_pass"
    ));
}

#[test]
fn render_mesh_material_handoff_fails_on_transient_model_mesh_region() {
    let material = prepared_material("renderable_index:42", false);
    let pass = material_pass_evidence(&material);

    let inspection =
        inspect_render_mesh_material_handoff(RenderMeshMaterialHandoffInspectionRequest {
            prepared_material: &material,
            material_passes: &[pass],
            require_model_mesh_selection: true,
            require_material_consuming_pass: true,
        });

    assert!(!inspection.is_ready());
    assert!(has_error(
        &inspection.diagnostics,
        "transient_model_mesh_region"
    ));
}

#[test]
fn render_mesh_material_handoff_fails_on_pass_count_drift() {
    let material = prepared_material("source_material_slot:body", false);
    let mut pass = material_pass_evidence(&material);
    pass.prepared_model_mesh_material_selection_count = 0;
    pass.model_mesh_material_selections_available_to_pass
        .clear();

    let inspection =
        inspect_render_mesh_material_handoff(RenderMeshMaterialHandoffInspectionRequest {
            prepared_material: &material,
            material_passes: &[pass],
            require_model_mesh_selection: true,
            require_material_consuming_pass: true,
        });

    assert!(!inspection.is_ready());
    assert!(has_error(
        &inspection.diagnostics,
        "material_pass_model_mesh_count_drift"
    ));
    assert!(has_error(
        &inspection.diagnostics,
        "model_mesh_selection_not_exposed_to_pass"
    ));
}

fn prepared_material(
    region_key: &str,
    used_default_fallback: bool,
) -> PreparedMaterialFeatureContribution {
    let instance_id = "material.product.7";
    let binding_table =
        PreparedMaterialBindingTable::fixed_capacity([PreparedMaterialBindingSlot::new(
            0,
            instance_id,
            "formed.material.7",
            "shader.artifact.scene",
            "material.cache.7",
            "shader.cache.scene",
        )])
        .expect("binding table should be valid");
    let source = PreparedModelMeshMaterialSourceIdentity::new(10, 20)
        .expect("source identity should be valid")
        .with_source_revision_id(30)
        .with_source_revision("revision-30");
    let region = PreparedModelMeshMaterialRegionIdentity {
        source,
        region_key: region_key.to_string(),
    };
    let selection = PreparedModelMeshMaterialSelection::new(region, 1, 2, 0, used_default_fallback)
        .expect("selection should be valid");

    PreparedMaterialFeatureContribution {
        instances: vec![PreparedMaterialInstanceInput {
            material_instance_id: instance_id.to_string(),
            specialization_key_fragment: "specialization.scene".to_string(),
            parameter_payload: PreparedMaterialParameterPayloadV1::new(
                PreparedMaterialParameterProfile::RenderMaterial,
                PreparedMaterialOutputTarget::RenderMaterial,
                [PreparedMaterialParameterInput::new(
                    "base_color",
                    PreparedMaterialParameterKind::Vector4,
                )],
            ),
            texture_bindings: Vec::new(),
        }],
        binding_table,
        scene_bundle: Some(PreparedSceneMaterialBundle::new_with_resource_layout(
            "shader.artifact.scene",
            "shader.cache.scene",
            "generated/scene_material.wgsl",
            "shader.identity.scene",
            "scene.material.table:v1",
            "resource.layout:v1",
        )),
        model_mesh_material_selections: vec![selection],
    }
}

fn material_pass_evidence(
    material: &PreparedMaterialFeatureContribution,
) -> RenderPassMaterialBindingEvidence {
    let selection = &material.model_mesh_material_selections[0];
    RenderPassMaterialBindingEvidence {
        consumes_material_resources: true,
        prepared_material_available: true,
        material_table_identity: material
            .scene_bundle
            .as_ref()
            .map(|bundle| bundle.material_table_identity.clone()),
        scene_shader_identity: material
            .scene_bundle
            .as_ref()
            .map(|bundle| bundle.shader_identity.clone()),
        scene_shader_path: material
            .scene_bundle
            .as_ref()
            .map(|bundle| bundle.shader_path.clone()),
        material_instance_count: material.instances.len(),
        material_binding_slot_count: material.binding_table.slots.len(),
        prepared_model_mesh_material_selection_count: material.model_mesh_material_selections.len(),
        model_mesh_material_selections_available_to_pass: vec![
            RenderPassModelMeshMaterialSelectionEvidence {
                source_asset_id: selection.surface.source.asset_id,
                source_id: selection.surface.source.source_id,
                source_revision_id: selection.surface.source.source_revision_id,
                source_revision: selection.surface.source.source_revision.clone(),
                region_key: selection.surface.region_key.clone(),
                requested_material_slot_id: selection.requested_material_slot_id,
                resolved_material_slot_id: selection.resolved_material_slot_id,
                material_table_index: selection.material_table_index,
                used_default_fallback: selection.used_default_fallback,
            },
        ],
    }
}

fn has_error(
    diagnostics: &[engine::plugins::render::inspect::RenderMeshMaterialHandoffDiagnostic],
    code: &str,
) -> bool {
    diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == RenderMeshMaterialHandoffDiagnosticSeverity::Error
            && diagnostic.code == code
    })
}
