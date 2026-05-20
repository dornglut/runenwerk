use std::collections::{BTreeMap, BTreeSet};

use asset::{AssetDiagnosticCode, AssetDiagnosticRecord};
use editor_scene::{SceneMaterialAssignmentState, SceneMaterialSlot, SceneMaterialSlotId};
use engine::plugins::render::{
    FeatureContributionStatus, FeatureFallbackPolicy, PreparedMaterialBindingSlot,
    PreparedMaterialBindingTable, PreparedMaterialFeatureContribution,
    PreparedMaterialFeatureResource, PreparedMaterialInstanceInput, PreparedMaterialOutputTarget,
    PreparedMaterialParameterInput, PreparedMaterialParameterKind,
    PreparedMaterialParameterPayloadV1, PreparedMaterialParameterProfile,
    PreparedMaterialTextureBinding, PreparedMaterialTextureKind, PreparedSceneMaterialBundle,
};
use material_graph::{MaterialOutputTarget, MaterialParameterKind};

use crate::material_lab::{
    EditorMaterialPreviewProduct, EditorSceneMaterialTableShaderBundle,
    MaterialRendererParameterProfile, PreviewSceneMaterialSlot, PreviewSceneProduct,
    PreviewSceneProductBuildOutcome, PreviewSceneProductDiagnostic, PreviewSceneProductMode,
    PreviewSceneProductRequestIdentity, PreviewSceneResourceSlot, PreviewSceneResourceSlotMapping,
    PreviewSceneShaderProductRef,
};

const SINGLE_MATERIAL_PREVIEW_SCENE_SLOT_ID: &str = "single-material.active";
const SINGLE_MATERIAL_RESOURCE_LAYOUT_IDENTITY: &str = "";

#[derive(Debug, Clone, Copy)]
pub struct SceneMaterialSlotProduct<'a> {
    pub slot_id: SceneMaterialSlotId,
    pub preview: &'a EditorMaterialPreviewProduct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneMaterialTableShaderBuildRequest<'a> {
    pub compile_request: engine::plugins::render::SceneMaterialTableCompileRequest<'a>,
    pub material_table_identity: String,
    pub resource_layout_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneMaterialTableBundleExpectation {
    pub material_table_identity: String,
    pub resource_layout_identity: String,
}

pub fn build_preview_scene_product_for_single_material(
    preview: &EditorMaterialPreviewProduct,
) -> PreviewSceneProductBuildOutcome {
    match single_material_preview_scene_product(preview) {
        Ok(product) => PreviewSceneProductBuildOutcome::ready(product),
        Err(diagnostic) => PreviewSceneProductBuildOutcome::failed_closed([diagnostic]),
    }
}

pub fn build_preview_scene_product_for_scene_material_table(
    preview: &EditorMaterialPreviewProduct,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    slot_products: &[SceneMaterialSlotProduct<'_>],
    scene_table_bundle: Option<&EditorSceneMaterialTableShaderBundle>,
) -> PreviewSceneProductBuildOutcome {
    let Some(assignments) = scene_material_assignments else {
        return build_preview_scene_product_for_single_material(preview);
    };
    let slots = match resolved_scene_material_slots(preview, Some(assignments), slot_products) {
        Ok(slots) => slots,
        Err(diagnostic) => {
            return PreviewSceneProductBuildOutcome::failed_closed([
                preview_scene_product_diagnostic_from_asset(
                    "material.preview_scene.unresolved_scene_slot",
                    diagnostic,
                ),
            ]);
        }
    };
    let resource_layout = resolved_scene_material_table_resource_layout(&slots);
    let material_table_identity = resolved_scene_material_table_identity(
        Some(assignments),
        &slots,
        resource_layout.identity.as_str(),
    );
    let request_identity = preview_scene_product_request_identity_from_resolved_slots(
        preview,
        &slots,
        material_table_identity.as_str(),
        resource_layout.identity.as_str(),
        &resource_layout,
        scene_table_bundle,
    );

    match build_preview_scene_product_from_resolved_slots(
        preview,
        &slots,
        material_table_identity,
        resource_layout.identity.clone(),
        &resource_layout,
        scene_table_bundle,
    ) {
        Ok(product) => PreviewSceneProductBuildOutcome::ready(product),
        Err(diagnostic) => PreviewSceneProductBuildOutcome::failed_closed_for_request(
            request_identity,
            [diagnostic],
        ),
    }
}

pub fn prepared_material_contribution_for_preview(
    preview: &EditorMaterialPreviewProduct,
) -> PreparedMaterialFeatureContribution {
    prepared_material_contribution_for_preview_with_scene_materials(preview, None)
}

pub fn prepared_material_contribution_for_preview_with_scene_materials(
    preview: &EditorMaterialPreviewProduct,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
) -> PreparedMaterialFeatureContribution {
    PreparedMaterialFeatureContribution {
        instances: vec![PreparedMaterialInstanceInput {
            material_instance_id: format!("material.product.{}", preview.product.product_id.raw()),
            specialization_key_fragment: preview.product.specialization_fragment.0.clone(),
            parameter_payload: material_parameter_payload(preview),
            texture_bindings: prepared_texture_bindings(preview),
        }],
        binding_table: scene_material_binding_table(preview, scene_material_assignments),
        scene_bundle: Some(PreparedSceneMaterialBundle::new(
            preview.scene_shader_artifact_id.raw().to_string(),
            preview.scene_shader_cache_key.as_str().to_string(),
            preview.scene_shader_path.clone(),
            preview.scene_shader_identity.clone(),
            scene_material_table_identity(preview, scene_material_assignments),
        )),
        model_mesh_material_selections: Vec::new(),
    }
}

pub fn prepared_material_resource_for_preview_with_resolved_scene_materials(
    preview: Option<&EditorMaterialPreviewProduct>,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    slot_products: &[SceneMaterialSlotProduct<'_>],
) -> Result<PreparedMaterialFeatureResource, AssetDiagnosticRecord> {
    prepared_material_resource_for_preview_with_resolved_scene_materials_and_bundle(
        preview,
        scene_material_assignments,
        slot_products,
        None,
    )
}

pub fn prepared_material_resource_for_preview_with_resolved_scene_materials_and_bundle(
    preview: Option<&EditorMaterialPreviewProduct>,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    slot_products: &[SceneMaterialSlotProduct<'_>],
    scene_table_bundle: Option<&EditorSceneMaterialTableShaderBundle>,
) -> Result<PreparedMaterialFeatureResource, AssetDiagnosticRecord> {
    match preview {
        Some(preview) => {
            let payload =
                prepared_material_contribution_for_preview_with_resolved_scene_materials_and_bundle(
                    preview,
                    scene_material_assignments,
                    slot_products,
                    scene_table_bundle,
                )?;
            payload.validate_portable_limits().map_err(|error| {
                AssetDiagnosticRecord::error(
                    AssetDiagnosticCode::RatificationRejected,
                    format!(
                        "material renderer handoff rejected portable binding limits: {}",
                        error
                    ),
                )
            })?;
            Ok(PreparedMaterialFeatureResource {
                status: FeatureContributionStatus::Ready,
                fallback_policy: FeatureFallbackPolicy::ReuseLastGood,
                payload,
            })
        }
        None => prepared_material_resource_for_preview(None),
    }
}

pub fn prepared_material_resource_for_preview_scene_product(
    product: &PreviewSceneProduct,
    preview: &EditorMaterialPreviewProduct,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    slot_products: &[SceneMaterialSlotProduct<'_>],
    scene_table_bundle: Option<&EditorSceneMaterialTableShaderBundle>,
) -> Result<PreparedMaterialFeatureResource, AssetDiagnosticRecord> {
    let payload = prepared_material_contribution_for_preview_scene_product(
        product,
        preview,
        scene_material_assignments,
        slot_products,
        scene_table_bundle,
    )?;
    payload.validate_portable_limits().map_err(|error| {
        AssetDiagnosticRecord::error(
            AssetDiagnosticCode::RatificationRejected,
            format!(
                "material renderer handoff rejected portable binding limits: {}",
                error
            ),
        )
    })?;
    Ok(PreparedMaterialFeatureResource {
        status: FeatureContributionStatus::Ready,
        fallback_policy: FeatureFallbackPolicy::ReuseLastGood,
        payload,
    })
}

pub fn prepared_material_contribution_for_preview_scene_product(
    product: &PreviewSceneProduct,
    preview: &EditorMaterialPreviewProduct,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    slot_products: &[SceneMaterialSlotProduct<'_>],
    scene_table_bundle: Option<&EditorSceneMaterialTableShaderBundle>,
) -> Result<PreparedMaterialFeatureContribution, AssetDiagnosticRecord> {
    if scene_material_assignments.is_none() {
        let expected = single_material_preview_scene_product(preview)
            .map_err(asset_diagnostic_from_preview_scene_product_diagnostic)?;
        if expected.product_identity != product.product_identity {
            return Err(stale_preview_scene_product_asset_diagnostic());
        }
        return Ok(
            prepared_material_contribution_from_single_preview_scene_product(product, preview),
        );
    }

    let slots = resolved_scene_material_slots(preview, scene_material_assignments, slot_products)?;
    let resource_layout = resolved_scene_material_table_resource_layout(&slots);
    let material_table_identity = resolved_scene_material_table_identity(
        scene_material_assignments,
        &slots,
        resource_layout.identity.as_str(),
    );
    let expected = build_preview_scene_product_from_resolved_slots(
        preview,
        &slots,
        material_table_identity,
        resource_layout.identity.clone(),
        &resource_layout,
        scene_table_bundle,
    )
    .map_err(asset_diagnostic_from_preview_scene_product_diagnostic)?;
    if expected.product_identity != product.product_identity {
        return Err(stale_preview_scene_product_asset_diagnostic());
    }

    Ok(
        prepared_material_contribution_from_resolved_preview_scene_product(
            product,
            &slots,
            &resource_layout,
        ),
    )
}

pub fn prepared_material_contribution_for_preview_with_resolved_scene_materials(
    preview: &EditorMaterialPreviewProduct,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    slot_products: &[SceneMaterialSlotProduct<'_>],
) -> Result<PreparedMaterialFeatureContribution, AssetDiagnosticRecord> {
    prepared_material_contribution_for_preview_with_resolved_scene_materials_and_bundle(
        preview,
        scene_material_assignments,
        slot_products,
        None,
    )
}

pub fn prepared_material_contribution_for_preview_with_resolved_scene_materials_and_bundle(
    preview: &EditorMaterialPreviewProduct,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    slot_products: &[SceneMaterialSlotProduct<'_>],
    scene_table_bundle: Option<&EditorSceneMaterialTableShaderBundle>,
) -> Result<PreparedMaterialFeatureContribution, AssetDiagnosticRecord> {
    if scene_material_assignments.is_none() {
        let product = single_material_preview_scene_product(preview)
            .map_err(asset_diagnostic_from_preview_scene_product_diagnostic)?;
        return Ok(
            prepared_material_contribution_from_single_preview_scene_product(&product, preview),
        );
    }
    let slots = resolved_scene_material_slots(preview, scene_material_assignments, slot_products)?;
    let resource_layout = resolved_scene_material_table_resource_layout(&slots);
    let material_table_identity = resolved_scene_material_table_identity(
        scene_material_assignments,
        &slots,
        resource_layout.identity.as_str(),
    );
    let product = build_preview_scene_product_from_resolved_slots(
        preview,
        &slots,
        material_table_identity,
        resource_layout.identity.clone(),
        &resource_layout,
        scene_table_bundle,
    )
    .map_err(asset_diagnostic_from_preview_scene_product_diagnostic)?;

    Ok(
        prepared_material_contribution_from_resolved_preview_scene_product(
            &product,
            &slots,
            &resource_layout,
        ),
    )
}

fn prepared_material_contribution_from_single_preview_scene_product(
    product: &PreviewSceneProduct,
    preview: &EditorMaterialPreviewProduct,
) -> PreparedMaterialFeatureContribution {
    PreparedMaterialFeatureContribution {
        instances: vec![PreparedMaterialInstanceInput {
            material_instance_id: format!("material.product.{}", preview.product.product_id.raw()),
            specialization_key_fragment: preview.product.specialization_fragment.0.clone(),
            parameter_payload: material_parameter_payload(preview),
            texture_bindings: prepared_texture_bindings(preview),
        }],
        binding_table: PreparedMaterialBindingTable::fixed_capacity([
            preview_material_binding_slot(preview, 0),
        ])
        .expect("single material preview uses one portable material binding slot"),
        scene_bundle: Some(scene_material_bundle_for_preview_scene_product(product)),
        model_mesh_material_selections: Vec::new(),
    }
}

fn prepared_material_contribution_from_resolved_preview_scene_product(
    product: &PreviewSceneProduct,
    slots: &[ResolvedSceneMaterialSlot<'_>],
    resource_layout: &ResolvedSceneMaterialTableResourceLayout,
) -> PreparedMaterialFeatureContribution {
    let mut instances = Vec::new();
    let mut seen_instances = BTreeSet::new();
    for resolved in slots {
        let material_instance_id = material_instance_id_for_slot(resolved.slot, resolved.preview);
        if seen_instances.insert(material_instance_id.clone()) {
            instances.push(PreparedMaterialInstanceInput {
                material_instance_id: material_instance_id.clone(),
                specialization_key_fragment: resolved
                    .preview
                    .product
                    .specialization_fragment
                    .0
                    .clone(),
                parameter_payload: material_parameter_payload(resolved.preview),
                texture_bindings: resource_layout
                    .bindings_for_instance(&material_instance_id)
                    .to_vec(),
            });
        }
    }
    let binding_slots = slots
        .iter()
        .map(|resolved| {
            scene_material_binding_slot_for_preview(
                resolved.preview,
                resolved.material_table_index,
                resolved.slot,
            )
        })
        .collect::<Vec<_>>();
    PreparedMaterialFeatureContribution {
        instances,
        binding_table: PreparedMaterialBindingTable::fixed_capacity(binding_slots)
            .expect("editor_scene palette enforces portable material binding slot limits"),
        scene_bundle: Some(scene_material_bundle_for_preview_scene_product(product)),
        model_mesh_material_selections: Vec::new(),
    }
}

pub fn scene_material_table_shader_build_request_for_preview<'a>(
    preview: &'a EditorMaterialPreviewProduct,
    scene_material_assignments: Option<&'a SceneMaterialAssignmentState>,
    slot_products: &[SceneMaterialSlotProduct<'a>],
) -> Result<Option<SceneMaterialTableShaderBuildRequest<'a>>, AssetDiagnosticRecord> {
    let slots = resolved_scene_material_slots(preview, scene_material_assignments, slot_products)?;
    if !requires_generated_scene_material_table_shader(&slots) {
        return Ok(None);
    }
    let expectation = scene_material_table_bundle_expectation(scene_material_assignments, &slots);
    let mut compile_slots = Vec::with_capacity(slots.len());
    for resolved in &slots {
        let Some(ir) = resolved.preview.product.executable_ir.as_ref() else {
            return Err(AssetDiagnosticRecord::error(
                AssetDiagnosticCode::RatificationRejected,
                format!(
                    "scene material slot {} has no executable material IR for scene table shader generation",
                    resolved.slot.slot_id.raw()
                ),
            ));
        };
        compile_slots.push(engine::plugins::render::SceneMaterialTableSlot {
            slot_index: resolved.material_table_index,
            material_instance_id: material_instance_id_for_slot(resolved.slot, resolved.preview),
            ir,
        });
    }
    Ok(Some(SceneMaterialTableShaderBuildRequest {
        compile_request: engine::plugins::render::SceneMaterialTableCompileRequest {
            slots: compile_slots,
        },
        material_table_identity: expectation.material_table_identity,
        resource_layout_identity: expectation.resource_layout_identity,
    }))
}

pub fn scene_material_table_bundle_expectation_for_preview(
    preview: &EditorMaterialPreviewProduct,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    slot_products: &[SceneMaterialSlotProduct<'_>],
) -> Result<Option<SceneMaterialTableBundleExpectation>, AssetDiagnosticRecord> {
    let slots = resolved_scene_material_slots(preview, scene_material_assignments, slot_products)?;
    if requires_generated_scene_material_table_shader(&slots) {
        Ok(Some(scene_material_table_bundle_expectation(
            scene_material_assignments,
            &slots,
        )))
    } else {
        Ok(None)
    }
}

struct ResolvedSceneMaterialSlot<'a> {
    material_table_index: u32,
    slot: &'a SceneMaterialSlot,
    preview: &'a EditorMaterialPreviewProduct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedSceneMaterialTableResourceLayout {
    identity: String,
    resources: Vec<PreviewSceneResourceSlot>,
    mappings_by_material_table_slot: BTreeMap<u32, Vec<PreviewSceneResourceSlotMapping>>,
    bindings_by_instance: BTreeMap<String, Vec<PreparedMaterialTextureBinding>>,
}

impl ResolvedSceneMaterialTableResourceLayout {
    fn bindings_for_instance(
        &self,
        material_instance_id: &str,
    ) -> &[PreparedMaterialTextureBinding] {
        self.bindings_by_instance
            .get(material_instance_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn resource_mappings_for_material_slot(
        &self,
        material_table_index: u32,
    ) -> &[PreviewSceneResourceSlotMapping] {
        self.mappings_by_material_table_slot
            .get(&material_table_index)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

fn resolved_scene_material_slots<'a>(
    default_preview: &'a EditorMaterialPreviewProduct,
    scene_material_assignments: Option<&'a SceneMaterialAssignmentState>,
    slot_products: &[SceneMaterialSlotProduct<'a>],
) -> Result<Vec<ResolvedSceneMaterialSlot<'a>>, AssetDiagnosticRecord> {
    let Some(assignments) = scene_material_assignments else {
        return Ok(Vec::new());
    };
    assignments
        .palette()
        .slots
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            let preview = slot_products
                .iter()
                .find(|product| product.slot_id == slot.slot_id)
                .map(|product| product.preview)
                .or_else(|| default_preview_for_unassigned_default_slot(slot, default_preview))
                .ok_or_else(|| {
                    AssetDiagnosticRecord::error(
                        AssetDiagnosticCode::RatificationRejected,
                        format!(
                            "scene material slot {} has no resolved source-backed material product",
                            slot.slot_id.raw()
                        ),
                    )
                })?;
            Ok(ResolvedSceneMaterialSlot {
                material_table_index: index as u32,
                slot,
                preview,
            })
        })
        .collect()
}

fn default_preview_for_unassigned_default_slot<'a>(
    slot: &SceneMaterialSlot,
    default_preview: &'a EditorMaterialPreviewProduct,
) -> Option<&'a EditorMaterialPreviewProduct> {
    if !slot.is_default {
        return None;
    }
    match (slot.material_asset_id, slot.source_ref.as_ref()) {
        (None, None) => Some(default_preview),
        (Some(asset_id), _) if asset_id == default_preview.asset_id => Some(default_preview),
        (_, Some(source_ref))
            if source_ref.asset_id == default_preview.asset_id
                && source_ref.source_id == default_preview.source_id =>
        {
            Some(default_preview)
        }
        _ => None,
    }
}

fn requires_generated_scene_material_table_shader(slots: &[ResolvedSceneMaterialSlot<'_>]) -> bool {
    slots.len() > 1
}

fn scene_material_table_bundle_expectation(
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    slots: &[ResolvedSceneMaterialSlot<'_>],
) -> SceneMaterialTableBundleExpectation {
    let resource_layout = resolved_scene_material_table_resource_layout(slots);
    let material_table_identity = resolved_scene_material_table_identity(
        scene_material_assignments,
        slots,
        resource_layout.identity.as_str(),
    );
    SceneMaterialTableBundleExpectation {
        material_table_identity,
        resource_layout_identity: resource_layout.identity,
    }
}

fn resolved_scene_material_table_resource_layout(
    slots: &[ResolvedSceneMaterialSlot<'_>],
) -> ResolvedSceneMaterialTableResourceLayout {
    let mut resource_slots = BTreeMap::<String, u32>::new();
    let mut resources = Vec::<PreviewSceneResourceSlot>::new();
    let mut layout_entries = Vec::<String>::new();
    let mut slot_mappings = Vec::<String>::new();
    let mut mappings_by_material_table_slot =
        BTreeMap::<u32, Vec<PreviewSceneResourceSlotMapping>>::new();
    let mut bindings_by_instance = BTreeMap::<String, Vec<PreparedMaterialTextureBinding>>::new();
    for slot in slots {
        let material_instance_id = material_instance_id_for_slot(slot.slot, slot.preview);
        for (local_resource_slot, resource) in slot.preview.resolved_resources.iter().enumerate() {
            let resource_identity = strict_resolved_resource_identity(resource);
            let resource_slot_index = match resource_slots.get(&resource_identity) {
                Some(index) => *index,
                None => {
                    let index = resource_slots.len() as u32;
                    resource_slots.insert(resource_identity.clone(), index);
                    layout_entries.push(format!(
                        "resource_slot={index}:identity={resource_identity}"
                    ));
                    resources.push(preview_scene_resource_slot_for_resource(
                        index,
                        resource,
                        resource_identity.clone(),
                    ));
                    bindings_by_instance
                        .entry(material_instance_id.clone())
                        .or_default()
                        .push(prepared_texture_binding_for_resource(resource, index));
                    index
                }
            };
            mappings_by_material_table_slot
                .entry(slot.material_table_index)
                .or_default()
                .push(PreviewSceneResourceSlotMapping::new(
                    local_resource_slot as u32,
                    resource_slot_index,
                ));
            slot_mappings.push(format!(
                "slot={}:instance={}:local_resource_slot={}:node={}:binding={}:resource_slot={}:identity={}",
                slot.material_table_index,
                material_instance_id,
                local_resource_slot,
                resource.node_id.raw(),
                resource.binding_key,
                resource_slot_index,
                resource_identity
            ));
        }
    }
    for bindings in bindings_by_instance.values_mut() {
        bindings.sort_by_key(|binding| binding.resource_slot_index);
    }
    ResolvedSceneMaterialTableResourceLayout {
        identity: canonical_identity(
            "scene-material-table-resource-layout-v1",
            layout_entries.into_iter().chain(slot_mappings),
        ),
        resources,
        mappings_by_material_table_slot,
        bindings_by_instance,
    }
}

fn single_material_preview_scene_product(
    preview: &EditorMaterialPreviewProduct,
) -> Result<PreviewSceneProduct, PreviewSceneProductDiagnostic> {
    let material_table_identity = scene_material_table_identity(preview, None);
    let shader = active_preview_scene_shader_ref(
        preview,
        material_table_identity.as_str(),
        SINGLE_MATERIAL_RESOURCE_LAYOUT_IDENTITY,
    );
    let resources = preview_scene_resource_slots_for_single_material(preview);
    let resource_mappings = resources
        .iter()
        .map(|resource| {
            PreviewSceneResourceSlotMapping::new(
                resource.table_resource_slot,
                resource.table_resource_slot,
            )
        })
        .collect::<Vec<_>>();
    let product = PreviewSceneProduct::new(
        PreviewSceneProductMode::SingleMaterial,
        preview.viewport_product_id,
        preview.product.product_id,
        preview.artifact_cache_key.clone(),
        material_table_identity,
        SINGLE_MATERIAL_RESOURCE_LAYOUT_IDENTITY,
        shader,
        [PreviewSceneMaterialSlot::new(
            0,
            SINGLE_MATERIAL_PREVIEW_SCENE_SLOT_ID,
            preview.product.product_id,
            preview.artifact_cache_key.clone(),
            preview.scene_shader_identity.clone(),
            resource_mappings,
        )],
        resources,
    );
    validate_preview_scene_product(&product)?;
    Ok(product)
}

fn preview_scene_product_request_identity_from_resolved_slots(
    active_preview: &EditorMaterialPreviewProduct,
    slots: &[ResolvedSceneMaterialSlot<'_>],
    material_table_identity: &str,
    resource_layout_identity: &str,
    resource_layout: &ResolvedSceneMaterialTableResourceLayout,
    scene_table_bundle: Option<&EditorSceneMaterialTableShaderBundle>,
) -> PreviewSceneProductRequestIdentity {
    let requires_generated_bundle = requires_generated_scene_material_table_shader(slots);
    let mode = if requires_generated_bundle {
        PreviewSceneProductMode::SceneMaterialTable
    } else {
        PreviewSceneProductMode::SingleMaterial
    };
    let shader = if requires_generated_bundle {
        scene_table_bundle
            .filter(|bundle| {
                bundle.matches_scene_table(material_table_identity, resource_layout_identity)
            })
            .map(|bundle| {
                generated_scene_table_shader_ref(
                    bundle,
                    material_table_identity,
                    resource_layout_identity,
                )
            })
    } else {
        Some(active_preview_scene_shader_ref(
            active_preview,
            material_table_identity,
            resource_layout_identity,
        ))
    };
    preview_scene_product_request_identity_from_resolved_slots_and_shader(
        active_preview,
        slots,
        material_table_identity,
        resource_layout_identity,
        resource_layout,
        mode,
        shader.as_ref(),
    )
}

#[allow(clippy::too_many_arguments)]
fn preview_scene_product_request_identity_from_resolved_slots_and_shader(
    active_preview: &EditorMaterialPreviewProduct,
    slots: &[ResolvedSceneMaterialSlot<'_>],
    material_table_identity: &str,
    resource_layout_identity: &str,
    resource_layout: &ResolvedSceneMaterialTableResourceLayout,
    mode: PreviewSceneProductMode,
    shader: Option<&PreviewSceneShaderProductRef>,
) -> PreviewSceneProductRequestIdentity {
    let product_slots = preview_scene_product_slots_for_resolved_slots(slots, resource_layout);
    PreviewSceneProductRequestIdentity::new(
        mode,
        active_preview.viewport_product_id,
        active_preview.product.product_id,
        &active_preview.artifact_cache_key,
        material_table_identity,
        resource_layout_identity,
        shader,
        &product_slots,
        &resource_layout.resources,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_preview_scene_product_from_resolved_slots(
    active_preview: &EditorMaterialPreviewProduct,
    slots: &[ResolvedSceneMaterialSlot<'_>],
    material_table_identity: String,
    resource_layout_identity: String,
    resource_layout: &ResolvedSceneMaterialTableResourceLayout,
    scene_table_bundle: Option<&EditorSceneMaterialTableShaderBundle>,
) -> Result<PreviewSceneProduct, PreviewSceneProductDiagnostic> {
    if slots.is_empty() {
        return Err(PreviewSceneProductDiagnostic::new(
            "material.preview_scene.empty_scene_table",
            "scene material table resolved no material slots for preview scene product build",
        ));
    }

    let requires_generated_bundle = requires_generated_scene_material_table_shader(slots);
    let (mode, shader) = if requires_generated_bundle {
        let bundle = scene_table_bundle.ok_or_else(|| {
            PreviewSceneProductDiagnostic::new(
                "material.preview_scene.generated_bundle_missing",
                "scene material table preview scene product requires a generated shader bundle",
            )
        })?;
        if !bundle.matches_scene_table(&material_table_identity, &resource_layout_identity) {
            return Err(PreviewSceneProductDiagnostic::new(
                "material.preview_scene.generated_bundle_stale",
                "scene material table generated shader bundle is stale for the current material table/resource layout",
            ));
        }
        (
            PreviewSceneProductMode::SceneMaterialTable,
            generated_scene_table_shader_ref(
                bundle,
                material_table_identity.as_str(),
                resource_layout_identity.as_str(),
            ),
        )
    } else {
        (
            PreviewSceneProductMode::SingleMaterial,
            active_preview_scene_shader_ref(
                active_preview,
                material_table_identity.as_str(),
                resource_layout_identity.as_str(),
            ),
        )
    };

    let product_slots = preview_scene_product_slots_for_resolved_slots(slots, resource_layout);
    let product = PreviewSceneProduct::new(
        mode,
        active_preview.viewport_product_id,
        active_preview.product.product_id,
        active_preview.artifact_cache_key.clone(),
        material_table_identity,
        resource_layout_identity,
        shader,
        product_slots,
        resource_layout.resources.clone(),
    );
    validate_preview_scene_product(&product)?;
    Ok(product)
}

fn preview_scene_product_slots_for_resolved_slots(
    slots: &[ResolvedSceneMaterialSlot<'_>],
    resource_layout: &ResolvedSceneMaterialTableResourceLayout,
) -> Vec<PreviewSceneMaterialSlot> {
    slots
        .iter()
        .map(|slot| {
            PreviewSceneMaterialSlot::new(
                slot.material_table_index,
                slot.slot.slot_id.raw().to_string(),
                slot.preview.product.product_id,
                slot.preview.artifact_cache_key.clone(),
                slot.preview.scene_shader_identity.clone(),
                resource_layout
                    .resource_mappings_for_material_slot(slot.material_table_index)
                    .iter()
                    .cloned(),
            )
        })
        .collect()
}

fn active_preview_scene_shader_ref(
    preview: &EditorMaterialPreviewProduct,
    material_table_identity: &str,
    resource_layout_identity: &str,
) -> PreviewSceneShaderProductRef {
    PreviewSceneShaderProductRef::new(
        preview.scene_shader_artifact_id.raw().to_string(),
        preview.scene_shader_cache_key.clone(),
        preview.scene_shader_identity.clone(),
        preview.scene_shader_path.clone(),
        material_table_identity.to_string(),
        resource_layout_identity.to_string(),
    )
}

fn generated_scene_table_shader_ref(
    bundle: &EditorSceneMaterialTableShaderBundle,
    material_table_identity: &str,
    resource_layout_identity: &str,
) -> PreviewSceneShaderProductRef {
    PreviewSceneShaderProductRef::new(
        bundle.shader_artifact_id.clone(),
        bundle.shader_cache_key.clone(),
        bundle.shader_identity.clone(),
        bundle.shader_path.clone(),
        material_table_identity.to_string(),
        resource_layout_identity.to_string(),
    )
}

fn scene_material_bundle_for_preview_scene_product(
    product: &PreviewSceneProduct,
) -> PreparedSceneMaterialBundle {
    PreparedSceneMaterialBundle::new_with_resource_layout(
        product.shader.shader_artifact_id.clone(),
        product.shader.shader_cache_key.as_str().to_string(),
        product.shader.shader_path.clone(),
        product.shader.shader_identity.clone(),
        product.material_table_identity.clone(),
        product.resource_layout_identity.clone(),
    )
}

fn preview_scene_resource_slots_for_single_material(
    preview: &EditorMaterialPreviewProduct,
) -> Vec<PreviewSceneResourceSlot> {
    preview
        .resolved_resources
        .iter()
        .enumerate()
        .map(|(index, resource)| {
            preview_scene_resource_slot_for_resource(
                index as u32,
                resource,
                strict_resolved_resource_identity(resource),
            )
        })
        .collect()
}

fn preview_scene_resource_slot_for_resource(
    table_resource_slot: u32,
    resource: &crate::material_lab::ResolvedMaterialResource,
    resource_product_identity: String,
) -> PreviewSceneResourceSlot {
    PreviewSceneResourceSlot::new(
        table_resource_slot,
        resource_product_identity,
        format!("{:?}", resource.kind),
        resource.dimension.clone(),
        format!(
            "color_space={}:pixel_format={:?}:supercompression={:?}:container_byte_length={:?}",
            resource.color_space,
            resource.descriptor.ktx2_metadata().pixel_format,
            resource.descriptor.ktx2_metadata().supercompression,
            resource.descriptor.ktx2_metadata().byte_length
        ),
        resource.sampler_policy.clone(),
        resource.artifact_id.raw().to_string(),
        resource.cache_key.clone(),
    )
}

fn validate_preview_scene_product(
    product: &PreviewSceneProduct,
) -> Result<(), PreviewSceneProductDiagnostic> {
    if product.shader.material_table_identity != product.material_table_identity {
        return Err(PreviewSceneProductDiagnostic::new(
            "material.preview_scene.shader_material_table_identity_mismatch",
            "preview scene product shader material table identity does not match the product material table identity",
        ));
    }
    if product.shader.resource_layout_identity != product.resource_layout_identity {
        return Err(PreviewSceneProductDiagnostic::new(
            "material.preview_scene.shader_resource_layout_identity_mismatch",
            "preview scene product shader resource layout identity does not match the product resource layout identity",
        ));
    }

    let mut resource_slots = BTreeMap::<u32, &PreviewSceneResourceSlot>::new();
    for resource in &product.resources {
        if let Some(existing) = resource_slots.insert(resource.table_resource_slot, resource) {
            if existing.resource_product_identity != resource.resource_product_identity
                || existing.artifact_identity != resource.artifact_identity
                || existing.artifact_cache_key != resource.artifact_cache_key
            {
                return Err(PreviewSceneProductDiagnostic::new(
                    "material.preview_scene.resource_slot_identity_conflict",
                    format!(
                        "table resource slot {} has conflicting resource identities",
                        resource.table_resource_slot
                    ),
                ));
            }
            return Err(PreviewSceneProductDiagnostic::new(
                "material.preview_scene.resource_slot_duplicate",
                format!(
                    "table resource slot {} is duplicated in preview scene product",
                    resource.table_resource_slot
                ),
            ));
        }
    }

    let mut material_slots = BTreeSet::<u32>::new();
    for slot in &product.slots {
        if !material_slots.insert(slot.material_slot_index) {
            return Err(PreviewSceneProductDiagnostic::new(
                "material.preview_scene.material_slot_duplicate",
                format!(
                    "material slot {} is duplicated in preview scene product",
                    slot.material_slot_index
                ),
            ));
        }
        let mut local_resource_slots = BTreeSet::<u32>::new();
        for mapping in &slot.resource_slot_mappings {
            if !local_resource_slots.insert(mapping.local_resource_slot) {
                return Err(PreviewSceneProductDiagnostic::new(
                    "material.preview_scene.local_resource_slot_duplicate",
                    format!(
                        "material slot {} has duplicate local resource slot {}",
                        slot.material_slot_index, mapping.local_resource_slot
                    ),
                ));
            }
            if !resource_slots.contains_key(&mapping.table_resource_slot) {
                return Err(PreviewSceneProductDiagnostic::new(
                    "material.preview_scene.resource_mapping_missing_table_slot",
                    format!(
                        "material slot {} maps local resource slot {} to missing table resource slot {}",
                        slot.material_slot_index,
                        mapping.local_resource_slot,
                        mapping.table_resource_slot
                    ),
                ));
            }
        }
    }

    Ok(())
}

fn asset_diagnostic_from_preview_scene_product_diagnostic(
    diagnostic: PreviewSceneProductDiagnostic,
) -> AssetDiagnosticRecord {
    AssetDiagnosticRecord::error(
        AssetDiagnosticCode::RatificationRejected,
        diagnostic.message,
    )
}

fn stale_preview_scene_product_asset_diagnostic() -> AssetDiagnosticRecord {
    AssetDiagnosticRecord::error(
        AssetDiagnosticCode::RatificationRejected,
        "current preview scene product is stale for the active material preview request",
    )
}

fn preview_scene_product_diagnostic_from_asset(
    code: impl Into<String>,
    diagnostic: AssetDiagnosticRecord,
) -> PreviewSceneProductDiagnostic {
    PreviewSceneProductDiagnostic::new(code, diagnostic.message)
}

fn strict_resolved_resource_identity(
    resource: &crate::material_lab::ResolvedMaterialResource,
) -> String {
    canonical_identity(
        "resolved-material-resource-v1",
        [
            format!("product={}", resource.descriptor.product_id.raw()),
            format!("reference={}", resource.reference.canonical_component()),
            format!("kind={:?}", resource.kind),
            format!("dimension={}", resource.dimension),
            format!("texture_dimension={:?}", resource.descriptor.dimension),
            format!("descriptor_hash={}", resource.descriptor.descriptor_hash()),
            format!("color_space={}", resource.color_space),
            format!("sampler={}", resource.sampler_policy),
            format!("artifact_id={}", resource.artifact_id.raw()),
            format!("artifact_revision={}", resource.artifact_revision),
            format!("cache_key={}", resource.cache_key.as_str()),
            format!("residency={}", resource.residency_identity),
            format!(
                "pixel_format={:?}",
                resource.descriptor.ktx2_metadata().pixel_format
            ),
            format!(
                "supercompression={:?}",
                resource.descriptor.ktx2_metadata().supercompression
            ),
            format!(
                "container_byte_length={:?}",
                resource.descriptor.ktx2_metadata().byte_length
            ),
        ],
    )
}

fn canonical_identity(label: &str, parts: impl IntoIterator<Item = String>) -> String {
    let mut bytes = Vec::<u8>::new();
    canonical_field(&mut bytes, "label", label);
    for part in parts {
        canonical_field(&mut bytes, "part", part.as_str());
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn canonical_field(bytes: &mut Vec<u8>, label: &str, value: &str) {
    bytes.extend_from_slice(label.as_bytes());
    bytes.push(b'=');
    bytes.extend_from_slice(value.len().to_string().as_bytes());
    bytes.push(b':');
    bytes.extend_from_slice(value.as_bytes());
    bytes.push(b'\n');
}

fn scene_material_binding_slot_for_preview(
    preview: &EditorMaterialPreviewProduct,
    material_table_index: u32,
    slot: &SceneMaterialSlot,
) -> PreparedMaterialBindingSlot {
    PreparedMaterialBindingSlot::new(
        material_table_index,
        material_instance_id_for_slot(slot, preview),
        preview.artifact_id.raw().to_string(),
        preview.shader_artifact_id.raw().to_string(),
        preview.artifact_cache_key.as_str().to_string(),
        preview.shader_cache_key.as_str().to_string(),
    )
}

fn material_instance_id_for_slot(
    slot: &SceneMaterialSlot,
    preview: &EditorMaterialPreviewProduct,
) -> String {
    slot.material_asset_id
        .map(|asset_id| format!("material.asset.{}", asset_id.raw()))
        .or_else(|| {
            slot.source_ref.as_ref().map(|source_ref| {
                format!(
                    "material.source.{}.{}",
                    source_ref.asset_id.raw(),
                    source_ref.source_id.raw()
                )
            })
        })
        .unwrap_or_else(|| format!("material.product.{}", preview.product.product_id.raw()))
}

fn resolved_scene_material_table_identity(
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    slots: &[ResolvedSceneMaterialSlot<'_>],
    resource_layout_identity: &str,
) -> String {
    let mut identity = scene_material_assignments
        .map(stable_scene_material_assignment_identity)
        .unwrap_or_else(|| "scene-material-table:v1:single-preview".to_string());
    identity.push_str(&format!("|resource_layout={resource_layout_identity}"));
    for slot in slots {
        identity.push_str(&format!(
            "|table_slot={}:product={}:shader={}:material_cache={}:shader_cache={}",
            slot.material_table_index,
            slot.preview.product.product_id.raw(),
            slot.preview.scene_shader_identity,
            slot.preview.artifact_cache_key.as_str(),
            slot.preview.shader_cache_key.as_str()
        ));
    }
    identity
}

fn stable_scene_material_assignment_identity(assignments: &SceneMaterialAssignmentState) -> String {
    let mut identity = format!(
        "scene-material-table:v2:revision={}",
        assignments.source_revision()
    );
    for (index, slot) in assignments.palette().slots.iter().enumerate() {
        identity.push_str(&format!(
            "|slot={index}:slot_id={}:entry={}:default={}",
            slot.slot_id.raw(),
            slot.palette_entry_id.raw(),
            slot.is_default,
        ));
        if let Some(source_ref) = &slot.source_ref {
            identity.push_str(&format!(
                ":source_asset={}:source_id={}:source_revision_id={}:source_revision={}",
                source_ref.asset_id.raw(),
                source_ref.source_id.raw(),
                source_ref
                    .source_revision_id
                    .map(|revision| revision.raw().to_string())
                    .unwrap_or_default(),
                source_ref.source_revision.as_deref().unwrap_or_default()
            ));
        }
        if let Some(material_asset_id) = slot.material_asset_id {
            identity.push_str(&format!(":material_asset={}", material_asset_id.raw()));
        }
    }
    for assignment in assignments.assignments() {
        identity.push_str(&format!(
            "|sdf_primitive={}:slot={}",
            assignment.primitive.entity_id().0,
            assignment.slot_id.raw()
        ));
    }
    identity
}

fn scene_material_binding_table(
    preview: &EditorMaterialPreviewProduct,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
) -> PreparedMaterialBindingTable {
    let slots = match scene_material_assignments {
        Some(assignments) => assignments
            .palette()
            .slots
            .iter()
            .enumerate()
            .map(|(index, slot)| scene_material_binding_slot(preview, index as u32, slot))
            .collect::<Vec<_>>(),
        None => vec![preview_material_binding_slot(preview, 0)],
    };
    PreparedMaterialBindingTable::fixed_capacity(slots)
        .expect("editor_scene palette enforces portable material binding slot limits")
}

fn scene_material_binding_slot(
    preview: &EditorMaterialPreviewProduct,
    material_table_index: u32,
    slot: &SceneMaterialSlot,
) -> PreparedMaterialBindingSlot {
    let material_instance_id = slot
        .material_asset_id
        .map(|asset_id| format!("material.asset.{}", asset_id.raw()))
        .or_else(|| {
            slot.source_ref.as_ref().map(|source_ref| {
                format!(
                    "material.source.{}.{}",
                    source_ref.asset_id.raw(),
                    source_ref.source_id.raw()
                )
            })
        })
        .unwrap_or_else(|| {
            format!(
                "material.product.{}.slot.{}",
                preview.product.product_id.raw(),
                slot.slot_id.raw()
            )
        });
    PreparedMaterialBindingSlot::new(
        material_table_index,
        material_instance_id,
        preview.artifact_id.raw().to_string(),
        preview.shader_artifact_id.raw().to_string(),
        preview.artifact_cache_key.as_str().to_string(),
        preview.shader_cache_key.as_str().to_string(),
    )
}

fn preview_material_binding_slot(
    preview: &EditorMaterialPreviewProduct,
    material_table_index: u32,
) -> PreparedMaterialBindingSlot {
    PreparedMaterialBindingSlot::new(
        material_table_index,
        format!("material.product.{}", preview.product.product_id.raw()),
        preview.artifact_id.raw().to_string(),
        preview.shader_artifact_id.raw().to_string(),
        preview.artifact_cache_key.as_str().to_string(),
        preview.shader_cache_key.as_str().to_string(),
    )
}

pub fn prepared_material_resource_for_preview(
    preview: Option<&EditorMaterialPreviewProduct>,
) -> Result<PreparedMaterialFeatureResource, AssetDiagnosticRecord> {
    prepared_material_resource_for_preview_with_scene_materials(preview, None)
}

pub fn prepared_material_resource_for_preview_with_scene_materials(
    preview: Option<&EditorMaterialPreviewProduct>,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
) -> Result<PreparedMaterialFeatureResource, AssetDiagnosticRecord> {
    match preview {
        Some(preview) => {
            let payload = prepared_material_contribution_for_preview_with_scene_materials(
                preview,
                scene_material_assignments,
            );
            payload.validate_portable_limits().map_err(|error| {
                AssetDiagnosticRecord::error(
                    AssetDiagnosticCode::RatificationRejected,
                    format!(
                        "material renderer handoff rejected portable binding limits: {}",
                        error
                    ),
                )
            })?;
            Ok(PreparedMaterialFeatureResource {
                status: FeatureContributionStatus::Ready,
                fallback_policy: FeatureFallbackPolicy::ReuseLastGood,
                payload,
            })
        }
        None => Ok(PreparedMaterialFeatureResource {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::ReuseLastGood,
            payload: PreparedMaterialFeatureContribution::default(),
        }),
    }
}

fn scene_material_table_identity(
    preview: &EditorMaterialPreviewProduct,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
) -> String {
    let base = format!(
        "fixed64:slot0:{}:{}:{}",
        preview.product.product_id.raw(),
        preview.artifact_cache_key.as_str(),
        preview.shader_cache_key.as_str()
    );
    match scene_material_assignments {
        Some(assignments) => {
            format!("{}:{}", base, assignments.material_table_identity())
        }
        None => base,
    }
}

pub fn material_parameter_payload(
    preview: &EditorMaterialPreviewProduct,
) -> PreparedMaterialParameterPayloadV1 {
    PreparedMaterialParameterPayloadV1::new(
        prepared_parameter_profile(preview.renderer_parameter_profile),
        prepared_material_output_target(preview.product.output_target),
        preview.product.parameters.iter().map(|parameter| {
            PreparedMaterialParameterInput::new(
                parameter.key.clone(),
                prepared_parameter_kind(parameter.kind),
            )
        }),
    )
}

fn prepared_parameter_profile(
    profile: MaterialRendererParameterProfile,
) -> PreparedMaterialParameterProfile {
    match profile {
        MaterialRendererParameterProfile::PbrPreview => {
            PreparedMaterialParameterProfile::PbrPreview
        }
        MaterialRendererParameterProfile::RenderMaterial => {
            PreparedMaterialParameterProfile::RenderMaterial
        }
    }
}

fn prepared_material_output_target(
    output_target: MaterialOutputTarget,
) -> PreparedMaterialOutputTarget {
    match output_target {
        MaterialOutputTarget::PbrPreview => PreparedMaterialOutputTarget::PbrPreview,
        MaterialOutputTarget::FieldMaterialChannel => {
            PreparedMaterialOutputTarget::FieldMaterialChannel
        }
        MaterialOutputTarget::RenderMaterial => PreparedMaterialOutputTarget::RenderMaterial,
    }
}

fn prepared_parameter_kind(kind: MaterialParameterKind) -> PreparedMaterialParameterKind {
    match kind {
        MaterialParameterKind::Scalar => PreparedMaterialParameterKind::Scalar,
        MaterialParameterKind::Vector2 => PreparedMaterialParameterKind::Vector2,
        MaterialParameterKind::Vector3 => PreparedMaterialParameterKind::Vector3,
        MaterialParameterKind::Vector4 => PreparedMaterialParameterKind::Vector4,
        MaterialParameterKind::Texture2D => PreparedMaterialParameterKind::Texture2D,
        MaterialParameterKind::Texture3D => PreparedMaterialParameterKind::Texture3D,
    }
}

fn prepared_texture_bindings(
    preview: &EditorMaterialPreviewProduct,
) -> Vec<PreparedMaterialTextureBinding> {
    preview
        .resolved_resources
        .iter()
        .enumerate()
        .map(|(index, resource)| prepared_texture_binding_for_resource(resource, index as u32))
        .collect()
}

fn prepared_texture_binding_for_resource(
    resource: &crate::material_lab::ResolvedMaterialResource,
    resource_slot_index: u32,
) -> PreparedMaterialTextureBinding {
    let mut binding = PreparedMaterialTextureBinding::new(
        resource.node_id.raw(),
        resource.binding_key.clone(),
        resource.artifact_id.raw().to_string(),
        resource.artifact_path.clone(),
        match resource.kind {
            asset::AssetKind::Texture3DVolume => PreparedMaterialTextureKind::Texture3D,
            _ => PreparedMaterialTextureKind::Texture2D,
        },
        resource.cache_key.as_str().to_string(),
    )
    .with_resource_slot_index(resource_slot_index)
    .with_texture_dimension(resource.dimension.clone())
    .with_extent(
        resource.descriptor.extent.width,
        resource.descriptor.extent.height,
        resource.descriptor.extent.depth,
    )
    .with_residency_identity(resource.residency_identity.clone())
    .with_artifact_revision(resource.artifact_revision.clone())
    .with_descriptor_hash(resource.descriptor.descriptor_hash().to_string())
    .with_ktx2_contract(
        format!("{:?}", resource.descriptor.ktx2_metadata().pixel_format),
        format!("{:?}", resource.descriptor.ktx2_metadata().supercompression),
        resource.descriptor.ktx2_metadata().byte_length,
    );
    binding.sampler_policy = resource.sampler_policy.clone();
    binding
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material_lab::{MaterialRendererParameterProfile, PreviewSceneProductBuildStatus};
    use asset::{
        ArtifactCacheKey, AssetDiagnosticCode, AssetKind, asset_artifact_id, asset_id,
        asset_source_id,
    };
    use editor_core::EntityId;
    use editor_scene::{
        SceneMaterialAssignmentState, SceneMaterialPalette, SceneMaterialSlot, SceneMaterialSlotId,
        SdfPrimitiveMaterialSlotAssignment, SdfPrimitiveSourceId,
    };
    use material_graph::{
        FormedMaterialProduct, MaterialCacheKey, MaterialGraphDocumentId, MaterialOutputTarget,
        MaterialParameterDescriptor, MaterialParameterKind, MaterialProductId,
    };
    use resource_ref::ResourceRef;
    use texture::{TextureDescriptor, TextureDimension, TextureExtent, TextureProductId};

    #[test]
    fn material_handoff_prepared_resource_uses_formed_product_specialization_and_parameters() {
        let mut product = FormedMaterialProduct::new(
            MaterialProductId::new(3),
            MaterialGraphDocumentId::new(2),
            MaterialOutputTarget::RenderMaterial,
            MaterialCacheKey::new("material-cache"),
        );
        product.parameters = vec![MaterialParameterDescriptor::new(
            "roughness",
            MaterialParameterKind::Scalar,
        )];
        let preview = EditorMaterialPreviewProduct::new(
            asset_id(1),
            asset_source_id(2),
            asset_artifact_id(4),
            ArtifactCacheKey::new("asset-cache"),
            product,
            MaterialRendererParameterProfile::RenderMaterial,
            asset_artifact_id(5),
            ArtifactCacheKey::new("shader-cache"),
            ".runenwerk/artifacts/material-shader-5.wgsl",
            "shader-identity",
            asset_artifact_id(6),
            ArtifactCacheKey::new("scene-shader-cache"),
            ".runenwerk/artifacts/material-scene-shader-6.wgsl",
            "scene-shader-identity",
            Vec::new(),
        );

        let prepared =
            prepared_material_resource_for_preview(Some(&preview)).expect("handoff should prepare");

        assert_eq!(prepared.status, FeatureContributionStatus::Ready);
        assert_eq!(prepared.payload.instances.len(), 1);
        assert_eq!(
            prepared.payload.instances[0].specialization_key_fragment,
            "material.first_slice"
        );
        let encoded = prepared.payload.instances[0].parameter_payload.encode_v1();
        let payload =
            PreparedMaterialParameterPayloadV1::decode_v1(&encoded).expect("payload should decode");
        assert_eq!(
            payload.profile,
            PreparedMaterialParameterProfile::RenderMaterial
        );
        assert_eq!(
            payload.output_target,
            PreparedMaterialOutputTarget::RenderMaterial
        );
        assert_eq!(payload.parameters.len(), 1);
        assert_eq!(payload.parameters[0].key, "roughness");
        assert_eq!(prepared.payload.binding_table.slots.len(), 1);
        let scene_bundle = prepared
            .payload
            .scene_bundle
            .as_ref()
            .expect("material handoff should carry the scene bundle as feature data");
        assert_eq!(
            scene_bundle.shader_path,
            ".runenwerk/artifacts/material-scene-shader-6.wgsl"
        );
        assert_eq!(scene_bundle.shader_identity, "scene-shader-identity");
        let blob = std::str::from_utf8(&encoded).expect("payload should be utf8");
        assert!(blob.contains("format=32:runenwerk.material-parameters.v1"));
        assert!(blob.contains("profile=15:render_material"));
        assert!(blob.contains("parameter_kind=6:scalar"));
        assert!(
            !blob.contains("Scalar"),
            "prepared material payload must not use Rust debug formatting"
        );
    }

    #[test]
    fn material_handoff_reports_portable_limit_diagnostics() {
        let mut product = FormedMaterialProduct::new(
            MaterialProductId::new(3),
            MaterialGraphDocumentId::new(2),
            MaterialOutputTarget::RenderMaterial,
            MaterialCacheKey::new("material-cache"),
        );
        product.parameters = vec![MaterialParameterDescriptor::new(
            "albedo",
            MaterialParameterKind::Texture2D,
        )];
        let mut preview = EditorMaterialPreviewProduct::new(
            asset_id(1),
            asset_source_id(2),
            asset_artifact_id(4),
            ArtifactCacheKey::new("asset-cache"),
            product,
            MaterialRendererParameterProfile::RenderMaterial,
            asset_artifact_id(5),
            ArtifactCacheKey::new("shader-cache"),
            ".runenwerk/artifacts/material-shader-5.wgsl",
            "shader-identity",
            asset_artifact_id(6),
            ArtifactCacheKey::new("scene-shader-cache"),
            ".runenwerk/artifacts/material-scene-shader-6.wgsl",
            "scene-shader-identity",
            (0..129)
                .map(test_resolved_texture_resource)
                .collect::<Vec<_>>(),
        );

        let diagnostic = prepared_material_resource_for_preview(Some(&preview))
            .expect_err("portable texture binding limit must be a visible diagnostic");

        assert_eq!(diagnostic.code, AssetDiagnosticCode::RatificationRejected);
        assert!(
            diagnostic.message.contains("portable binding limits"),
            "portable limit failure should not collapse into a generic missing feature"
        );

        preview.resolved_resources.truncate(128);
        assert!(prepared_material_resource_for_preview(Some(&preview)).is_ok());
    }

    #[test]
    fn material_table_identity_changes_with_sdf_assignment_state() {
        let product = FormedMaterialProduct::new(
            MaterialProductId::new(3),
            MaterialGraphDocumentId::new(2),
            MaterialOutputTarget::RenderMaterial,
            MaterialCacheKey::new("material-cache"),
        );
        let preview = EditorMaterialPreviewProduct::new(
            asset_id(1),
            asset_source_id(2),
            asset_artifact_id(4),
            ArtifactCacheKey::new("asset-cache"),
            product,
            MaterialRendererParameterProfile::RenderMaterial,
            asset_artifact_id(5),
            ArtifactCacheKey::new("shader-cache"),
            ".runenwerk/artifacts/material-shader-5.wgsl",
            "shader-identity",
            asset_artifact_id(6),
            ArtifactCacheKey::new("scene-shader-cache"),
            ".runenwerk/artifacts/material-scene-shader-6.wgsl",
            "scene-shader-identity",
            Vec::new(),
        );
        let slot_two = SceneMaterialSlotId::new(2);
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(slot_two, "Assigned"),
        ])
        .expect("valid palette");
        let mut assignments =
            SceneMaterialAssignmentState::new(palette.clone(), []).expect("default state");
        let before = prepared_material_contribution_for_preview_with_scene_materials(
            &preview,
            Some(&assignments),
        )
        .scene_bundle
        .expect("scene bundle")
        .material_table_identity;

        assignments = SceneMaterialAssignmentState::new(
            palette,
            [SdfPrimitiveMaterialSlotAssignment::new(
                SdfPrimitiveSourceId::new(EntityId(42)),
                slot_two,
            )],
        )
        .expect("assigned state");
        let after = prepared_material_contribution_for_preview_with_scene_materials(
            &preview,
            Some(&assignments),
        )
        .scene_bundle
        .expect("scene bundle")
        .material_table_identity;

        assert_ne!(before, after);
        assert!(after.contains("sdf_primitive=42:slot=2"));
    }

    #[test]
    fn material_binding_table_follows_editor_scene_palette_slots() {
        let product = FormedMaterialProduct::new(
            MaterialProductId::new(3),
            MaterialGraphDocumentId::new(2),
            MaterialOutputTarget::RenderMaterial,
            MaterialCacheKey::new("material-cache"),
        );
        let preview = EditorMaterialPreviewProduct::new(
            asset_id(1),
            asset_source_id(2),
            asset_artifact_id(4),
            ArtifactCacheKey::new("asset-cache"),
            product,
            MaterialRendererParameterProfile::RenderMaterial,
            asset_artifact_id(5),
            ArtifactCacheKey::new("shader-cache"),
            ".runenwerk/artifacts/material-shader-5.wgsl",
            "shader-identity",
            asset_artifact_id(6),
            ArtifactCacheKey::new("scene-shader-cache"),
            ".runenwerk/artifacts/material-scene-shader-6.wgsl",
            "scene-shader-identity",
            Vec::new(),
        );
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [SdfPrimitiveMaterialSlotAssignment::new(
                SdfPrimitiveSourceId::new(EntityId(42)),
                SceneMaterialSlotId::new(2),
            )],
        )
        .expect("valid assignments");

        let contribution = prepared_material_contribution_for_preview_with_scene_materials(
            &preview,
            Some(&assignments),
        );

        assert_eq!(contribution.binding_table.slots.len(), 2);
        assert_eq!(contribution.binding_table.slots[0].slot_index, 0);
        assert_eq!(contribution.binding_table.slots[1].slot_index, 1);
        assert_eq!(
            contribution.binding_table.slots[1].material_instance_id,
            "material.asset.8"
        );
        assert_eq!(
            contribution.binding_table.slots[1].formed_material_artifact_id,
            "4"
        );
        assert_eq!(contribution.binding_table.slots[1].shader_artifact_id, "5");
        assert_eq!(
            contribution.binding_table.slots[1].material_cache_key,
            "asset-cache"
        );
        assert_eq!(
            contribution.binding_table.slots[1].shader_cache_key,
            "shader-cache"
        );
        assert!(!contribution.binding_table.slots[1].prior_valid);
    }

    #[test]
    fn material_binding_table_uses_resolved_source_backed_slot_products() {
        let default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let assigned_preview = test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [SdfPrimitiveMaterialSlotAssignment::new(
                SdfPrimitiveSourceId::new(EntityId(42)),
                SceneMaterialSlotId::new(2),
            )],
        )
        .expect("valid assignments");

        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let bundle = test_scene_table_bundle(&default_preview, &assignments, &slot_products);
        let contribution =
            prepared_material_contribution_for_preview_with_resolved_scene_materials_and_bundle(
                &default_preview,
                Some(&assignments),
                &slot_products,
                Some(&bundle),
            )
            .expect("source-backed slot product should prepare");

        assert_eq!(contribution.instances.len(), 2);
        assert_eq!(
            contribution.instances[0].material_instance_id,
            "material.product.3"
        );
        assert_eq!(
            contribution.instances[1].material_instance_id,
            "material.asset.8"
        );
        assert_eq!(contribution.binding_table.slots.len(), 2);
        assert_eq!(
            contribution.binding_table.slots[1].formed_material_artifact_id,
            "10"
        );
        assert_eq!(contribution.binding_table.slots[1].shader_artifact_id, "11");
        assert_eq!(
            contribution.binding_table.slots[1].material_cache_key,
            "asset-cache-rock"
        );
        let scene_bundle = contribution.scene_bundle.expect("scene bundle");
        assert!(scene_bundle.material_table_identity.contains("product=9"));
        assert!(!scene_bundle.resource_layout_identity.is_empty());
        assert_eq!(scene_bundle.shader_path, bundle.shader_path);
    }

    #[test]
    fn single_material_preview_scene_product_builds_from_active_preview() {
        let mut preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        preview.resolved_resources = vec![test_resolved_texture_resource(0)];

        let outcome = build_preview_scene_product_for_single_material(&preview);

        assert!(matches!(
            outcome.status,
            PreviewSceneProductBuildStatus::Ready
        ));
        let product = outcome.product.expect("single material product");
        assert_eq!(product.mode, PreviewSceneProductMode::SingleMaterial);
        assert_eq!(
            product.active_material_product_id,
            preview.product.product_id
        );
        assert_eq!(
            product.active_material_artifact_cache_key,
            preview.artifact_cache_key
        );
        assert_eq!(
            product.shader.shader_identity,
            preview.scene_shader_identity
        );
        assert_eq!(
            product.shader.shader_cache_key,
            preview.scene_shader_cache_key
        );
        assert_eq!(product.resource_layout_identity, "");
        assert_eq!(product.slots.len(), 1);
        assert_eq!(product.resources.len(), 1);
        assert_eq!(
            product.slots[0].resource_slot_mappings,
            vec![PreviewSceneResourceSlotMapping::new(0, 0)]
        );
    }

    #[test]
    fn single_material_preview_scene_product_state_does_not_require_generated_bundle() {
        let preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");

        let outcome =
            build_preview_scene_product_for_scene_material_table(&preview, None, &[], None);

        assert!(matches!(
            outcome.status,
            PreviewSceneProductBuildStatus::Ready
        ));
        let product = outcome.product.expect("single material product");
        assert_eq!(product.mode, PreviewSceneProductMode::SingleMaterial);
        assert!(
            outcome
                .request_identity
                .as_ref()
                .is_some_and(|identity| identity.matches_product(&product))
        );
    }

    #[test]
    fn scene_material_table_preview_scene_product_requires_generated_bundle() {
        let default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let assigned_preview = test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];

        let outcome = build_preview_scene_product_for_scene_material_table(
            &default_preview,
            Some(&assignments),
            &slot_products,
            None,
        );

        assert!(outcome.product.is_none());
        let PreviewSceneProductBuildStatus::FailedClosed { diagnostics } = outcome.status else {
            panic!("missing generated bundle must fail closed");
        };
        assert_eq!(
            diagnostics[0].code,
            "material.preview_scene.generated_bundle_missing"
        );
    }

    #[test]
    fn scene_material_table_preview_scene_product_state_requires_matching_bundle() {
        let default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let assigned_preview = test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let stale_bundle = EditorSceneMaterialTableShaderBundle::new(
            "stale-artifact",
            ArtifactCacheKey::new("stale-cache"),
            ".runenwerk/artifacts/generated/material-scene-table-shader/stale.wgsl",
            "stale-shader",
            "stale-material-table",
            "stale-resource-layout",
        );
        let matching_bundle =
            test_scene_table_bundle(&default_preview, &assignments, &slot_products);

        let missing = build_preview_scene_product_for_scene_material_table(
            &default_preview,
            Some(&assignments),
            &slot_products,
            None,
        );
        assert!(missing.product.is_none());
        assert!(missing.request_identity.is_some());

        let stale = build_preview_scene_product_for_scene_material_table(
            &default_preview,
            Some(&assignments),
            &slot_products,
            Some(&stale_bundle),
        );
        assert!(stale.product.is_none());
        assert!(stale.request_identity.is_some());

        let ready = build_preview_scene_product_for_scene_material_table(
            &default_preview,
            Some(&assignments),
            &slot_products,
            Some(&matching_bundle),
        );
        let product = ready.product.expect("matching bundle should build");
        assert_eq!(product.mode, PreviewSceneProductMode::SceneMaterialTable);
        assert!(
            ready
                .request_identity
                .as_ref()
                .is_some_and(|identity| identity.matches_product(&product))
        );
    }

    #[test]
    fn stale_scene_material_table_bundle_fails_preview_scene_product_build() {
        let default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let assigned_preview = test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let stale_bundle = EditorSceneMaterialTableShaderBundle::new(
            "stale-artifact",
            ArtifactCacheKey::new("stale-cache"),
            ".runenwerk/artifacts/generated/material-scene-table-shader/stale.wgsl",
            "stale-shader",
            "stale-material-table",
            "stale-resource-layout",
        );

        let outcome = build_preview_scene_product_for_scene_material_table(
            &default_preview,
            Some(&assignments),
            &slot_products,
            Some(&stale_bundle),
        );

        assert!(outcome.product.is_none());
        let PreviewSceneProductBuildStatus::FailedClosed { diagnostics } = outcome.status else {
            panic!("stale generated bundle must fail closed");
        };
        assert_eq!(
            diagnostics[0].code,
            "material.preview_scene.generated_bundle_stale"
        );
    }

    #[test]
    fn preview_scene_product_builder_preserves_table_wide_resource_slots() {
        let mut default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        default_preview.resolved_resources = vec![test_resolved_texture_resource(0)];
        let mut assigned_preview =
            test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        assigned_preview.resolved_resources = vec![test_resolved_texture_resource(1)];
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let bundle = test_scene_table_bundle(&default_preview, &assignments, &slot_products);

        let outcome = build_preview_scene_product_for_scene_material_table(
            &default_preview,
            Some(&assignments),
            &slot_products,
            Some(&bundle),
        );

        let product = outcome.product.expect("scene table product");
        assert_eq!(product.mode, PreviewSceneProductMode::SceneMaterialTable);
        assert_eq!(product.resources.len(), 2);
        assert_eq!(product.resources[0].table_resource_slot, 0);
        assert_eq!(product.resources[1].table_resource_slot, 1);
        assert_ne!(
            product.resources[0].resource_product_identity,
            product.resources[1].resource_product_identity
        );
        assert!(
            product
                .resources
                .iter()
                .all(|resource| !resource.resource_product_identity.contains(".runenwerk"))
        );
    }

    #[test]
    fn preview_scene_product_builder_records_slot_to_table_resource_mappings() {
        let mut default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        default_preview.resolved_resources = vec![test_resolved_texture_resource(0)];
        let mut assigned_preview =
            test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        assigned_preview.resolved_resources = vec![test_resolved_texture_resource(1)];
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let bundle = test_scene_table_bundle(&default_preview, &assignments, &slot_products);

        let outcome = build_preview_scene_product_for_scene_material_table(
            &default_preview,
            Some(&assignments),
            &slot_products,
            Some(&bundle),
        );

        let product = outcome.product.expect("scene table product");
        assert_eq!(
            product.slots[0].resource_slot_mappings,
            vec![PreviewSceneResourceSlotMapping::new(0, 0)]
        );
        assert_eq!(
            product.slots[1].resource_slot_mappings,
            vec![PreviewSceneResourceSlotMapping::new(0, 1)]
        );
    }

    #[test]
    fn unresolved_explicit_scene_slot_does_not_fallback_to_active_preview() {
        let default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let explicit_default =
            SceneMaterialSlot::default_generated().with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([explicit_default]).expect("valid palette"),
            [],
        )
        .expect("valid assignments");

        let outcome = build_preview_scene_product_for_scene_material_table(
            &default_preview,
            Some(&assignments),
            &[],
            None,
        );

        assert!(outcome.product.is_none());
        let PreviewSceneProductBuildStatus::FailedClosed { diagnostics } = outcome.status else {
            panic!("unresolved explicit slot must fail closed");
        };
        assert_eq!(
            diagnostics[0].code,
            "material.preview_scene.unresolved_scene_slot"
        );
        assert!(diagnostics[0].message.contains("slot 1"));
    }

    #[test]
    fn renderer_handoff_uses_preview_scene_product_without_behavior_change() {
        let mut preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        preview.resolved_resources = vec![test_resolved_texture_resource(0)];

        let legacy = prepared_material_contribution_for_preview(&preview);
        let refactored =
            prepared_material_contribution_for_preview_with_resolved_scene_materials_and_bundle(
                &preview,
                None,
                &[],
                None,
            )
            .expect("single preview handoff should prepare");

        assert_eq!(refactored.instances.len(), legacy.instances.len());
        for (actual, expected) in refactored.instances.iter().zip(legacy.instances.iter()) {
            assert_eq!(actual.material_instance_id, expected.material_instance_id);
            assert_eq!(
                actual.specialization_key_fragment,
                expected.specialization_key_fragment
            );
            assert_eq!(
                actual.parameter_payload.encode_v1(),
                expected.parameter_payload.encode_v1()
            );
            assert_eq!(actual.texture_bindings, expected.texture_bindings);
        }
        assert_eq!(refactored.binding_table, legacy.binding_table);
        assert_eq!(refactored.scene_bundle, legacy.scene_bundle);
    }

    #[test]
    fn single_material_path_still_uses_active_preview_scene_shader() {
        let preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");

        let outcome = build_preview_scene_product_for_single_material(&preview);
        let product = outcome.product.expect("single preview product");
        let contribution =
            prepared_material_contribution_for_preview_with_resolved_scene_materials_and_bundle(
                &preview,
                None,
                &[],
                None,
            )
            .expect("single preview handoff should prepare");

        assert_eq!(
            product.shader.shader_identity,
            preview.scene_shader_identity
        );
        assert_eq!(
            product.shader.shader_artifact_id,
            preview.scene_shader_artifact_id.raw().to_string()
        );
        assert_eq!(
            contribution
                .scene_bundle
                .as_ref()
                .expect("scene bundle")
                .shader_identity,
            preview.scene_shader_identity
        );
    }

    #[test]
    fn multi_slot_path_uses_generated_scene_table_shader_ref() {
        let default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let assigned_preview = test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let bundle = test_scene_table_bundle(&default_preview, &assignments, &slot_products);

        let outcome = build_preview_scene_product_for_scene_material_table(
            &default_preview,
            Some(&assignments),
            &slot_products,
            Some(&bundle),
        );
        let product = outcome.product.expect("scene table product");
        let contribution =
            prepared_material_contribution_for_preview_with_resolved_scene_materials_and_bundle(
                &default_preview,
                Some(&assignments),
                &slot_products,
                Some(&bundle),
            )
            .expect("scene table handoff should prepare");

        assert_eq!(product.mode, PreviewSceneProductMode::SceneMaterialTable);
        assert_eq!(product.shader.shader_artifact_id, bundle.shader_artifact_id);
        assert_eq!(product.shader.shader_cache_key, bundle.shader_cache_key);
        assert_eq!(product.shader.shader_identity, bundle.shader_identity);
        assert_eq!(
            contribution
                .scene_bundle
                .as_ref()
                .expect("scene bundle")
                .shader_identity,
            bundle.shader_identity
        );
    }

    #[test]
    fn stale_preview_scene_product_fails_before_renderer_handoff() {
        let default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let assigned_preview = test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let bundle = test_scene_table_bundle(&default_preview, &assignments, &slot_products);
        let stale_product = build_preview_scene_product_for_single_material(&default_preview)
            .product
            .expect("single material product");

        let diagnostic = prepared_material_contribution_for_preview_scene_product(
            &stale_product,
            &default_preview,
            Some(&assignments),
            &slot_products,
            Some(&bundle),
        )
        .expect_err("stale product must fail before prepared renderer handoff");

        assert_eq!(diagnostic.code, AssetDiagnosticCode::RatificationRejected);
        assert!(diagnostic.message.contains("stale"));
    }

    #[test]
    fn stale_preview_scene_product_shader_identity_fails_before_renderer_handoff() {
        let default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let assigned_preview = test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let bundle = test_scene_table_bundle(&default_preview, &assignments, &slot_products);
        let mut stale_bundle = bundle.clone();
        stale_bundle.shader_artifact_id = "stale-scene-table-artifact".to_string();
        stale_bundle.shader_cache_key = ArtifactCacheKey::new("stale-scene-table-cache");
        stale_bundle.shader_identity = "stale-scene-table-shader-identity".to_string();
        let stale_product = build_preview_scene_product_for_scene_material_table(
            &default_preview,
            Some(&assignments),
            &slot_products,
            Some(&stale_bundle),
        )
        .product
        .expect("stale shader product still matches the scene request shape");

        let diagnostic = prepared_material_contribution_for_preview_scene_product(
            &stale_product,
            &default_preview,
            Some(&assignments),
            &slot_products,
            Some(&bundle),
        )
        .expect_err("stale shader identity must fail before prepared renderer handoff");

        assert_eq!(diagnostic.code, AssetDiagnosticCode::RatificationRejected);
        assert!(diagnostic.message.contains("stale"));
    }

    #[test]
    fn scene_material_table_handoff_remaps_duplicate_local_texture_slots() {
        let mut default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        default_preview.resolved_resources = vec![test_resolved_texture_resource(0)];
        let mut assigned_preview =
            test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        assigned_preview.resolved_resources = vec![test_resolved_texture_resource(1)];
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [SdfPrimitiveMaterialSlotAssignment::new(
                SdfPrimitiveSourceId::new(EntityId(42)),
                SceneMaterialSlotId::new(2),
            )],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let bundle = test_scene_table_bundle(&default_preview, &assignments, &slot_products);

        let contribution =
            prepared_material_contribution_for_preview_with_resolved_scene_materials_and_bundle(
                &default_preview,
                Some(&assignments),
                &slot_products,
                Some(&bundle),
            )
            .expect("scene material table should prepare with table-wide texture slots");

        assert_eq!(contribution.instances.len(), 2);
        let resource_slots = contribution
            .instances
            .iter()
            .flat_map(|instance| instance.texture_bindings.iter())
            .map(|binding| binding.resource_slot_index)
            .collect::<Vec<_>>();
        assert_eq!(resource_slots, vec![0, 1]);
        assert!(
            contribution.validate_portable_limits().is_ok(),
            "local slot 0 from different resources must become valid table-wide slots"
        );
    }

    #[test]
    fn scene_material_table_handoff_deduplicates_strictly_identical_resources() {
        let mut default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let shared = test_resolved_texture_resource(7);
        default_preview.resolved_resources = vec![shared.clone()];
        let mut assigned_preview =
            test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        let mut assigned_resource = shared;
        assigned_resource.node_id = graph::NodeId::new(99);
        assigned_preview.resolved_resources = vec![assigned_resource];
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let bundle = test_scene_table_bundle(&default_preview, &assignments, &slot_products);

        let contribution =
            prepared_material_contribution_for_preview_with_resolved_scene_materials_and_bundle(
                &default_preview,
                Some(&assignments),
                &slot_products,
                Some(&bundle),
            )
            .expect("identical resources should deduplicate");

        let flattened = contribution
            .instances
            .iter()
            .flat_map(|instance| instance.texture_bindings.iter())
            .collect::<Vec<_>>();
        assert_eq!(flattened.len(), 1);
        assert_eq!(flattened[0].resource_slot_index, 0);
    }

    #[test]
    fn scene_material_table_handoff_treats_sampler_contract_as_resource_identity() {
        let mut default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let shared = test_resolved_texture_resource(9);
        default_preview.resolved_resources = vec![shared.clone()];
        let mut identical_assigned_preview =
            test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        let mut identical_assigned_resource = shared.clone();
        identical_assigned_resource.node_id = graph::NodeId::new(99);
        identical_assigned_preview.resolved_resources = vec![identical_assigned_resource];
        let mut assigned_preview =
            test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        let mut assigned_resource = shared;
        assigned_resource.sampler_policy = "nearest_clamp".to_string();
        assigned_preview.resolved_resources = vec![assigned_resource];
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let identical_slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &identical_assigned_preview,
        }];
        let identical_expectation = scene_material_table_bundle_expectation_for_preview(
            &default_preview,
            Some(&assignments),
            &identical_slot_products,
        )
        .expect("identical scene table expectation should resolve")
        .expect("test palette requires a generated bundle");
        let changed_sampler_expectation = scene_material_table_bundle_expectation_for_preview(
            &default_preview,
            Some(&assignments),
            &slot_products,
        )
        .expect("changed sampler scene table expectation should resolve")
        .expect("test palette requires a generated bundle");
        assert_ne!(
            identical_expectation.resource_layout_identity,
            changed_sampler_expectation.resource_layout_identity
        );
        let bundle = test_scene_table_bundle(&default_preview, &assignments, &slot_products);

        let contribution =
            prepared_material_contribution_for_preview_with_resolved_scene_materials_and_bundle(
                &default_preview,
                Some(&assignments),
                &slot_products,
                Some(&bundle),
            )
            .expect("different sampler contracts must not deduplicate");

        let resource_slots = contribution
            .instances
            .iter()
            .flat_map(|instance| instance.texture_bindings.iter())
            .map(|binding| binding.resource_slot_index)
            .collect::<Vec<_>>();
        assert_eq!(resource_slots, vec![0, 1]);
    }

    #[test]
    fn stale_scene_material_table_shader_bundle_fails_closed() {
        let default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let assigned_preview = test_preview_product_with_ids(asset_id(8), 9, 10, 11, 12, "rock");
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let stale_bundle = EditorSceneMaterialTableShaderBundle::new(
            "stale-artifact",
            ArtifactCacheKey::new("stale-cache"),
            ".runenwerk/artifacts/generated/material-scene-table-shader/stale.wgsl",
            "stale-shader",
            "stale-material-table",
            "stale-resource-layout",
        );

        let diagnostic =
            prepared_material_contribution_for_preview_with_resolved_scene_materials_and_bundle(
                &default_preview,
                Some(&assignments),
                &slot_products,
                Some(&stale_bundle),
            )
            .expect_err("stale scene table shader bundle must fail closed");

        assert_eq!(diagnostic.code, AssetDiagnosticCode::RatificationRejected);
        assert!(diagnostic.message.contains("stale"));
    }

    #[test]
    fn unresolved_source_backed_scene_material_slot_fails_closed() {
        let default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let assigned_slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Assigned")
            .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), assigned_slot])
                .expect("valid palette"),
            [],
        )
        .expect("valid assignments");

        let diagnostic = prepared_material_contribution_for_preview_with_resolved_scene_materials(
            &default_preview,
            Some(&assignments),
            &[],
        )
        .expect_err("missing source-backed product must fail closed");

        assert_eq!(diagnostic.code, AssetDiagnosticCode::RatificationRejected);
        assert!(diagnostic.message.contains("slot 2"));
        assert!(
            diagnostic
                .message
                .contains("no resolved source-backed material product")
        );
    }

    #[test]
    fn unresolved_explicit_default_scene_material_slot_does_not_use_active_preview_fallback() {
        let default_preview = test_preview_product_with_ids(asset_id(1), 3, 4, 5, 6, "default");
        let explicit_default =
            SceneMaterialSlot::default_generated().with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([explicit_default]).expect("valid palette"),
            [],
        )
        .expect("valid assignments");

        let diagnostic = prepared_material_contribution_for_preview_with_resolved_scene_materials(
            &default_preview,
            Some(&assignments),
            &[],
        )
        .expect_err("explicit unresolved default slot must not fall back to active preview");

        assert_eq!(diagnostic.code, AssetDiagnosticCode::RatificationRejected);
        assert!(diagnostic.message.contains("slot 1"));
        assert!(
            diagnostic
                .message
                .contains("no resolved source-backed material product")
        );
    }

    fn test_resolved_texture_resource(index: u64) -> crate::material_lab::ResolvedMaterialResource {
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(index + 1),
            format!("Texture {index}"),
            TextureDimension::Texture2D,
            TextureExtent::new(1, 1, 1),
        );
        crate::material_lab::ResolvedMaterialResource {
            node_id: graph::NodeId::new(index + 1),
            binding_key: format!("albedo_{index}"),
            reference: ResourceRef::new("texture", format!("texture.{index}"))
                .expect("test resource ref should be valid"),
            artifact_id: asset_artifact_id(index + 100),
            artifact_path: format!(".runenwerk/artifacts/texture-{index}.ktx2"),
            kind: AssetKind::Texture2D,
            cache_key: ArtifactCacheKey::new(format!("texture-cache-{index}")),
            descriptor,
            artifact_revision: "1".to_string(),
            dimension: "2d".to_string(),
            color_space: "linear".to_string(),
            sampler_policy: "linear_repeat".to_string(),
            residency_identity: format!("ktx2:texture:{index}"),
        }
    }

    fn test_scene_table_bundle(
        default_preview: &EditorMaterialPreviewProduct,
        assignments: &SceneMaterialAssignmentState,
        slot_products: &[SceneMaterialSlotProduct<'_>],
    ) -> EditorSceneMaterialTableShaderBundle {
        let expectation = scene_material_table_bundle_expectation_for_preview(
            default_preview,
            Some(assignments),
            slot_products,
        )
        .expect("scene table expectation should resolve")
        .expect("test palette requires generated table shader");
        EditorSceneMaterialTableShaderBundle::new(
            "scene-table-artifact",
            ArtifactCacheKey::new("scene-table-cache"),
            ".runenwerk/artifacts/generated/material-scene-table-shader/test.wgsl",
            "scene-table-shader-identity",
            expectation.material_table_identity,
            expectation.resource_layout_identity,
        )
    }

    fn test_preview_product_with_ids(
        asset_id: asset::AssetId,
        product_id: u64,
        artifact_id: u64,
        shader_artifact_id: u64,
        scene_shader_artifact_id: u64,
        label: &str,
    ) -> EditorMaterialPreviewProduct {
        let product = FormedMaterialProduct::new(
            MaterialProductId::new(product_id),
            MaterialGraphDocumentId::new(product_id + 100),
            MaterialOutputTarget::RenderMaterial,
            MaterialCacheKey::new(format!("material-cache-{label}")),
        );
        EditorMaterialPreviewProduct::new(
            asset_id,
            asset_source_id(product_id + 200),
            asset_artifact_id(artifact_id),
            ArtifactCacheKey::new(format!("asset-cache-{label}")),
            product,
            MaterialRendererParameterProfile::RenderMaterial,
            asset_artifact_id(shader_artifact_id),
            ArtifactCacheKey::new(format!("shader-cache-{label}")),
            format!(".runenwerk/artifacts/material-shader-{label}.wgsl"),
            format!("shader-identity-{label}"),
            asset_artifact_id(scene_shader_artifact_id),
            ArtifactCacheKey::new(format!("scene-shader-cache-{label}")),
            format!(".runenwerk/artifacts/material-scene-shader-{label}.wgsl"),
            format!("scene-shader-identity-{label}"),
            Vec::new(),
        )
    }
}
