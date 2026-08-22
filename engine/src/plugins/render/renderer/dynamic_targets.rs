use super::render_flow::{
    ResolvedColorTargetView, ResolvedDepthTargetView, ResolvedTextureRef, RuntimeResourceKey,
    RuntimeTextureRef, RuntimeTextureView,
};
use super::resource_descriptors::{texture_descriptor, whole_texture_view_descriptor};
use crate::plugins::gpu::{
    CurrentRenderTextureUploadTerminal, GpuContext, GpuCopyExtent, GpuRealizedTexture,
    GpuRealizedTextureView, GpuResourceLifetime, GpuTextureAspect, GpuTextureCopyRegion,
    GpuTextureFormat, GpuTextureHandle, GpuTextureOrigin, GpuTextureUsage, GpuTextureViewHandle,
    GpuTransferRegion, GpuUploadOperation, GpuWorkResourceIdAllocator, PreparedGpuData,
    TransferData,
};
use crate::plugins::render::{
    RenderDynamicTextureRetention, RenderDynamicTextureTargetDescriptor,
    RenderDynamicTextureTargetKey, RenderDynamicTextureTargetSignature,
    RenderDynamicTextureUploadDescriptor, RenderPassId, RenderTextureSampleMode,
    RenderTextureTargetFormat, RenderTextureTargetUsage, RenderTextureUploadAlphaMode,
};
use anyhow::{Result, bail};
use std::collections::BTreeMap;
use wgpu::*;

#[derive(Debug)]
pub struct RendererDynamicTextureTarget {
    pub _handle: GpuTextureHandle,
    pub view_handle: GpuTextureViewHandle,
    pub realized: GpuRealizedTexture,
    pub realized_view: GpuRealizedTextureView,
    pub format: TextureFormat,
    pub size: (u32, u32),
    pub descriptor: RenderDynamicTextureTargetDescriptor,
    pub signature: RenderDynamicTextureTargetSignature,
    pub history_signature: Option<String>,
    pub generation: u64,
    pub uploaded_product_generation: Option<u64>,
    pub stale: bool,
    pub last_invalidation_reason: Option<String>,
    unrequested_frames: u32,
}

#[derive(Debug, Default)]
pub struct RendererDynamicTextureTargetCache {
    resource_ids: GpuWorkResourceIdAllocator,
    targets: BTreeMap<RenderDynamicTextureTargetKey, RendererDynamicTextureTarget>,
}

#[derive(Debug, Clone)]
struct RendererPreparedDynamicTextureUpload {
    target_key: RenderDynamicTextureTargetKey,
    product_generation: u64,
    operation: GpuUploadOperation,
    legacy_realized: GpuRealizedTexture,
}

#[derive(Debug, Default)]
pub(super) struct RendererPreparedDynamicTextureUploadBatch {
    uploads: Vec<RendererPreparedDynamicTextureUpload>,
    diagnostics: Vec<RendererDynamicTextureUploadDiagnostic>,
}

impl RendererDynamicTextureTargetCache {
    pub fn realize_for_frame(
        &mut self,
        context: &GpuContext,
        requests: &[RenderDynamicTextureTargetDescriptor],
        history_signatures: &BTreeMap<RenderDynamicTextureTargetKey, String>,
    ) -> Result<()> {
        let requested_keys = requests
            .iter()
            .map(|descriptor| descriptor.key.clone())
            .collect::<std::collections::BTreeSet<_>>();

        for descriptor in requests {
            descriptor.validate()?;
            let signature = descriptor.signature();
            let history_signature = history_signatures.get(&descriptor.key).cloned();
            let previous_generation = self
                .targets
                .get(&descriptor.key)
                .map(|target| target.generation)
                .unwrap_or(0);
            let invalidation_reason = self.targets.get(&descriptor.key).and_then(|target| {
                if target.signature != signature {
                    Some("descriptor_signature_changed".to_string())
                } else if target.history_signature != history_signature {
                    Some("history_signature_changed".to_string())
                } else {
                    None
                }
            });
            let should_recreate = self
                .targets
                .get(&descriptor.key)
                .map(|_| invalidation_reason.is_some())
                .unwrap_or(true);
            if should_recreate {
                let label = format!("dynamic_render_target_{}", descriptor.key);
                let handle = self
                    .resource_ids
                    .allocate_texture_handle(texture_descriptor(
                        label.clone(),
                        (descriptor.width.max(1), descriptor.height.max(1)),
                        dynamic_format_to_gpu(descriptor.format),
                        dynamic_usage_to_gpu(descriptor.usage),
                        GpuResourceLifetime::Retained,
                    )?)?;
                let realized = context.realize_texture(&handle)?;
                let view_handle = self.resource_ids.allocate_texture_view_handle(
                    whole_texture_view_descriptor(format!("{label}_view"), &handle)?,
                )?;
                let realized_view = context.realize_texture_view(&view_handle, &realized)?;
                self.targets.insert(
                    descriptor.key.clone(),
                    RendererDynamicTextureTarget {
                        _handle: handle,
                        view_handle,
                        realized,
                        realized_view,
                        format: dynamic_format_to_wgpu(descriptor.format),
                        size: (descriptor.width.max(1), descriptor.height.max(1)),
                        descriptor: descriptor.clone(),
                        signature,
                        history_signature,
                        generation: previous_generation.saturating_add(1),
                        uploaded_product_generation: None,
                        stale: false,
                        last_invalidation_reason: invalidation_reason,
                        unrequested_frames: 0,
                    },
                );
            } else if let Some(existing) = self.targets.get_mut(&descriptor.key) {
                existing.descriptor = descriptor.clone();
                existing.history_signature = history_signature;
                existing.stale = false;
                existing.last_invalidation_reason = None;
                existing.unrequested_frames = 0;
            }
        }

        self.targets.retain(|key, target| {
            if requested_keys.contains(key) {
                return true;
            }
            target.unrequested_frames = target.unrequested_frames.saturating_add(1);
            target.stale = true;
            target.last_invalidation_reason = Some("not_requested_this_frame".to_string());
            match target.descriptor.retention {
                RenderDynamicTextureRetention::RetainWhileRequested => false,
                RenderDynamicTextureRetention::RetainUntilViewportClose => true,
                RenderDynamicTextureRetention::RetainForFrames(frames) => {
                    target.unrequested_frames <= frames
                }
            }
        });

        Ok(())
    }

    pub(super) fn prepare_uploads(
        &self,
        uploads: &[RenderDynamicTextureUploadDescriptor],
    ) -> RendererPreparedDynamicTextureUploadBatch {
        let mut batch = RendererPreparedDynamicTextureUploadBatch::default();
        for upload in uploads {
            match self.prepare_upload(upload) {
                Ok(prepared) => batch.uploads.push(prepared),
                Err(message) => batch
                    .diagnostics
                    .push(RendererDynamicTextureUploadDiagnostic {
                        target_key: upload.target_key.clone(),
                        message,
                    }),
            }
        }
        batch
    }

    pub fn apply_uploads(
        &mut self,
        context: &GpuContext,
        queue: &Queue,
        uploads: &[RenderDynamicTextureUploadDescriptor],
    ) -> RendererDynamicTextureUploadReport {
        let prepared = self.prepare_uploads(uploads);
        self.apply_prepared_uploads(context, queue, prepared)
    }

    pub(super) fn apply_prepared_uploads(
        &mut self,
        context: &GpuContext,
        queue: &Queue,
        prepared: RendererPreparedDynamicTextureUploadBatch,
    ) -> RendererDynamicTextureUploadReport {
        let mut report = RendererDynamicTextureUploadReport {
            applied_count: 0,
            rejected_count: prepared.diagnostics.len(),
            diagnostics: prepared.diagnostics,
        };
        for upload in prepared.uploads {
            match apply_canonical_dynamic_texture_upload(
                context,
                queue,
                &upload.legacy_realized,
                &upload.operation,
            ) {
                Ok(()) => {
                    let Some(target) = self.targets.get_mut(&upload.target_key) else {
                        report.rejected_count = report.rejected_count.saturating_add(1);
                        report
                            .diagnostics
                            .push(RendererDynamicTextureUploadDiagnostic {
                                target_key: upload.target_key,
                                message: "prepared dynamic texture upload lost its target before physical application"
                                    .to_string(),
                            });
                        continue;
                    };
                    // The legacy path advances product-generation evidence only after the physical
                    // queue write succeeds. G5C1 must preserve the same acceptance boundary when
                    // this operation joins the frame submission: preparation alone must never
                    // advance this state.
                    target.uploaded_product_generation = Some(upload.product_generation);
                    report.applied_count = report.applied_count.saturating_add(1);
                }
                Err(message) => {
                    report.rejected_count = report.rejected_count.saturating_add(1);
                    report
                        .diagnostics
                        .push(RendererDynamicTextureUploadDiagnostic {
                            target_key: upload.target_key,
                            message,
                        });
                }
            }
        }
        report
    }

    fn prepare_upload(
        &self,
        upload: &RenderDynamicTextureUploadDescriptor,
    ) -> std::result::Result<RendererPreparedDynamicTextureUpload, String> {
        upload.validate().map_err(|err| err.to_string())?;
        let target = self.targets.get(&upload.target_key).ok_or_else(|| {
            format!(
                "dynamic texture upload references missing target '{}'",
                upload.target_key
            )
        })?;
        if !target.descriptor.usage.copy_dst {
            return Err(format!(
                "dynamic texture upload target '{}' is not copy-dst compatible",
                upload.target_key
            ));
        }
        if target.descriptor.format != upload.format {
            return Err(format!(
                "dynamic texture upload target '{}' format {:?} does not match upload format {:?}",
                upload.target_key, target.descriptor.format, upload.format
            ));
        }
        let end_x = upload
            .origin_x
            .checked_add(upload.width)
            .ok_or_else(|| "dynamic texture upload x range overflowed".to_string())?;
        let end_y = upload
            .origin_y
            .checked_add(upload.height)
            .ok_or_else(|| "dynamic texture upload y range overflowed".to_string())?;
        if end_x > target.size.0 || end_y > target.size.1 {
            return Err(format!(
                "dynamic texture upload target '{}' region {}x{} at {},{} exceeds target {}x{}",
                upload.target_key,
                upload.width,
                upload.height,
                upload.origin_x,
                upload.origin_y,
                target.size.0,
                target.size.1
            ));
        }

        let bytes = upload_bytes_for_gpu(upload);
        let operation = canonical_dynamic_texture_upload(target, upload, bytes)
            .map_err(|error| error.to_string())?;
        Ok(RendererPreparedDynamicTextureUpload {
            target_key: upload.target_key.clone(),
            product_generation: upload.product_generation,
            operation,
            legacy_realized: target.realized.clone(),
        })
    }

    pub fn texture_ref<'a>(
        &'a self,
        pass_id: RenderPassId,
        key: &RenderDynamicTextureTargetKey,
    ) -> Result<ResolvedTextureRef<'a>> {
        let target = self.targets.get(key).ok_or_else(|| {
            anyhow::anyhow!(
                "pass '{}' references missing dynamic texture target '{}'",
                pass_id,
                key
            )
        })?;
        Ok(ResolvedTextureRef {
            id: RuntimeResourceKey::DynamicTexture(key.clone()),
            texture: RuntimeTextureRef::Realized(&target.realized),
            view_handle: Some(&target.view_handle),
            format: target.format,
            size: target.size,
            is_depth: target.descriptor.format.is_depth(),
        })
    }

    pub fn color_target_view<'a>(
        &self,
        pass_id: RenderPassId,
        key: &RenderDynamicTextureTargetKey,
    ) -> Result<ResolvedColorTargetView<'a>> {
        let target = self.targets.get(key).ok_or_else(|| {
            anyhow::anyhow!(
                "pass '{}' writes missing dynamic color target '{}'",
                pass_id,
                key
            )
        })?;
        if target.descriptor.format.is_depth() || !target.descriptor.usage.color_attachment {
            bail!(
                "pass '{}' writes dynamic target '{}' as color, but descriptor is not color-attachment compatible",
                pass_id,
                key
            );
        }
        Ok(ResolvedColorTargetView {
            view: RuntimeTextureView::Realized(target.realized_view.clone()),
            format: target.format,
        })
    }

    pub fn depth_target_view(
        &self,
        pass_id: RenderPassId,
        key: &RenderDynamicTextureTargetKey,
    ) -> Result<ResolvedDepthTargetView> {
        let target = self.targets.get(key).ok_or_else(|| {
            anyhow::anyhow!(
                "pass '{}' writes missing dynamic depth target '{}'",
                pass_id,
                key
            )
        })?;
        if !target.descriptor.format.is_depth() || !target.descriptor.usage.depth_attachment {
            bail!(
                "pass '{}' writes dynamic target '{}' as depth, but descriptor is not depth-attachment compatible",
                pass_id,
                key
            );
        }
        Ok(ResolvedDepthTargetView {
            view: target.realized_view.clone(),
            format: target.format,
        })
    }

    pub fn ui_texture_view_handle(
        &self,
        key: &RenderDynamicTextureTargetKey,
    ) -> Result<GpuTextureViewHandle> {
        let target = self.targets.get(key).ok_or_else(|| {
            anyhow::anyhow!("ui viewport embed references missing dynamic texture '{key}'")
        })?;
        if !target.descriptor.usage.sampled
            || target.descriptor.sample_mode == RenderTextureSampleMode::NotSampled
        {
            bail!("ui viewport embed cannot sample non-sampleable dynamic texture '{key}'");
        }
        if !target.descriptor.format.is_displayable() {
            bail!("ui viewport embed cannot sample non-displayable dynamic texture '{key}'");
        }
        Ok(target.view_handle.clone())
    }
}

fn canonical_dynamic_texture_upload(
    target: &RendererDynamicTextureTarget,
    upload: &RenderDynamicTextureUploadDescriptor,
    bytes: Vec<u8>,
) -> std::result::Result<GpuUploadOperation, Box<dyn std::error::Error>> {
    let region = GpuTextureCopyRegion::new(
        &target._handle,
        0,
        GpuTextureOrigin::new(upload.origin_x, upload.origin_y, 0),
        GpuTextureAspect::Color,
        GpuCopyExtent::new(upload.width, upload.height, 1)?,
    )?;
    let common = target._handle.descriptor().common();
    let payload = PreparedGpuData::<TransferData>::from_pod_transfer(
        format!("dynamic texture upload {}", upload.target_key),
        bytes.as_slice(),
        common.provenance().clone(),
    )?;
    Ok(GpuUploadOperation::new(region.into(), payload)?)
}

fn apply_canonical_dynamic_texture_upload(
    context: &GpuContext,
    queue: &Queue,
    realized: &GpuRealizedTexture,
    operation: &GpuUploadOperation,
) -> std::result::Result<(), String> {
    let GpuTransferRegion::Texture(region) = operation.destination() else {
        return Err("dynamic texture upload lowered to a non-texture destination".to_string());
    };
    let extent = region.extent();
    let bytes_per_row = extent
        .width()
        .checked_mul(region.texture().descriptor().format().bytes_per_texel())
        .ok_or_else(|| "dynamic texture upload row byte length overflowed".to_string())?;
    let aspect = match region.aspect() {
        GpuTextureAspect::All | GpuTextureAspect::Color => TextureAspect::All,
        GpuTextureAspect::DepthOnly => TextureAspect::DepthOnly,
        GpuTextureAspect::StencilOnly => TextureAspect::StencilOnly,
    };
    let origin = region.origin();

    context
        .current_render_execution_bridge()
        .for_texture_upload(
            realized,
            UploadDynamicTexture {
                queue,
                bytes: operation.payload().as_bytes(),
                mip_level: region.mip_level(),
                origin: Origin3d {
                    x: origin.x(),
                    y: origin.y(),
                    z: origin.z(),
                },
                aspect,
                bytes_per_row,
                rows_per_image: extent.height(),
                size: Extent3d {
                    width: extent.width(),
                    height: extent.height(),
                    depth_or_array_layers: extent.depth_or_layers(),
                },
            },
        )
        .map_err(|error| error.to_string())
}

struct UploadDynamicTexture<'a> {
    queue: &'a Queue,
    bytes: &'a [u8],
    mip_level: u32,
    origin: Origin3d,
    aspect: TextureAspect,
    bytes_per_row: u32,
    rows_per_image: u32,
    size: Extent3d,
}

impl CurrentRenderTextureUploadTerminal for UploadDynamicTexture<'_> {
    fn upload_texture(self, texture: &Texture) {
        self.queue.write_texture(
            TexelCopyTextureInfo {
                texture,
                mip_level: self.mip_level,
                origin: self.origin,
                aspect: self.aspect,
            },
            self.bytes,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.bytes_per_row),
                rows_per_image: Some(self.rows_per_image),
            },
            self.size,
        );
    }
}

fn dynamic_format_to_gpu(format: RenderTextureTargetFormat) -> GpuTextureFormat {
    match format {
        RenderTextureTargetFormat::Rgba8Unorm => GpuTextureFormat::Rgba8Unorm,
        RenderTextureTargetFormat::Rgba8UnormSrgb => GpuTextureFormat::Rgba8UnormSrgb,
        RenderTextureTargetFormat::R32Uint => GpuTextureFormat::R32Uint,
        RenderTextureTargetFormat::Depth32Float => GpuTextureFormat::Depth32Float,
    }
}

fn dynamic_usage_to_gpu(usage: RenderTextureTargetUsage) -> Vec<GpuTextureUsage> {
    let mut out = Vec::new();
    if usage.sampled {
        out.push(GpuTextureUsage::Sampled);
    }
    if usage.storage {
        out.push(GpuTextureUsage::StorageWrite);
    }
    if usage.color_attachment {
        out.push(GpuTextureUsage::ColorAttachment);
    }
    if usage.depth_attachment {
        out.push(GpuTextureUsage::DepthStencilAttachment);
    }
    if usage.copy_src {
        out.push(GpuTextureUsage::CopySource);
    }
    if usage.copy_dst {
        out.push(GpuTextureUsage::CopyDestination);
    }
    out
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RendererDynamicTextureUploadReport {
    pub applied_count: usize,
    pub rejected_count: usize,
    pub diagnostics: Vec<RendererDynamicTextureUploadDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererDynamicTextureUploadDiagnostic {
    pub target_key: RenderDynamicTextureTargetKey,
    pub message: String,
}

fn upload_bytes_for_gpu(upload: &RenderDynamicTextureUploadDescriptor) -> Vec<u8> {
    match upload.alpha_mode {
        RenderTextureUploadAlphaMode::Straight => upload.rgba8.clone(),
        RenderTextureUploadAlphaMode::Premultiplied => unpremultiply_rgba8(&upload.rgba8),
    }
}

fn unpremultiply_rgba8(bytes: &[u8]) -> Vec<u8> {
    let mut out = bytes.to_vec();
    for pixel in out.as_chunks_mut::<4>().0 {
        let alpha = pixel[3];
        if alpha == 0 {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
            continue;
        }
        for channel in &mut pixel[..3] {
            let value = (u16::from(*channel) * 255 + u16::from(alpha) / 2) / u16::from(alpha);
            *channel = value.min(255) as u8;
        }
    }
    out
}

pub fn dynamic_format_to_wgpu(format: RenderTextureTargetFormat) -> TextureFormat {
    match format {
        RenderTextureTargetFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
        RenderTextureTargetFormat::Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
        RenderTextureTargetFormat::R32Uint => TextureFormat::R32Uint,
        RenderTextureTargetFormat::Depth32Float => TextureFormat::Depth32Float,
    }
}
