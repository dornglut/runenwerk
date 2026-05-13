use super::render_flow::{
    ResolvedColorTargetView, ResolvedDepthTargetView, ResolvedTextureRef, RuntimeResourceKey,
    RuntimeTextureView,
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
    pub texture: Texture,
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
    targets: BTreeMap<RenderDynamicTextureTargetKey, RendererDynamicTextureTarget>,
}

impl RendererDynamicTextureTargetCache {
    pub fn realize_for_frame(
        &mut self,
        device: &Device,
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
                let texture = device.create_texture(&TextureDescriptor {
                    label: Some(label.as_str()),
                    size: Extent3d {
                        width: descriptor.width.max(1),
                        height: descriptor.height.max(1),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: dynamic_format_to_wgpu(descriptor.format),
                    usage: dynamic_usage_to_wgpu(descriptor.usage),
                    view_formats: &[],
                });
                self.targets.insert(
                    descriptor.key.clone(),
                    RendererDynamicTextureTarget {
                        texture,
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

    pub fn apply_uploads(
        &mut self,
        queue: &Queue,
        uploads: &[RenderDynamicTextureUploadDescriptor],
    ) -> RendererDynamicTextureUploadReport {
        let mut report = RendererDynamicTextureUploadReport::default();
        for upload in uploads {
            match self.apply_upload(queue, upload) {
                Ok(()) => report.applied_count = report.applied_count.saturating_add(1),
                Err(message) => {
                    report.rejected_count = report.rejected_count.saturating_add(1);
                    report
                        .diagnostics
                        .push(RendererDynamicTextureUploadDiagnostic {
                            target_key: upload.target_key.clone(),
                            message,
                        });
                }
            }
        }
        report
    }

    fn apply_upload(
        &mut self,
        queue: &Queue,
        upload: &RenderDynamicTextureUploadDescriptor,
    ) -> std::result::Result<(), String> {
        upload.validate().map_err(|err| err.to_string())?;
        let target = self.targets.get_mut(&upload.target_key).ok_or_else(|| {
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
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &target.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: upload.origin_x,
                    y: upload.origin_y,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            &bytes,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(upload.width.saturating_mul(4).max(4)),
                rows_per_image: Some(upload.height.max(1)),
            },
            Extent3d {
                width: upload.width,
                height: upload.height,
                depth_or_array_layers: 1,
            },
        );
        target.uploaded_product_generation = Some(upload.product_generation);
        Ok(())
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
            texture: &target.texture,
            format: target.format,
            size: target.size,
            is_depth: target.descriptor.format.is_depth(),
            generation: Some(target.generation),
        })
    }

    pub fn color_target_view(
        &self,
        pass_id: RenderPassId,
        key: &RenderDynamicTextureTargetKey,
    ) -> Result<ResolvedColorTargetView<'static>> {
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
            view: RuntimeTextureView::Owned(
                target
                    .texture
                    .create_view(&TextureViewDescriptor::default()),
            ),
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
            view: target
                .texture
                .create_view(&TextureViewDescriptor::default()),
            format: target.format,
        })
    }

    pub fn ui_texture_view(&self, key: &RenderDynamicTextureTargetKey) -> Result<TextureView> {
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
        Ok(target
            .texture
            .create_view(&TextureViewDescriptor::default()))
    }
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
    for pixel in out.chunks_exact_mut(4) {
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

pub fn dynamic_usage_to_wgpu(usage: RenderTextureTargetUsage) -> TextureUsages {
    let mut out = TextureUsages::empty();
    if usage.color_attachment || usage.depth_attachment {
        out |= TextureUsages::RENDER_ATTACHMENT;
    }
    if usage.sampled {
        out |= TextureUsages::TEXTURE_BINDING;
    }
    if usage.storage {
        out |= TextureUsages::STORAGE_BINDING;
    }
    if usage.copy_src {
        out |= TextureUsages::COPY_SRC;
    }
    if usage.copy_dst {
        out |= TextureUsages::COPY_DST;
    }
    out
}
