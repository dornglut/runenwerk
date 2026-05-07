use super::*;

impl FlowRuntimeResources {
    pub fn inspect_entries(&self, flow_id: RenderFlowId) -> Vec<RuntimeResourceInspectionEntry> {
        let mut entries = Vec::<RuntimeResourceInspectionEntry>::new();
        for (id, descriptor) in &self.descriptors {
            let lifetime = descriptor.lifetime();
            let imported = lifetime.is_imported();
            let kind = resource_kind_name(descriptor).to_string();

            let mut realized = false;
            let mut reuse = RuntimeResourceReuse::NotRealized;
            let mut size_bytes = None::<u64>;
            let mut texture_size = None::<(u32, u32)>;
            let mut element_count = None::<u64>;
            let mut generation = None::<u64>;

            match descriptor {
                RenderResourceDescriptor::UniformBuffer(_) => {
                    if let Some(buffer) = self.buffers.get(id) {
                        realized = true;
                        reuse = if buffer.reused_last_frame {
                            RuntimeResourceReuse::Reused
                        } else {
                            RuntimeResourceReuse::Created
                        };
                        size_bytes = Some(buffer.size);
                        element_count = Some(1);
                        generation = Some(buffer.generation);
                    }
                }
                RenderResourceDescriptor::StorageBuffer(value) => {
                    if let Some(buffer) = self.buffers.get(id) {
                        realized = true;
                        reuse = if buffer.reused_last_frame {
                            RuntimeResourceReuse::Reused
                        } else {
                            RuntimeResourceReuse::Created
                        };
                        size_bytes = Some(buffer.size);
                        element_count = Some(value.element_count);
                        generation = Some(buffer.generation);
                    } else {
                        element_count = Some(value.element_count);
                    }
                }
                RenderResourceDescriptor::SampledTexture(_)
                | RenderResourceDescriptor::StorageTexture(_)
                | RenderResourceDescriptor::ColorTarget(_)
                | RenderResourceDescriptor::DepthTarget(_)
                | RenderResourceDescriptor::HistoryTexture(_) => {
                    if let Some(texture) = self.textures.get(id) {
                        realized = true;
                        reuse = if texture.reused_last_frame {
                            RuntimeResourceReuse::Reused
                        } else {
                            RuntimeResourceReuse::Created
                        };
                        texture_size = Some(texture.size);
                        generation = Some(texture.generation);
                    }
                }
                RenderResourceDescriptor::TargetAlias(_)
                | RenderResourceDescriptor::ImportedTexture(_)
                | RenderResourceDescriptor::ImportedBuffer(_) => {}
            }

            entries.push(RuntimeResourceInspectionEntry {
                flow_id: flow_id.to_string(),
                id: id.to_string(),
                kind,
                lifetime,
                imported,
                realized,
                reuse,
                size_bytes,
                texture_size,
                element_count,
                generation,
            });
        }
        for ((invocation_id, id), buffer) in &self.invocation_uniform_buffers {
            let Some(descriptor) = self.descriptors.get(id) else {
                continue;
            };
            if !matches!(descriptor, RenderResourceDescriptor::UniformBuffer(_)) {
                continue;
            }
            entries.push(RuntimeResourceInspectionEntry {
                flow_id: flow_id.to_string(),
                id: RuntimeResourceKey::InvocationUniform {
                    invocation_id: invocation_id.clone(),
                    resource_id: *id,
                }
                .to_string(),
                kind: resource_kind_name(descriptor).to_string(),
                lifetime: descriptor.lifetime(),
                imported: false,
                realized: true,
                reuse: if buffer.reused_last_frame {
                    RuntimeResourceReuse::Reused
                } else {
                    RuntimeResourceReuse::Created
                },
                size_bytes: Some(buffer.size),
                texture_size: None,
                element_count: Some(1),
                generation: Some(buffer.generation),
            });
        }
        entries
    }
}
