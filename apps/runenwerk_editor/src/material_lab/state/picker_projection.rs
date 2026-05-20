use super::*;
use crate::material_lab::model_mesh_regions::catalog_model_mesh_material_regions;

pub(super) fn material_node_palette_view_model(search_query: &str) -> MaterialNodePaletteViewModel {
    let needle = search_query.trim().to_ascii_lowercase();
    let mut categories =
        std::collections::BTreeMap::<String, Vec<MaterialNodePaletteItemViewModel>>::new();
    for descriptor in material_graph::MaterialNodeCatalog::first_slice().descriptors() {
        if !needle.is_empty()
            && !descriptor.key.to_ascii_lowercase().contains(&needle)
            && !descriptor.label.to_ascii_lowercase().contains(&needle)
        {
            continue;
        }
        let category = descriptor
            .key
            .split_once('.')
            .map(|(prefix, _)| match prefix {
                "pbr" => "PBR",
                "sdf" => "SDF Context",
                "proc" => "Procedural",
                "math" => "Math",
                "texture" => "Textures",
                "coord" => "Coordinates",
                _ => "Material",
            })
            .unwrap_or("Material")
            .to_string();
        categories
            .entry(category)
            .or_default()
            .push(MaterialNodePaletteItemViewModel {
                descriptor_key: descriptor.key.clone(),
                label: descriptor.label.clone(),
                output_targets: descriptor.output_targets.clone(),
            });
    }
    MaterialNodePaletteViewModel {
        search_query: search_query.to_string(),
        categories: categories
            .into_iter()
            .map(|(label, nodes)| MaterialNodePaletteCategoryViewModel { label, nodes })
            .collect(),
    }
}

pub(super) fn material_node_picker_view_model(
    open: bool,
    search_query: &str,
    highlighted_descriptor_key: Option<&str>,
) -> MaterialNodePickerViewModel {
    let palette = material_node_palette_view_model(search_query);
    let highlighted_descriptor_key = highlighted_descriptor_key
        .filter(|descriptor_key| {
            palette.categories.iter().any(|category| {
                category
                    .nodes
                    .iter()
                    .any(|node| node.descriptor_key == *descriptor_key)
            })
        })
        .map(str::to_string)
        .or_else(|| {
            palette
                .categories
                .iter()
                .flat_map(|category| category.nodes.iter())
                .next()
                .map(|node| node.descriptor_key.clone())
        });
    MaterialNodePickerViewModel {
        open,
        search_query: search_query.to_string(),
        highlighted_descriptor_key,
        categories: palette.categories,
    }
}

pub(super) fn first_palette_descriptor_key(search_query: &str) -> Option<String> {
    material_node_palette_view_model(search_query)
        .categories
        .into_iter()
        .flat_map(|category| category.nodes.into_iter())
        .next()
        .map(|node| node.descriptor_key)
}

pub(super) fn palette_contains_descriptor(search_query: &str, descriptor_key: &str) -> bool {
    material_node_palette_view_model(search_query)
        .categories
        .iter()
        .any(|category| {
            category
                .nodes
                .iter()
                .any(|node| node.descriptor_key == descriptor_key)
        })
}

pub(super) fn material_texture_resource_picker_view_model(
    catalog: &AssetCatalog,
    search_query: &str,
) -> MaterialTextureResourcePickerViewModel {
    let normalized = search_query.trim().to_ascii_lowercase();
    let options = catalog
        .assets()
        .filter_map(|record| {
            let artifact = record
                .artifact_ids
                .iter()
                .filter_map(|artifact_id| catalog.artifact(*artifact_id))
                .find_map(|artifact| {
                    let (descriptor, descriptor_hash, artifact_uri) = match &artifact.payload_kind {
                        ArtifactPayloadKind::TextureProduct {
                            descriptor,
                            descriptor_hash,
                            artifact_uri,
                        }
                        | ArtifactPayloadKind::GeneratedTextureProduct {
                            descriptor,
                            descriptor_hash,
                            artifact_uri,
                        } => (descriptor, descriptor_hash, artifact_uri),
                        _ => return None,
                    };
                    let resource_kind = match descriptor.dimension {
                        texture::TextureDimension::Texture2D => {
                            material_graph::MaterialResourceKind::Texture2D
                        }
                        texture::TextureDimension::Texture3DVolume => {
                            material_graph::MaterialResourceKind::Texture3D
                        }
                    };
                    let artifact_uri = artifact_uri.as_ref()?;
                    Some(MaterialTextureResourceOptionViewModel {
                        stable_id: record.stable_name.clone(),
                        display_name: record.display_name.clone(),
                        asset_id: record.asset_id,
                        artifact_id: artifact.artifact_id,
                        product_id: descriptor.product_id.raw(),
                        resource_kind,
                        descriptor_hash: descriptor_hash.clone(),
                        artifact_uri: artifact_uri.clone(),
                        valid: artifact.validity == ArtifactValidity::Valid,
                        diagnostic: (artifact.validity != ArtifactValidity::Valid)
                            .then(|| format!("artifact validity is {:?}", artifact.validity)),
                    })
                })?;
            if normalized.is_empty()
                || record
                    .stable_name
                    .to_ascii_lowercase()
                    .contains(&normalized)
                || record
                    .display_name
                    .to_ascii_lowercase()
                    .contains(&normalized)
            {
                Some(artifact)
            } else {
                None
            }
        })
        .collect();

    MaterialTextureResourcePickerViewModel {
        search_query: search_query.to_string(),
        options,
    }
}

pub(super) fn material_model_mesh_region_binding_view_model(
    catalog: &AssetCatalog,
    assignments: Option<&SceneMaterialAssignmentState>,
) -> Vec<MaterialModelMeshRegionBindingViewModel> {
    catalog_model_mesh_material_regions(catalog)
        .into_iter()
        .map(|region| {
            let valid = region.is_assignable();
            let diagnostic = region.display_diagnostic().or_else(|| {
                (region.artifact_validity != ArtifactValidity::Valid)
                    .then(|| format!("artifact validity is {:?}", region.artifact_validity))
            });
            let (assigned_slot_id, assigned_slot_label) = assignments
                .and_then(|assignments| {
                    let source_region = region.scene_material_region().ok()?;
                    let slot_id = assignments
                        .model_mesh_assignments()
                        .find(|assignment| assignment.material_region == source_region)?
                        .slot_id;
                    let label = assignments
                        .palette()
                        .slots
                        .iter()
                        .find(|slot| slot.slot_id == slot_id)
                        .map(|slot| slot.display_name.clone());
                    Some((Some(slot_id), label))
                })
                .unwrap_or((None, None));
            MaterialModelMeshRegionBindingViewModel {
                asset_id: region.asset_id,
                stable_name: region.asset_stable_name,
                asset_display_name: region.asset_display_name,
                artifact_id: region.artifact_id,
                source_id: region.source_id,
                source_revision_id: region.source_revision_id,
                source_revision: region.source_revision,
                material_region_key: region.material_region_key,
                material_region_label: region.display_name,
                assigned_slot_id,
                assigned_slot_label,
                valid,
                diagnostic,
            }
        })
        .collect()
}

pub(super) fn material_scene_material_slot_option_view_model(
    assignments: Option<&SceneMaterialAssignmentState>,
) -> Vec<MaterialSceneMaterialSlotOptionViewModel> {
    assignments
        .map(|assignments| {
            assignments
                .palette()
                .slots
                .iter()
                .map(|slot| MaterialSceneMaterialSlotOptionViewModel {
                    slot_id: slot.slot_id,
                    display_name: slot.display_name.clone(),
                    is_default: slot.is_default,
                })
                .collect()
        })
        .unwrap_or_default()
}
