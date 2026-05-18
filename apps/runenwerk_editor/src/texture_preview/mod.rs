//! App-owned texture preview projection and runtime upload handoff.

use asset::{
    ArtifactPayloadKind, ArtifactValidity, AssetArtifactDescriptor, AssetCatalog, AssetId,
};
use editor_shell::{
    TexturePreviewChannelSelection, TextureSurfaceAction, TextureViewerSurfaceKind,
};
use engine::plugins::render::inspect::{
    TexturePreviewChannelMode, TexturePreviewUploadRequest, prepare_texture_preview_upload_proof,
};
use engine::plugins::render::{
    PreparedMaterialTextureBinding, PreparedMaterialTextureKind, RenderDynamicTextureRetention,
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetKey,
    RenderDynamicTextureUploadDescriptor, RenderTextureSampleMode, RenderTextureTargetUsage,
    RenderTextureUploadAlphaMode,
};
use texture::{
    TextureDescriptor, TextureDimension, TexturePreviewChannel, TexturePreviewDescriptor,
    ratify_texture_preview,
};
use ui_render_data::ProductSurfaceTextureBindingSource;

use crate::editor_app::RunenwerkEditorApp;

pub const TEXTURE_PREVIEW_DYNAMIC_NAMESPACE: &str = "runenwerk.editor.texture_preview";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TexturePreviewRuntime {
    texture2d: TexturePreviewControls,
    volume: TexturePreviewControls,
}

impl TexturePreviewRuntime {
    pub fn controls(&self, surface: TextureViewerSurfaceKind) -> TexturePreviewControls {
        match surface {
            TextureViewerSurfaceKind::Texture2D => self.texture2d,
            TextureViewerSurfaceKind::VolumeTexture3D => self.volume,
        }
    }

    pub fn apply_action(&mut self, action: TextureSurfaceAction) {
        match action {
            TextureSurfaceAction::SetPreviewMip { surface, mip_level } => {
                self.controls_mut(surface).mip_level = mip_level;
            }
            TextureSurfaceAction::SetPreviewSlice {
                surface,
                slice_index,
            } => {
                self.controls_mut(surface).slice_index = slice_index;
            }
            TextureSurfaceAction::SetPreviewChannel { surface, channel } => {
                self.controls_mut(surface).channel = channel;
            }
            TextureSurfaceAction::ResetPreview { surface } => {
                *self.controls_mut(surface) = TexturePreviewControls::default();
            }
        }
    }

    fn controls_mut(&mut self, surface: TextureViewerSurfaceKind) -> &mut TexturePreviewControls {
        match surface {
            TextureViewerSurfaceKind::Texture2D => &mut self.texture2d,
            TextureViewerSurfaceKind::VolumeTexture3D => &mut self.volume,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TexturePreviewControls {
    pub mip_level: u32,
    pub slice_index: u32,
    pub channel: TexturePreviewChannelSelection,
}

impl Default for TexturePreviewControls {
    fn default() -> Self {
        Self {
            mip_level: 0,
            slice_index: 0,
            channel: TexturePreviewChannelSelection::All,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexturePreviewViewModel {
    pub surface: TextureViewerSurfaceKind,
    pub descriptor: Option<TexturePreviewDescriptor>,
    pub product_surface: Option<TexturePreviewProductSurface>,
    pub proof: Option<TexturePreviewProofMetadata>,
    pub diagnostics: Vec<TexturePreviewDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexturePreviewProductSurface {
    pub source: ProductSurfaceTextureBindingSource,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexturePreviewPreparedUpload {
    pub target: RenderDynamicTextureTargetDescriptor,
    pub upload: RenderDynamicTextureUploadDescriptor,
    pub proof: TexturePreviewProofMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexturePreviewProofMetadata {
    pub texture_product_id: u64,
    pub descriptor_hash: String,
    pub artifact_uri: String,
    pub upload_format: String,
    pub mip_count: u32,
    pub selected_mip: u32,
    pub selected_slice: u32,
    pub selected_channel: String,
    pub sampler_identity: String,
    pub bind_group_identity: String,
    pub residency_state: String,
    pub residency_class: String,
    pub target_key: RenderDynamicTextureTargetKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexturePreviewDiagnostic {
    pub code: TexturePreviewDiagnosticCode,
    pub message: String,
}

impl TexturePreviewDiagnostic {
    pub fn new(code: TexturePreviewDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TexturePreviewDiagnosticCode {
    MissingTextureProduct,
    MissingArtifactUri,
    MissingArtifactPath,
    InvalidDescriptorHash,
    InvalidDescriptor,
    UnsupportedFormat,
    FailedUpload,
    MissingResidency,
    StaleArtifact,
    FailedPreservedArtifact,
    RejectedArtifact,
}

impl RunenwerkEditorApp {
    pub fn texture_preview_runtime(&self) -> &TexturePreviewRuntime {
        &self.texture_preview_runtime
    }

    pub fn texture_preview_runtime_mut(&mut self) -> &mut TexturePreviewRuntime {
        &mut self.texture_preview_runtime
    }

    pub fn apply_texture_surface_action(&mut self, action: TextureSurfaceAction) {
        self.texture_preview_runtime.apply_action(action);
    }
}

pub fn texture_preview_view_model(
    catalog: &AssetCatalog,
    selected_asset_id: Option<AssetId>,
    runtime: &TexturePreviewRuntime,
    surface: TextureViewerSurfaceKind,
) -> TexturePreviewViewModel {
    match prepare_texture_preview(catalog, selected_asset_id, runtime, surface) {
        Ok(prepared) => TexturePreviewViewModel {
            surface,
            descriptor: Some(preview_descriptor_from_proof(&prepared.proof)),
            product_surface: Some(TexturePreviewProductSurface {
                source: ProductSurfaceTextureBindingSource::dynamic_texture(
                    prepared.proof.target_key.namespace.clone(),
                    prepared.proof.target_key.target_id.clone(),
                ),
                width: prepared.upload.width,
                height: prepared.upload.height,
            }),
            proof: Some(prepared.proof),
            diagnostics: Vec::new(),
        },
        Err((descriptor, diagnostic)) => TexturePreviewViewModel {
            surface,
            descriptor,
            product_surface: None,
            proof: None,
            diagnostics: vec![diagnostic],
        },
    }
}

pub fn prepare_texture_preview(
    catalog: &AssetCatalog,
    selected_asset_id: Option<AssetId>,
    runtime: &TexturePreviewRuntime,
    surface: TextureViewerSurfaceKind,
) -> Result<
    TexturePreviewPreparedUpload,
    (Option<TexturePreviewDescriptor>, TexturePreviewDiagnostic),
> {
    let artifact =
        select_texture_artifact(catalog, selected_asset_id, surface).ok_or_else(|| {
            (
                None,
                TexturePreviewDiagnostic::new(
                    TexturePreviewDiagnosticCode::MissingTextureProduct,
                    "no catalog-backed texture product is available for this viewer",
                ),
            )
        })?;
    let (descriptor, descriptor_hash, artifact_uri) =
        payload_descriptor(artifact).map_err(|diagnostic| (None, diagnostic))?;
    let controls = runtime.controls(surface);
    let preview_descriptor = TexturePreviewDescriptor::new(descriptor.product_id)
        .with_mip_level(controls.mip_level)
        .with_slice_index(controls.slice_index)
        .with_channel(texture_channel(controls.channel));

    validate_artifact_state(artifact)
        .map_err(|diagnostic| (Some(preview_descriptor), diagnostic))?;
    if descriptor_hash != descriptor.descriptor_hash() {
        return Err((
            Some(preview_descriptor),
            TexturePreviewDiagnostic::new(
                TexturePreviewDiagnosticCode::InvalidDescriptorHash,
                format!(
                    "texture artifact {} descriptor hash '{}' does not match descriptor '{}'",
                    artifact.artifact_id.raw(),
                    descriptor_hash,
                    descriptor.descriptor_hash()
                ),
            ),
        ));
    }
    let report = ratify_texture_preview(descriptor, &preview_descriptor);
    if report.has_blocking_issues() {
        return Err((
            Some(preview_descriptor),
            TexturePreviewDiagnostic::new(
                TexturePreviewDiagnosticCode::InvalidDescriptor,
                format!(
                    "texture preview request failed ratification: {:?}",
                    report.issues()
                ),
            ),
        ));
    }
    let artifact_uri = artifact_uri.clone().ok_or_else(|| {
        (
            Some(preview_descriptor),
            TexturePreviewDiagnostic::new(
                TexturePreviewDiagnosticCode::MissingArtifactUri,
                format!(
                    "texture artifact {} has no artifact_uri; descriptor text is diagnostic only",
                    artifact.artifact_id.raw()
                ),
            ),
        )
    })?;
    let artifact_path = artifact.artifact_path.clone().ok_or_else(|| {
        (
            Some(preview_descriptor),
            TexturePreviewDiagnostic::new(
                TexturePreviewDiagnosticCode::MissingArtifactPath,
                format!(
                    "texture artifact {} has no artifact path for renderer residency",
                    artifact.artifact_id.raw()
                ),
            ),
        )
    })?;
    let target_key = texture_preview_target_key(surface, descriptor, controls);
    let sampler_identity = texture_sampler_identity(descriptor);
    let bind_group_identity = format!(
        "engine_ui_product_surface_bind_group:{}",
        target_key.label()
    );
    let binding = prepared_texture_binding(
        artifact,
        descriptor,
        artifact_path,
        sampler_identity.clone(),
    );
    let proof = prepare_texture_preview_upload_proof(&TexturePreviewUploadRequest {
        binding,
        selected_mip: controls.mip_level,
        selected_slice: controls.slice_index,
        selected_channel: engine_channel(controls.channel),
        sampler_identity: sampler_identity.clone(),
        bind_group_identity: bind_group_identity.clone(),
    })
    .map_err(|error| {
        (
            Some(preview_descriptor),
            TexturePreviewDiagnostic::new(
                TexturePreviewDiagnosticCode::FailedUpload,
                format!("texture preview GPU upload preparation failed: {error}"),
            ),
        )
    })?;

    let target = RenderDynamicTextureTargetDescriptor::new(
        target_key.clone(),
        proof.width,
        proof.height,
        proof.upload_format,
        RenderTextureTargetUsage::color_sampled(),
        RenderTextureSampleMode::FilterableFloat,
        RenderDynamicTextureRetention::RetainWhileRequested,
    );
    let upload = RenderDynamicTextureUploadDescriptor::rgba8_with_format(
        target_key.clone(),
        0,
        0,
        proof.width,
        proof.height,
        proof.upload_format,
        RenderTextureUploadAlphaMode::Straight,
        artifact.artifact_revision_id.raw(),
        proof.rgba8,
    );
    let metadata = TexturePreviewProofMetadata {
        texture_product_id: descriptor.product_id.raw(),
        descriptor_hash: descriptor_hash.clone(),
        artifact_uri,
        upload_format: format!("{:?}", proof.upload_format),
        mip_count: descriptor.mip_count,
        selected_mip: proof.selected_mip,
        selected_slice: proof.selected_slice,
        selected_channel: proof.selected_channel.as_label().to_string(),
        sampler_identity,
        bind_group_identity,
        residency_state: format!("{:?}", proof.residency_state),
        residency_class: proof.residency_class,
        target_key,
    };

    Ok(TexturePreviewPreparedUpload {
        target,
        upload,
        proof: metadata,
    })
}

fn preview_descriptor_from_proof(proof: &TexturePreviewProofMetadata) -> TexturePreviewDescriptor {
    TexturePreviewDescriptor::new(texture::TextureProductId::new(proof.texture_product_id))
        .with_mip_level(proof.selected_mip)
        .with_slice_index(proof.selected_slice)
        .with_channel(match proof.selected_channel.as_str() {
            "r" => TexturePreviewChannel::R,
            "g" => TexturePreviewChannel::G,
            "b" => TexturePreviewChannel::B,
            "a" => TexturePreviewChannel::A,
            _ => TexturePreviewChannel::All,
        })
}

fn select_texture_artifact(
    catalog: &AssetCatalog,
    selected_asset_id: Option<AssetId>,
    surface: TextureViewerSurfaceKind,
) -> Option<&AssetArtifactDescriptor> {
    if let Some(asset_id) = selected_asset_id {
        let selected_artifact = catalog.asset(asset_id).and_then(|record| {
            record
                .artifact_ids
                .iter()
                .filter_map(|artifact_id| catalog.artifact(*artifact_id))
                .find(|artifact| artifact_matches_surface(artifact, surface))
        });
        if selected_artifact.is_some() {
            return selected_artifact;
        }
    }

    catalog
        .artifacts
        .values()
        .find(|artifact| artifact_matches_surface(artifact, surface))
}

fn artifact_matches_surface(
    artifact: &AssetArtifactDescriptor,
    surface: TextureViewerSurfaceKind,
) -> bool {
    payload_dimension(artifact).is_some_and(|dimension| {
        matches!(
            (surface, dimension),
            (
                TextureViewerSurfaceKind::Texture2D,
                TextureDimension::Texture2D
            ) | (
                TextureViewerSurfaceKind::VolumeTexture3D,
                TextureDimension::Texture3DVolume
            )
        )
    })
}

fn payload_dimension(artifact: &AssetArtifactDescriptor) -> Option<TextureDimension> {
    match &artifact.payload_kind {
        ArtifactPayloadKind::TextureProduct { descriptor, .. }
        | ArtifactPayloadKind::GeneratedTextureProduct { descriptor, .. } => {
            Some(descriptor.dimension)
        }
        _ => None,
    }
}

fn payload_descriptor(
    artifact: &AssetArtifactDescriptor,
) -> Result<(&TextureDescriptor, &String, &Option<String>), TexturePreviewDiagnostic> {
    match &artifact.payload_kind {
        ArtifactPayloadKind::TextureProduct {
            descriptor,
            descriptor_hash,
            artifact_uri,
        }
        | ArtifactPayloadKind::GeneratedTextureProduct {
            descriptor,
            descriptor_hash,
            artifact_uri,
        } => Ok((descriptor, descriptor_hash, artifact_uri)),
        _ => Err(TexturePreviewDiagnostic::new(
            TexturePreviewDiagnosticCode::MissingTextureProduct,
            format!(
                "artifact {} does not carry TextureProduct or GeneratedTextureProduct payload",
                artifact.artifact_id.raw()
            ),
        )),
    }
}

fn validate_artifact_state(
    artifact: &AssetArtifactDescriptor,
) -> Result<(), TexturePreviewDiagnostic> {
    match artifact.validity {
        ArtifactValidity::Valid => Ok(()),
        ArtifactValidity::Stale => Err(TexturePreviewDiagnostic::new(
            TexturePreviewDiagnosticCode::StaleArtifact,
            format!(
                "texture artifact {} is stale; preview must not silently use stale artifact data",
                artifact.artifact_id.raw()
            ),
        )),
        ArtifactValidity::FailedPreserved => Err(TexturePreviewDiagnostic::new(
            TexturePreviewDiagnosticCode::FailedPreservedArtifact,
            format!(
                "texture artifact {} is failed-preserved; prior-valid preview requires explicit diagnostic",
                artifact.artifact_id.raw()
            ),
        )),
        ArtifactValidity::Rejected => Err(TexturePreviewDiagnostic::new(
            TexturePreviewDiagnosticCode::RejectedArtifact,
            format!(
                "texture artifact {} was rejected by ratification",
                artifact.artifact_id.raw()
            ),
        )),
    }
}

fn prepared_texture_binding(
    artifact: &AssetArtifactDescriptor,
    descriptor: &TextureDescriptor,
    artifact_path: String,
    sampler_identity: String,
) -> PreparedMaterialTextureBinding {
    let texture_kind = match descriptor.dimension {
        TextureDimension::Texture2D => PreparedMaterialTextureKind::Texture2D,
        TextureDimension::Texture3DVolume => PreparedMaterialTextureKind::Texture3D,
    };
    PreparedMaterialTextureBinding::new(
        0,
        "texture_preview",
        artifact.artifact_id.raw().to_string(),
        artifact_path,
        texture_kind,
        artifact.cache_key.as_str().to_string(),
    )
    .with_extent(
        descriptor.extent.width,
        descriptor.extent.height,
        descriptor.extent.depth,
    )
    .with_texture_dimension(format!("{:?}", descriptor.dimension))
    .with_residency_identity(texture_residency_identity(artifact, descriptor))
    .with_artifact_revision(artifact.artifact_revision_id.raw().to_string())
    .with_descriptor_hash(descriptor.descriptor_hash().to_string())
    .with_ktx2_contract(
        format!("{:?}", descriptor.ktx2_metadata().pixel_format),
        format!("{:?}", descriptor.ktx2_metadata().supercompression),
        descriptor.ktx2_metadata().byte_length,
    )
    .tap_sampler_policy(sampler_identity)
}

trait PreparedTextureBindingExt {
    fn tap_sampler_policy(self, sampler_identity: String) -> Self;
}

impl PreparedTextureBindingExt for PreparedMaterialTextureBinding {
    fn tap_sampler_policy(mut self, sampler_identity: String) -> Self {
        self.sampler_policy = sampler_identity;
        self
    }
}

fn texture_preview_target_key(
    surface: TextureViewerSurfaceKind,
    descriptor: &TextureDescriptor,
    controls: TexturePreviewControls,
) -> RenderDynamicTextureTargetKey {
    let surface_label = match surface {
        TextureViewerSurfaceKind::Texture2D => "texture2d",
        TextureViewerSurfaceKind::VolumeTexture3D => "texture3d",
    };
    RenderDynamicTextureTargetKey::new(
        TEXTURE_PREVIEW_DYNAMIC_NAMESPACE,
        format!(
            "{surface_label}.product{}.mip{}.slice{}.{}",
            descriptor.product_id.raw(),
            controls.mip_level,
            controls.slice_index,
            channel_label(controls.channel)
        ),
    )
}

fn texture_residency_identity(
    artifact: &AssetArtifactDescriptor,
    descriptor: &TextureDescriptor,
) -> String {
    format!(
        "ktx2:{}:{}:{}:{}",
        artifact.artifact_id.raw(),
        artifact.artifact_revision_id.raw(),
        artifact.cache_key.as_str(),
        descriptor.descriptor_hash()
    )
}

fn texture_sampler_identity(descriptor: &TextureDescriptor) -> String {
    format!(
        "min={:?};mag={:?};wrap_u={:?};wrap_v={:?};wrap_w={:?};aniso={}",
        descriptor.sampler.min_filter,
        descriptor.sampler.mag_filter,
        descriptor.sampler.wrap_u,
        descriptor.sampler.wrap_v,
        descriptor.sampler.wrap_w,
        descriptor.sampler.anisotropy
    )
}

fn texture_channel(channel: TexturePreviewChannelSelection) -> TexturePreviewChannel {
    match channel {
        TexturePreviewChannelSelection::All => TexturePreviewChannel::All,
        TexturePreviewChannelSelection::R => TexturePreviewChannel::R,
        TexturePreviewChannelSelection::G => TexturePreviewChannel::G,
        TexturePreviewChannelSelection::B => TexturePreviewChannel::B,
        TexturePreviewChannelSelection::A => TexturePreviewChannel::A,
    }
}

fn engine_channel(channel: TexturePreviewChannelSelection) -> TexturePreviewChannelMode {
    match channel {
        TexturePreviewChannelSelection::All => TexturePreviewChannelMode::All,
        TexturePreviewChannelSelection::R => TexturePreviewChannelMode::R,
        TexturePreviewChannelSelection::G => TexturePreviewChannelMode::G,
        TexturePreviewChannelSelection::B => TexturePreviewChannelMode::B,
        TexturePreviewChannelSelection::A => TexturePreviewChannelMode::A,
    }
}

fn channel_label(channel: TexturePreviewChannelSelection) -> &'static str {
    match channel {
        TexturePreviewChannelSelection::All => "all",
        TexturePreviewChannelSelection::R => "r",
        TexturePreviewChannelSelection::G => "g",
        TexturePreviewChannelSelection::B => "b",
        TexturePreviewChannelSelection::A => "a",
    }
}
