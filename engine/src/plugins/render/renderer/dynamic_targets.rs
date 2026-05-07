use super::render_flow::{
    ResolvedColorTargetView, ResolvedDepthTargetView, ResolvedTextureRef, RuntimeResourceKey,
    RuntimeTextureView,
};
use crate::plugins::render::{
    RenderDynamicTextureRetention, RenderDynamicTextureTargetDescriptor,
    RenderDynamicTextureTargetKey, RenderDynamicTextureTargetSignature, RenderPassId,
    RenderTextureSampleMode, RenderTextureTargetFormat, RenderTextureTargetUsage,
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
    pub generation: u64,
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
    ) -> Result<()> {
        let requested_keys = requests
            .iter()
            .map(|descriptor| descriptor.key.clone())
            .collect::<std::collections::BTreeSet<_>>();

        for descriptor in requests {
            descriptor.validate()?;
            let signature = descriptor.signature();
            let previous_generation = self
                .targets
                .get(&descriptor.key)
                .map(|target| target.generation)
                .unwrap_or(0);
            let should_recreate = self
                .targets
                .get(&descriptor.key)
                .map(|target| target.signature != signature)
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
                        generation: previous_generation.saturating_add(1),
                        stale: false,
                        last_invalidation_reason: should_recreate
                            .then_some("descriptor_signature_changed".to_string()),
                        unrequested_frames: 0,
                    },
                );
            } else if let Some(existing) = self.targets.get_mut(&descriptor.key) {
                existing.descriptor = descriptor.clone();
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

pub fn dynamic_format_to_wgpu(format: RenderTextureTargetFormat) -> TextureFormat {
    match format {
        RenderTextureTargetFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
        RenderTextureTargetFormat::Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
        RenderTextureTargetFormat::R32Uint => TextureFormat::R32Uint,
        RenderTextureTargetFormat::Depth32Float => TextureFormat::Depth32Float,
    }
}

fn dynamic_usage_to_wgpu(usage: RenderTextureTargetUsage) -> TextureUsages {
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
