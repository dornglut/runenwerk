//! File: apps/runenwerk_editor/src/material_lab/resource_resolution.rs
//! Purpose: App-owned resolution from material resource refs to exact catalog artifacts.

use asset::{
    ArtifactPayloadKind, ArtifactValidity, AssetArtifactDescriptor, AssetArtifactId, AssetCatalog,
    AssetDiagnosticCode, AssetDiagnosticRecord, AssetDiagnosticSeverity, AssetKind,
};
use editor_shell::{
    MaterialDiagnosticSeverity, MaterialResourceBindingDiagnosticViewModel,
    MaterialResourceBindingStatusKind,
};
use material_graph::{MaterialIr, MaterialResourceBinding};
use resource_ref::ResourceRef;
use texture::{
    TextureContainerMetadata, TextureDescriptor, TextureDimension, TextureTranscodeStatus,
    ratify_texture_descriptor,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMaterialResource {
    pub node_id: graph::NodeId,
    pub binding_key: String,
    pub reference: ResourceRef,
    pub artifact_id: AssetArtifactId,
    pub artifact_path: String,
    pub kind: AssetKind,
    pub cache_key: asset::ArtifactCacheKey,
    pub descriptor: TextureDescriptor,
    pub artifact_revision: String,
    pub dimension: String,
    pub color_space: String,
    pub sampler_policy: String,
    pub residency_identity: String,
}

pub fn resolve_material_resources(
    catalog: &AssetCatalog,
    ir: &MaterialIr,
) -> Result<Vec<ResolvedMaterialResource>, Vec<AssetDiagnosticRecord>> {
    let mut resolved = Vec::new();
    let mut diagnostics = Vec::new();
    for binding in &ir.required_resources {
        match resolve_material_resource_binding(catalog, binding) {
            Ok(resource) => resolved.push(resource),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    if diagnostics.is_empty() {
        Ok(resolved)
    } else {
        Err(diagnostics)
    }
}

pub fn material_resource_binding_diagnostic_row(
    catalog: &AssetCatalog,
    binding: &MaterialResourceBinding,
) -> MaterialResourceBindingDiagnosticViewModel {
    let binding_label = format!(
        "node {} resource '{}'",
        binding.node_id.raw(),
        binding.binding_key
    );
    let resource_label = binding.reference.stable_id.as_str().to_string();
    let expected_kind = match expected_texture_kind(binding.reference.kind.as_str()) {
        Some(kind) => kind,
        None => {
            return material_resource_binding_row(
                MaterialResourceBindingStatusKind::Unsupported,
                "material.resource.unsupported_kind",
                binding_label,
                resource_label,
                Some(binding.reference.kind.as_str().to_string()),
                None,
                format!(
                    "resource kind '{}' is not supported by Material Lab texture bindings",
                    binding.reference.kind.as_str()
                ),
            );
        }
    };
    let expected_label = Some(asset_kind_label(expected_kind).to_string());
    let stable_id = binding.reference.stable_id.as_str();
    let matching_assets = catalog
        .assets()
        .filter(|record| record.stable_name == stable_id)
        .collect::<Vec<_>>();
    if matching_assets.is_empty() {
        return material_resource_binding_row(
            MaterialResourceBindingStatusKind::Missing,
            "material.resource.missing",
            binding_label,
            resource_label,
            expected_label,
            None,
            format!("texture asset '{stable_id}' is not present in the asset catalog"),
        );
    }
    if matching_assets.len() > 1 {
        return material_resource_binding_row(
            MaterialResourceBindingStatusKind::Ambiguous,
            "material.resource.ambiguous_asset",
            binding_label,
            resource_label,
            expected_label,
            None,
            format!("texture asset stable id '{stable_id}' resolves to multiple catalog assets"),
        );
    }

    let asset = matching_assets[0];
    if asset.kind != expected_kind {
        return material_resource_binding_row(
            MaterialResourceBindingStatusKind::Incompatible,
            "material.resource.incompatible_asset_kind",
            binding_label,
            resource_label,
            expected_label,
            None,
            format!(
                "texture asset '{stable_id}' is {:?}, expected {:?}",
                asset.kind, expected_kind
            ),
        );
    }

    let artifacts = asset
        .artifact_ids
        .iter()
        .filter_map(|artifact_id| catalog.artifact(*artifact_id))
        .collect::<Vec<_>>();
    let generated_candidates = artifacts
        .iter()
        .filter(|artifact| artifact.kind == expected_kind)
        .filter(|artifact| generated_texture_payload_matches(artifact, expected_kind))
        .collect::<Vec<_>>();
    let artifact_selector = binding
        .reference
        .artifact
        .as_ref()
        .map(|artifact| artifact.as_str());
    let candidates = artifacts
        .iter()
        .copied()
        .filter(|artifact| artifact.kind == expected_kind)
        .filter(|artifact| artifact.validity == ArtifactValidity::Valid)
        .filter(|artifact| texture_payload_matches(artifact, expected_kind))
        .filter(|artifact| {
            artifact_selector.is_none_or(|selector| artifact_matches_selector(artifact, selector))
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        if generated_candidates
            .iter()
            .any(|artifact| artifact.validity != ArtifactValidity::Valid)
        {
            return material_resource_binding_row(
                MaterialResourceBindingStatusKind::GeneratedUnavailable,
                "material.resource.generated_unavailable",
                binding_label,
                resource_label,
                expected_label,
                generated_candidates
                    .first()
                    .map(|artifact| artifact_label(artifact.artifact_id)),
                format!(
                    "generated texture asset '{stable_id}' exists but has no valid selected artifact"
                ),
            );
        }
        if artifacts
            .iter()
            .any(|artifact| artifact.kind == expected_kind && !texture_payload_is_texture(artifact))
        {
            return material_resource_binding_row(
                MaterialResourceBindingStatusKind::Unsupported,
                "material.resource.unsupported_payload",
                binding_label,
                resource_label,
                expected_label,
                None,
                format!("texture asset '{stable_id}' has no texture-product artifact payload"),
            );
        }
        if artifacts.iter().any(|artifact| texture_payload_is_texture(artifact)) {
            return material_resource_binding_row(
                MaterialResourceBindingStatusKind::Incompatible,
                "material.resource.incompatible_artifact",
                binding_label,
                resource_label,
                expected_label,
                artifacts.first().map(|artifact| artifact_label(artifact.artifact_id)),
                format!(
                    "texture asset '{stable_id}' has texture artifacts that do not match the expected kind"
                ),
            );
        }
        return material_resource_binding_row(
            MaterialResourceBindingStatusKind::Unresolved,
            "material.resource.unresolved",
            binding_label,
            resource_label,
            expected_label,
            None,
            format!(
                "texture asset '{stable_id}' has no exact valid {:?} artifact",
                expected_kind
            ),
        );
    }

    if candidates.len() > 1 {
        return material_resource_binding_row(
            MaterialResourceBindingStatusKind::Ambiguous,
            "material.resource.ambiguous_artifact",
            binding_label,
            resource_label,
            expected_label,
            None,
            format!(
                "texture asset '{stable_id}' resolves to multiple valid artifacts; add an artifact selector"
            ),
        );
    }

    let artifact = candidates[0];
    let resolved_artifact_label = Some(artifact_label(artifact.artifact_id));
    let Some(artifact_path) = artifact.artifact_path.as_deref() else {
        let status = if generated_texture_payload_matches(artifact, expected_kind) {
            MaterialResourceBindingStatusKind::GeneratedUnavailable
        } else {
            MaterialResourceBindingStatusKind::Unsupported
        };
        return material_resource_binding_row(
            status,
            "material.resource.artifact_path_missing",
            binding_label,
            resource_label,
            expected_label,
            resolved_artifact_label,
            format!(
                "texture artifact {} has no artifact path for KTX2 residency",
                artifact.artifact_id.raw()
            ),
        );
    };
    if !artifact_path.to_ascii_lowercase().ends_with(".ktx2") {
        return material_resource_binding_row(
            MaterialResourceBindingStatusKind::Unsupported,
            "material.resource.unsupported_artifact_path",
            binding_label,
            resource_label,
            expected_label,
            resolved_artifact_label,
            format!(
                "texture artifact {} is '{}', but Material Lab texture residency requires KTX2",
                artifact.artifact_id.raw(),
                artifact_path
            ),
        );
    }

    match texture_descriptor_for_artifact(artifact, expected_kind) {
        Ok(descriptor) => {
            let descriptor_report = ratify_texture_descriptor(&descriptor);
            if descriptor_report.has_blocking_issues() {
                return material_resource_binding_row(
                    MaterialResourceBindingStatusKind::Incompatible,
                    "material.resource.invalid_descriptor",
                    binding_label,
                    resource_label,
                    expected_label,
                    resolved_artifact_label,
                    format!(
                        "texture artifact {} has invalid descriptor issues: {:?}",
                        artifact.artifact_id.raw(),
                        descriptor_report.issues()
                    ),
                );
            }
            let ktx2 = descriptor.ktx2_metadata();
            if ktx2.byte_length.is_none() {
                return material_resource_binding_row(
                    MaterialResourceBindingStatusKind::Unsupported,
                    "material.resource.missing_ktx2_bytes",
                    binding_label,
                    resource_label,
                    expected_label,
                    resolved_artifact_label,
                    format!(
                        "texture artifact {} descriptor has no KTX2 byte length",
                        artifact.artifact_id.raw()
                    ),
                );
            }
            if matches!(ktx2.transcode_status, TextureTranscodeStatus::Unsupported) {
                return material_resource_binding_row(
                    MaterialResourceBindingStatusKind::Unsupported,
                    "material.resource.unsupported_transcode",
                    binding_label,
                    resource_label,
                    expected_label,
                    resolved_artifact_label,
                    format!(
                        "texture artifact {} marks KTX2 runtime transcode as unsupported",
                        artifact.artifact_id.raw()
                    ),
                );
            }
            let status = if generated_texture_payload_matches(artifact, expected_kind) {
                MaterialResourceBindingStatusKind::GeneratedAvailable
            } else {
                MaterialResourceBindingStatusKind::Resolved
            };
            material_resource_binding_row(
                status,
                "material.resource.resolved",
                binding_label,
                resource_label,
                expected_label,
                resolved_artifact_label,
                format!(
                    "texture resource resolved to artifact {}",
                    artifact.artifact_id.raw()
                ),
            )
        }
        Err(error) => material_resource_binding_row(
            MaterialResourceBindingStatusKind::Incompatible,
            "material.resource.incompatible_descriptor",
            binding_label,
            resource_label,
            expected_label,
            resolved_artifact_label,
            error.message,
        ),
    }
}

fn resolve_material_resource_binding(
    catalog: &AssetCatalog,
    binding: &MaterialResourceBinding,
) -> Result<ResolvedMaterialResource, AssetDiagnosticRecord> {
    let expected_kind =
        expected_texture_kind(binding.reference.kind.as_str()).ok_or_else(|| {
            diagnostic(format!(
                "material node {} resource '{}' uses unsupported resource kind '{}'",
                binding.node_id.raw(),
                binding.binding_key,
                binding.reference.kind.as_str()
            ))
        })?;
    let stable_id = binding.reference.stable_id.as_str();
    let mut matching_assets = catalog
        .assets()
        .filter(|record| record.stable_name == stable_id)
        .collect::<Vec<_>>();
    if matching_assets.is_empty() {
        return Err(diagnostic(format!(
            "material node {} resource '{}' references missing texture asset '{}'",
            binding.node_id.raw(),
            binding.binding_key,
            stable_id
        )));
    }
    if matching_assets.len() > 1 {
        return Err(diagnostic(format!(
            "material node {} resource '{}' ambiguously references texture asset '{}'",
            binding.node_id.raw(),
            binding.binding_key,
            stable_id
        )));
    }
    let asset = matching_assets.remove(0);
    if asset.kind != expected_kind {
        return Err(diagnostic(format!(
            "material node {} resource '{}' references asset '{}' as {:?}, expected {:?}",
            binding.node_id.raw(),
            binding.binding_key,
            stable_id,
            asset.kind,
            expected_kind
        )));
    }

    let artifact_selector = binding
        .reference
        .artifact
        .as_ref()
        .map(|artifact| artifact.as_str());
    let candidates = asset
        .artifact_ids
        .iter()
        .filter_map(|artifact_id| catalog.artifact(*artifact_id))
        .filter(|artifact| artifact.kind == expected_kind)
        .filter(|artifact| artifact.validity == ArtifactValidity::Valid)
        .filter(|artifact| texture_payload_matches(artifact, expected_kind))
        .filter(|artifact| {
            artifact_selector.is_none_or(|selector| artifact_matches_selector(artifact, selector))
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Err(diagnostic(format!(
            "material node {} resource '{}' references texture asset '{}' but no exact valid {:?} artifact is available",
            binding.node_id.raw(),
            binding.binding_key,
            stable_id,
            expected_kind
        )));
    }
    if candidates.len() > 1 {
        return Err(diagnostic(format!(
            "material node {} resource '{}' references texture asset '{}' but resolves to multiple valid artifacts; add an artifact selector",
            binding.node_id.raw(),
            binding.binding_key,
            stable_id
        )));
    }

    let artifact = candidates[0];
    let artifact_path = artifact.artifact_path.clone().ok_or_else(|| {
        diagnostic(format!(
            "material node {} resource '{}' resolved texture artifact {} without artifact path",
            binding.node_id.raw(),
            binding.binding_key,
            artifact.artifact_id.raw()
        ))
    })?;
    if !artifact_path.to_ascii_lowercase().ends_with(".ktx2") {
        return Err(diagnostic(format!(
            "material node {} resource '{}' resolved texture artifact {} to '{}', but material texture residency requires a KTX2 artifact",
            binding.node_id.raw(),
            binding.binding_key,
            artifact.artifact_id.raw(),
            artifact_path
        )));
    }
    let descriptor = texture_descriptor_for_artifact(artifact, expected_kind)?;
    let descriptor_report = ratify_texture_descriptor(&descriptor);
    if descriptor_report.has_blocking_issues() {
        return Err(diagnostic(format!(
            "material node {} resource '{}' resolved invalid texture descriptor for artifact {}: {:?}",
            binding.node_id.raw(),
            binding.binding_key,
            artifact.artifact_id.raw(),
            descriptor_report.issues()
        )));
    }
    let ktx2 = descriptor.ktx2_metadata();
    if ktx2.byte_length.is_none() {
        return Err(diagnostic(format!(
            "texture artifact {} descriptor has no KTX2 byte length; catalog texture residency requires validated artifact bytes",
            artifact.artifact_id.raw()
        )));
    }
    if matches!(ktx2.transcode_status, TextureTranscodeStatus::Unsupported) {
        return Err(diagnostic(format!(
            "texture artifact {} descriptor marks KTX2 runtime transcode as unsupported",
            artifact.artifact_id.raw()
        )));
    }

    Ok(ResolvedMaterialResource {
        node_id: binding.node_id,
        binding_key: binding.binding_key.clone(),
        reference: binding.reference.clone(),
        artifact_id: artifact.artifact_id,
        artifact_path,
        kind: artifact.kind,
        cache_key: artifact.cache_key.clone(),
        descriptor: descriptor.clone(),
        artifact_revision: artifact.artifact_revision_id.raw().to_string(),
        dimension: texture_dimension_descriptor_label(descriptor.dimension).to_string(),
        color_space: texture_color_space_label(descriptor.color_space).to_string(),
        sampler_policy: texture_sampler_policy_label(descriptor.sampler).to_string(),
        residency_identity: format!(
            "ktx2:{}:{}:{}:{}",
            artifact.artifact_id.raw(),
            artifact.artifact_revision_id.raw(),
            artifact.cache_key.as_str(),
            descriptor.descriptor_hash()
        ),
    })
}

fn material_resource_binding_row(
    status: MaterialResourceBindingStatusKind,
    code: impl Into<String>,
    binding_label: String,
    resource_key_or_slot_label: String,
    expected_kind_label: Option<String>,
    resolved_artifact_label: Option<String>,
    message: impl Into<String>,
) -> MaterialResourceBindingDiagnosticViewModel {
    MaterialResourceBindingDiagnosticViewModel {
        severity: resource_binding_severity(status),
        code: code.into(),
        binding_label,
        resource_key_or_slot_label,
        expected_kind_label,
        resolved_artifact_label,
        message: message.into(),
        status,
    }
}

fn resource_binding_severity(
    status: MaterialResourceBindingStatusKind,
) -> MaterialDiagnosticSeverity {
    match status {
        MaterialResourceBindingStatusKind::Resolved
        | MaterialResourceBindingStatusKind::GeneratedAvailable => MaterialDiagnosticSeverity::Info,
        MaterialResourceBindingStatusKind::GeneratedUnavailable
        | MaterialResourceBindingStatusKind::Unknown => MaterialDiagnosticSeverity::Warning,
        MaterialResourceBindingStatusKind::Missing
        | MaterialResourceBindingStatusKind::Ambiguous
        | MaterialResourceBindingStatusKind::Incompatible
        | MaterialResourceBindingStatusKind::Unsupported
        | MaterialResourceBindingStatusKind::Unresolved => MaterialDiagnosticSeverity::Error,
    }
}

fn asset_kind_label(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Texture2D => "texture_2d",
        AssetKind::Texture3DVolume => "texture_3d",
        _ => "unsupported_texture_kind",
    }
}

fn artifact_label(artifact_id: AssetArtifactId) -> String {
    format!("artifact {}", artifact_id.raw())
}

fn texture_descriptor_for_artifact(
    artifact: &AssetArtifactDescriptor,
    expected_kind: AssetKind,
) -> Result<TextureDescriptor, AssetDiagnosticRecord> {
    // catalog-persisted texture descriptor metadata is the only accepted
    // material resource truth; this resolver must never fabricate extents,
    // formats, or sampler policy from artifact ids.
    let descriptor = match &artifact.payload_kind {
        ArtifactPayloadKind::TextureProduct {
            descriptor,
            descriptor_hash,
            ..
        }
        | ArtifactPayloadKind::GeneratedTextureProduct {
            descriptor,
            descriptor_hash,
            ..
        } => {
            if descriptor_hash != descriptor.descriptor_hash() {
                return Err(diagnostic(format!(
                    "texture artifact {} descriptor hash '{}' does not match descriptor '{}'",
                    artifact.artifact_id.raw(),
                    descriptor_hash,
                    descriptor.descriptor_hash()
                )));
            }
            descriptor.clone()
        }
        _ => {
            return Err(diagnostic(format!(
                "texture artifact {} payload does not declare a texture product",
                artifact.artifact_id.raw()
            )));
        }
    };
    if !descriptor_dimension_matches_kind(descriptor.dimension, expected_kind) {
        return Err(diagnostic(format!(
            "texture artifact {} descriptor dimension {:?} does not match expected {:?}",
            artifact.artifact_id.raw(),
            descriptor.dimension,
            expected_kind
        )));
    }
    match &descriptor.container {
        TextureContainerMetadata::Ktx2(_) => {}
    }
    Ok(descriptor)
}

fn texture_dimension_descriptor_label(dimension: TextureDimension) -> &'static str {
    match dimension {
        TextureDimension::Texture2D => "texture_2d",
        TextureDimension::Texture3DVolume => "texture_3d",
    }
}

fn texture_color_space_label(color_space: texture::TextureColorSpace) -> &'static str {
    match color_space {
        texture::TextureColorSpace::Linear => "linear",
        texture::TextureColorSpace::Srgb => "srgb",
        texture::TextureColorSpace::Data => "data",
    }
}

fn texture_sampler_policy_label(sampler: texture::SamplerDescriptor) -> String {
    format!(
        "min={:?};mag={:?};wrap_u={:?};wrap_v={:?};wrap_w={:?};aniso={}",
        sampler.min_filter,
        sampler.mag_filter,
        sampler.wrap_u,
        sampler.wrap_v,
        sampler.wrap_w,
        sampler.anisotropy
    )
}

fn expected_texture_kind(kind: &str) -> Option<AssetKind> {
    match kind {
        "asset.catalog.texture2d" | "asset.catalog.texture_2d" | "texture2d" | "texture_2d" => {
            Some(AssetKind::Texture2D)
        }
        "asset.catalog.texture3d" | "asset.catalog.texture_3d" | "texture3d" | "texture_3d" => {
            Some(AssetKind::Texture3DVolume)
        }
        _ => None,
    }
}

fn texture_payload_matches(artifact: &AssetArtifactDescriptor, expected_kind: AssetKind) -> bool {
    match (&artifact.payload_kind, expected_kind) {
        (ArtifactPayloadKind::TextureProduct { descriptor, .. }, kind)
        | (ArtifactPayloadKind::GeneratedTextureProduct { descriptor, .. }, kind) => {
            descriptor_dimension_matches_kind(descriptor.dimension, kind)
        }
        _ => false,
    }
}

fn generated_texture_payload_matches(
    artifact: &AssetArtifactDescriptor,
    expected_kind: AssetKind,
) -> bool {
    match (&artifact.payload_kind, expected_kind) {
        (ArtifactPayloadKind::GeneratedTextureProduct { descriptor, .. }, kind) => {
            descriptor_dimension_matches_kind(descriptor.dimension, kind)
        }
        _ => false,
    }
}

fn texture_payload_is_texture(artifact: &AssetArtifactDescriptor) -> bool {
    matches!(
        artifact.payload_kind,
        ArtifactPayloadKind::TextureProduct { .. }
            | ArtifactPayloadKind::GeneratedTextureProduct { .. }
    )
}

fn descriptor_dimension_matches_kind(dimension: TextureDimension, kind: AssetKind) -> bool {
    matches!(
        (dimension, kind),
        (TextureDimension::Texture2D, AssetKind::Texture2D)
            | (
                TextureDimension::Texture3DVolume,
                AssetKind::Texture3DVolume
            )
    )
}

fn artifact_matches_selector(artifact: &AssetArtifactDescriptor, selector: &str) -> bool {
    selector == artifact.artifact_id.raw().to_string()
        || selector == artifact.cache_key.as_str()
        || artifact
            .artifact_path
            .as_ref()
            .is_some_and(|path| selector == path)
}

fn diagnostic(message: impl Into<String>) -> AssetDiagnosticRecord {
    AssetDiagnosticRecord::new(
        AssetDiagnosticCode::RatificationRejected,
        AssetDiagnosticSeverity::Error,
        message,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{ArtifactCacheKey, AssetRecord, asset_artifact_id, asset_id};
    use material_graph::MaterialResourceBinding;
    use resource_ref::ResourceRef;
    use texture::{Ktx2TextureMetadata, TextureExtent, TexturePixelFormat, TextureProductId};

    fn descriptor(
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

    fn texture_payload(descriptor: TextureDescriptor) -> ArtifactPayloadKind {
        ArtifactPayloadKind::TextureProduct {
            descriptor_hash: descriptor.descriptor_hash().to_string(),
            descriptor,
            artifact_uri: None,
        }
    }

    #[test]
    fn resolves_exact_valid_texture_artifact_by_asset_stable_name() {
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(1),
            "rock.albedo",
            "Rock Albedo",
            AssetKind::Texture2D,
        ));
        catalog.insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(7),
                asset_id(1),
                AssetKind::Texture2D,
                texture_payload(descriptor(
                    7,
                    TextureDimension::Texture2D,
                    TextureExtent::new(512, 512, 1),
                )),
                ArtifactCacheKey::new("texture-cache"),
            )
            .with_artifact_path(".runenwerk/artifacts/texture-7.ktx2"),
        );
        let binding = MaterialResourceBinding::new(
            graph::NodeId::new(4),
            "texture_ref",
            ResourceRef::new("asset.catalog.texture2d", "rock.albedo").expect("ref"),
        );

        let resolved = resolve_material_resource_binding(&catalog, &binding).expect("resolved");

        assert_eq!(resolved.artifact_id, asset_artifact_id(7));
        assert_eq!(resolved.kind, AssetKind::Texture2D);
        assert_eq!(resolved.descriptor.dimension, TextureDimension::Texture2D);
        assert_eq!(resolved.color_space, "linear");
    }

    #[test]
    fn rejects_ambiguous_texture_artifacts_without_selector() {
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(1),
            "rock.albedo",
            "Rock Albedo",
            AssetKind::Texture2D,
        ));
        for artifact_id in [asset_artifact_id(7), asset_artifact_id(8)] {
            catalog.insert_artifact(
                AssetArtifactDescriptor::new(
                    artifact_id,
                    asset_id(1),
                    AssetKind::Texture2D,
                    texture_payload(descriptor(
                        artifact_id.raw(),
                        TextureDimension::Texture2D,
                        TextureExtent::new(512, 512, 1),
                    )),
                    ArtifactCacheKey::new(format!("texture-cache-{}", artifact_id.raw())),
                )
                .with_artifact_path(format!(
                    ".runenwerk/artifacts/texture-{}.ktx2",
                    artifact_id.raw()
                )),
            );
        }
        let binding = MaterialResourceBinding::new(
            graph::NodeId::new(4),
            "texture_ref",
            ResourceRef::new("asset.catalog.texture2d", "rock.albedo").expect("ref"),
        );

        let error = resolve_material_resource_binding(&catalog, &binding).expect_err("ambiguous");

        assert!(error.message.contains("multiple valid artifacts"));
    }

    #[test]
    fn selector_picks_exact_texture_artifact() {
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(1),
            "rock.albedo",
            "Rock Albedo",
            AssetKind::Texture2D,
        ));
        for artifact_id in [asset_artifact_id(7), asset_artifact_id(8)] {
            catalog.insert_artifact(
                AssetArtifactDescriptor::new(
                    artifact_id,
                    asset_id(1),
                    AssetKind::Texture2D,
                    texture_payload(descriptor(
                        artifact_id.raw(),
                        TextureDimension::Texture2D,
                        TextureExtent::new(512, 512, 1),
                    )),
                    ArtifactCacheKey::new(format!("texture-cache-{}", artifact_id.raw())),
                )
                .with_artifact_path(format!(
                    ".runenwerk/artifacts/texture-{}.ktx2",
                    artifact_id.raw()
                )),
            );
        }
        let binding = MaterialResourceBinding::new(
            graph::NodeId::new(4),
            "texture_ref",
            ResourceRef::new("asset.catalog.texture2d", "rock.albedo")
                .expect("ref")
                .with_artifact("8"),
        );

        let resolved = resolve_material_resource_binding(&catalog, &binding).expect("resolved");

        assert_eq!(resolved.artifact_id, asset_artifact_id(8));
    }

    #[test]
    fn unsupported_texture_artifact_reports_binding_diagnostic() {
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(1),
            "rock.albedo",
            "Rock Albedo",
            AssetKind::Texture2D,
        ));
        catalog.insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(7),
                asset_id(1),
                AssetKind::Texture2D,
                ArtifactPayloadKind::DiagnosticCapture,
                ArtifactCacheKey::new("texture-cache"),
            )
            .with_artifact_path(".runenwerk/artifacts/texture-7.ktx2"),
        );
        let binding = MaterialResourceBinding::new(
            graph::NodeId::new(4),
            "texture_ref",
            ResourceRef::new("asset.catalog.texture2d", "rock.albedo").expect("ref"),
        );

        let row = material_resource_binding_diagnostic_row(&catalog, &binding);

        assert_eq!(row.status, MaterialResourceBindingStatusKind::Unsupported);
        assert_eq!(row.code, "material.resource.unsupported_payload");
        assert!(row.message.contains("texture-product artifact payload"));
    }

    #[test]
    fn resource_resolution_behavior_unchanged_by_diagnostic_dto_population() {
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(1),
            "rock.albedo",
            "Rock Albedo",
            AssetKind::Texture2D,
        ));
        catalog.insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(7),
                asset_id(1),
                AssetKind::Texture2D,
                texture_payload(descriptor(
                    7,
                    TextureDimension::Texture2D,
                    TextureExtent::new(512, 512, 1),
                )),
                ArtifactCacheKey::new("texture-cache"),
            )
            .with_artifact_path(".runenwerk/artifacts/texture-7.ktx2"),
        );
        let binding = MaterialResourceBinding::new(
            graph::NodeId::new(4),
            "texture_ref",
            ResourceRef::new("asset.catalog.texture2d", "rock.albedo").expect("ref"),
        );

        let before = resolve_material_resource_binding(&catalog, &binding).expect("resolved");
        let row = material_resource_binding_diagnostic_row(&catalog, &binding);
        let after = resolve_material_resource_binding(&catalog, &binding).expect("resolved");

        assert_eq!(row.status, MaterialResourceBindingStatusKind::Resolved);
        assert_eq!(before, after);
    }
}
