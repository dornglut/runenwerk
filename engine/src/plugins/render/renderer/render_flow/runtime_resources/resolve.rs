use super::*;

impl FlowRuntimeResources {
    fn resolve_resource_key_from_input(&self, value: &str) -> Option<RuntimeResourceKey> {
        if value == SURFACE_COLOR_RESOURCE_LABEL {
            return Some(RuntimeResourceKey::SurfaceColor);
        }
        if value == SURFACE_DEPTH_RESOURCE_LABEL {
            return Some(RuntimeResourceKey::SurfaceDepth);
        }

        if let Some(id) = self.resource_ids_by_label.get(value) {
            return Some(RuntimeResourceKey::FlowOwned(*id));
        }

        value
            .parse::<u64>()
            .ok()
            .map(RenderResourceId::new)
            .map(RuntimeResourceKey::FlowOwned)
    }

    fn kind_of_key(&self, key: RuntimeResourceKey) -> Option<RuntimeResourceKind> {
        match key {
            RuntimeResourceKey::FlowOwned(id) => self.kinds.get(&id).copied(),
            RuntimeResourceKey::SurfaceColor => Some(RuntimeResourceKind::TextureLike),
            RuntimeResourceKey::SurfaceDepth => None,
        }
    }

    fn descriptor_for_key(&self, key: RuntimeResourceKey) -> Option<&RenderResourceDescriptor> {
        match key {
            RuntimeResourceKey::FlowOwned(id) => self.descriptors.get(&id),
            RuntimeResourceKey::SurfaceColor | RuntimeResourceKey::SurfaceDepth => None,
        }
    }

    #[cfg(test)]
    pub fn kind_of(&self, id: &str) -> Option<RuntimeResourceKind> {
        self.resolve_resource_key_from_input(id)
            .and_then(|key| self.kind_of_key(key))
    }

    pub fn kind_of_resource(
        &self,
        resource_key: RuntimeResourceKey,
    ) -> Option<RuntimeResourceKind> {
        self.kind_of_key(resource_key)
    }

    pub fn capture_texture_class(
        &self,
        resource_id: &str,
        fallback_class: CaptureTextureClass,
    ) -> CaptureTextureClass {
        let Some(resource_key) = self.resolve_resource_key_from_input(resource_id) else {
            return fallback_class;
        };

        match resource_key {
            RuntimeResourceKey::SurfaceColor | RuntimeResourceKey::SurfaceDepth => {
                CaptureTextureClass::ImportedTexture
            }
            RuntimeResourceKey::FlowOwned(_) => {
                let Some(descriptor) = self.descriptor_for_key(resource_key) else {
                    return fallback_class;
                };
                match descriptor {
                    RenderResourceDescriptor::DepthTarget(_) => CaptureTextureClass::DepthTarget,
                    RenderResourceDescriptor::HistoryTexture(_) => {
                        CaptureTextureClass::HistoryTexture
                    }
                    RenderResourceDescriptor::ImportedTexture(_) => {
                        CaptureTextureClass::ImportedTexture
                    }
                    RenderResourceDescriptor::SampledTexture(_)
                    | RenderResourceDescriptor::StorageTexture(_)
                    | RenderResourceDescriptor::ColorTarget(_) => CaptureTextureClass::ColorTarget,
                    RenderResourceDescriptor::UniformBuffer(_)
                    | RenderResourceDescriptor::StorageBuffer(_)
                    | RenderResourceDescriptor::ImportedBuffer(_) => fallback_class,
                }
            }
        }
    }

    pub fn resolve_resource_key(
        &self,
        pass_id: RenderPassId,
        resource: &CompiledResourceRef,
        _role: &str,
    ) -> Result<RuntimeResourceKey> {
        match resource {
            CompiledResourceRef::FlowOwned(id) | CompiledResourceRef::Imported(id) => {
                Ok(RuntimeResourceKey::FlowOwned(*id))
            }
            CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::SurfaceColor) => {
                Ok(RuntimeResourceKey::SurfaceColor)
            }
            CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::SurfaceDepth) => {
                bail!(
                    "pass '{}' references imported builtin resource '{}' but surface-depth imports are not available in runtime execution yet; use flow-owned depth targets",
                    pass_id,
                    SURFACE_DEPTH_RESOURCE_LABEL
                )
            }
        }
    }

    pub fn resolve_color_target_from_plan<'a>(
        &'a self,
        pass_id: RenderPassId,
        targets: &CompiledTargetPlan,
        frame_view: &'a TextureView,
        frame_format: TextureFormat,
    ) -> Result<ResolvedColorTargetView<'a>> {
        if targets.color_outputs.len() != 1 {
            bail!(
                "pass '{}' declares {} color outputs, but runtime execution currently requires exactly one color output",
                pass_id,
                targets.color_outputs.len()
            );
        }

        let output = targets.color_outputs.first().ok_or_else(|| {
            anyhow::anyhow!(
                "pass '{}' is missing a color output target in execution plan",
                pass_id
            )
        })?;
        let output_key = self.resolve_resource_key(pass_id, output, "color_output")?;

        if output_key == RuntimeResourceKey::SurfaceColor {
            return Ok(ResolvedColorTargetView {
                view: RuntimeTextureView::Borrowed(frame_view),
                format: frame_format,
            });
        }

        let kind = self.kind_of_key(output_key).ok_or_else(|| {
            anyhow::anyhow!(
                "pass '{}' writes unknown color target '{}' during runtime encoding",
                pass_id,
                output_key
            )
        })?;
        if matches!(kind, RuntimeResourceKind::BufferLike) {
            bail!(
                "pass '{}' color target '{}' is buffer-like",
                pass_id,
                output_key
            );
        }

        let RuntimeResourceKey::FlowOwned(output_id) = output_key else {
            bail!(
                "pass '{}' targets imported texture '{}', but only '{}' is currently supported as imported color target",
                pass_id,
                output_key,
                SURFACE_COLOR_RESOURCE_LABEL
            );
        };

        let Some(texture) = self.textures.get(&output_id) else {
            bail!(
                "pass '{}' targets imported texture '{}', but only '{}' is currently supported as imported color target",
                pass_id,
                output_key,
                SURFACE_COLOR_RESOURCE_LABEL
            );
        };
        if texture.is_depth {
            bail!(
                "pass '{}' color target '{}' is depth-only",
                pass_id,
                output_key
            );
        }

        Ok(ResolvedColorTargetView {
            view: RuntimeTextureView::Owned(
                texture
                    .texture
                    .create_view(&TextureViewDescriptor::default()),
            ),
            format: texture.format,
        })
    }

    pub fn resolve_depth_target_from_plan(
        &self,
        pass_id: RenderPassId,
        targets: &CompiledTargetPlan,
    ) -> Result<Option<ResolvedDepthTargetView>> {
        let Some(depth_target) = targets.depth_output.as_ref() else {
            return Ok(None);
        };
        let resource_key = self.resolve_resource_key(pass_id, depth_target, "depth_output")?;
        if resource_key == RuntimeResourceKey::SurfaceColor {
            bail!(
                "graphics pass '{}' uses '{}' as depth target, which is not supported",
                pass_id,
                SURFACE_COLOR_RESOURCE_LABEL
            );
        }

        let kind = self.kind_of_key(resource_key).ok_or_else(|| {
            anyhow::anyhow!(
                "graphics pass '{}' uses unknown depth target '{}'",
                pass_id,
                resource_key
            )
        })?;
        if matches!(kind, RuntimeResourceKind::BufferLike) {
            bail!(
                "graphics pass '{}' uses '{}' as depth target, but it is buffer-like",
                pass_id,
                resource_key
            );
        }

        let RuntimeResourceKey::FlowOwned(resource_id) = resource_key else {
            bail!(
                "graphics pass '{}' uses imported depth target '{}' but runtime currently supports only flow-owned depth targets",
                pass_id,
                resource_key
            );
        };

        let Some(texture) = self.textures.get(&resource_id) else {
            bail!(
                "graphics pass '{}' uses imported depth target '{}' but runtime currently supports only flow-owned depth targets",
                pass_id,
                resource_key
            );
        };
        if !texture.is_depth {
            bail!(
                "graphics pass '{}' uses '{}' as depth target, but it is not depth-capable",
                pass_id,
                resource_key
            );
        }

        Ok(Some(ResolvedDepthTargetView {
            view: texture
                .texture
                .create_view(&TextureViewDescriptor::default()),
            format: texture.format,
        }))
    }

    pub fn resolve_texture_ref<'a>(
        &'a self,
        pass_id: RenderPassId,
        resource: &CompiledResourceRef,
        frame_texture: &'a Texture,
        frame_size: (u32, u32),
        frame_format: TextureFormat,
    ) -> Result<ResolvedTextureRef<'a>> {
        let resource_key = self.resolve_resource_key(pass_id, resource, "texture")?;
        self.resolve_texture(
            pass_id,
            resource_key,
            frame_texture,
            frame_size,
            frame_format,
        )
    }

    pub fn resolve_storage_buffer_ref<'a>(
        &'a self,
        pass_id: RenderPassId,
        resource: &CompiledResourceRef,
    ) -> Result<ResolvedBufferRef<'a>> {
        let resource_key = self.resolve_resource_key(pass_id, resource, "storage_buffer")?;
        self.resolve_storage_buffer_for_pass_by_key(pass_id, resource_key)
    }

    pub fn resolve_texture_from_label<'a>(
        &'a self,
        pass_label: &str,
        resource_id: &str,
        frame_texture: &'a Texture,
        frame_size: (u32, u32),
        frame_format: TextureFormat,
    ) -> Result<ResolvedTextureRef<'a>> {
        let resource_key = self
            .resolve_resource_key_from_input(resource_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "pass '{}' references unknown resource '{}' during runtime encoding",
                    pass_label,
                    resource_id
                )
            })?;

        self.resolve_texture_with_label(
            pass_label,
            resource_key,
            frame_texture,
            frame_size,
            frame_format,
        )
    }

    pub fn resolve_texture<'a>(
        &'a self,
        pass_id: RenderPassId,
        resource_key: RuntimeResourceKey,
        frame_texture: &'a Texture,
        frame_size: (u32, u32),
        frame_format: TextureFormat,
    ) -> Result<ResolvedTextureRef<'a>> {
        let pass_label = pass_id.to_string();
        self.resolve_texture_with_label(
            pass_label.as_str(),
            resource_key,
            frame_texture,
            frame_size,
            frame_format,
        )
    }

    fn resolve_texture_with_label<'a>(
        &'a self,
        pass_label: &str,
        resource_key: RuntimeResourceKey,
        frame_texture: &'a Texture,
        frame_size: (u32, u32),
        frame_format: TextureFormat,
    ) -> Result<ResolvedTextureRef<'a>> {
        if resource_key == RuntimeResourceKey::SurfaceColor {
            return Ok(ResolvedTextureRef {
                id: resource_key,
                texture: frame_texture,
                format: frame_format,
                size: frame_size,
                is_depth: false,
                generation: None,
            });
        }

        let kind = self.kind_of_key(resource_key).ok_or_else(|| {
            anyhow::anyhow!(
                "pass '{}' references unknown resource '{}' during runtime encoding",
                pass_label,
                resource_key
            )
        })?;
        if matches!(kind, RuntimeResourceKind::BufferLike) {
            bail!(
                "pass '{}' references '{}' as a texture, but it is buffer-like",
                pass_label,
                resource_key
            );
        }

        let RuntimeResourceKey::FlowOwned(resource_id) = resource_key else {
            bail!(
                "pass '{}' references imported texture '{}' but only imported '{}' is supported in core runtime execution",
                pass_label,
                resource_key,
                SURFACE_COLOR_RESOURCE_LABEL
            );
        };

        let Some(texture) = self.textures.get(&resource_id) else {
            bail!(
                "pass '{}' references imported texture '{}' but only imported '{}' is supported in core runtime execution",
                pass_label,
                resource_key,
                SURFACE_COLOR_RESOURCE_LABEL
            );
        };

        Ok(ResolvedTextureRef {
            id: resource_key,
            texture: &texture.texture,
            format: texture.format,
            size: texture.size,
            is_depth: texture.is_depth,
            generation: Some(texture.generation),
        })
    }

    pub fn resolve_ui_texture_view(
        &self,
        pass_id: &str,
        resource_id: &str,
        frame_texture: &Texture,
        frame_size: (u32, u32),
        frame_format: TextureFormat,
    ) -> Result<TextureView> {
        let texture = self.resolve_texture_from_label(
            pass_id,
            resource_id,
            frame_texture,
            frame_size,
            frame_format,
        )?;
        if texture.is_depth {
            bail!(
                "ui composite cannot sample depth texture '{}' for pass '{}'",
                resource_id,
                pass_id
            );
        }
        Ok(texture
            .texture
            .create_view(&TextureViewDescriptor::default()))
    }

    pub fn resolve_buffer_key<'a>(
        &'a self,
        pass_id: RenderPassId,
        resource_key: RuntimeResourceKey,
    ) -> Result<ResolvedBufferRef<'a>> {
        let kind = self.kind_of_key(resource_key).ok_or_else(|| {
            anyhow::anyhow!(
                "pass '{}' references unknown resource '{}' during runtime encoding",
                pass_id,
                resource_key
            )
        })?;
        if matches!(kind, RuntimeResourceKind::TextureLike) {
            bail!(
                "pass '{}' references '{}' as a buffer, but it is texture-like",
                pass_id,
                resource_key
            );
        }

        let RuntimeResourceKey::FlowOwned(resource_id) = resource_key else {
            bail!(
                "pass '{}' references imported buffer '{}' but core runtime execution only supports flow-owned buffers",
                pass_id,
                resource_key
            );
        };

        let Some(buffer) = self.buffers.get(&resource_id) else {
            bail!(
                "pass '{}' references imported buffer '{}' but core runtime execution only supports flow-owned buffers",
                pass_id,
                resource_key
            );
        };

        Ok(ResolvedBufferRef {
            id: resource_key,
            buffer: &buffer.buffer,
            size: buffer.size,
            kind: buffer.kind,
            generation: Some(buffer.generation),
        })
    }

    fn resolve_flow_owned_buffer<'a>(
        &'a self,
        resource_id: RenderResourceId,
    ) -> Result<ResolvedBufferRef<'a>> {
        let kind = self.kinds.get(&resource_id).copied().ok_or_else(|| {
            anyhow::anyhow!(
                "uniform upload references unknown resource '{}' during runtime encoding",
                resource_id
            )
        })?;
        if matches!(kind, RuntimeResourceKind::TextureLike) {
            bail!(
                "uniform upload references '{}' as a buffer, but it is texture-like",
                resource_id
            );
        }

        let Some(buffer) = self.buffers.get(&resource_id) else {
            bail!(
                "uniform upload references imported buffer '{}' but core runtime execution only supports flow-owned buffers",
                resource_id
            );
        };

        Ok(ResolvedBufferRef {
            id: RuntimeResourceKey::FlowOwned(resource_id),
            buffer: &buffer.buffer,
            size: buffer.size,
            kind: buffer.kind,
            generation: Some(buffer.generation),
        })
    }

    pub fn resolve_uniform_buffer_for_upload<'a>(
        &'a self,
        resource_id: RenderResourceId,
    ) -> Result<ResolvedBufferRef<'a>> {
        let resolved = self.resolve_flow_owned_buffer(resource_id)?;
        if !matches!(resolved.kind, RuntimeBufferKind::Uniform) {
            bail!(
                "uniform upload binds '{}' as a uniform buffer but the resource is not uniform",
                resolved.id
            );
        }
        Ok(resolved)
    }

    pub fn resolve_uniform_buffer_for_pass<'a>(
        &'a self,
        pass_id: RenderPassId,
        resource_id: RenderResourceId,
    ) -> Result<ResolvedBufferRef<'a>> {
        let resolved =
            self.resolve_buffer_key(pass_id, RuntimeResourceKey::FlowOwned(resource_id))?;
        if !matches!(resolved.kind, RuntimeBufferKind::Uniform) {
            bail!(
                "pass '{}' binds '{}' as a uniform buffer but the resource is not uniform",
                pass_id,
                resolved.id
            );
        }
        Ok(resolved)
    }

    pub fn resolve_storage_buffer_for_pass_by_key<'a>(
        &'a self,
        pass_id: RenderPassId,
        resource_key: RuntimeResourceKey,
    ) -> Result<ResolvedBufferRef<'a>> {
        let resolved = self.resolve_buffer_key(pass_id, resource_key)?;
        if !matches!(resolved.kind, RuntimeBufferKind::Storage) {
            bail!(
                "pass '{}' binds '{}' as a storage buffer but the resource is not storage",
                pass_id,
                resolved.id
            );
        }
        Ok(resolved)
    }
}
