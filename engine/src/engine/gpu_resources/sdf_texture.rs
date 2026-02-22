use wgpu::*;

pub struct SdfTexture {
	pub texture: Texture,
	pub view: TextureView,
	pub layout: BindGroupLayout,
	pub bind_group: BindGroup,
	pub dirty: bool,
	pub stride: usize,
	pub slot_index: u32,
}

impl SdfTexture {
	pub fn new(device: &Device, stride: usize, slot_index: u32) -> Self {
		let size = Extent3d {
			width: stride as u32,
			height: stride as u32,
			depth_or_array_layers: stride as u32,
		};

		let texture = device.create_texture(&TextureDescriptor {
			label: Some(&format!("texture{}", slot_index)),
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: TextureDimension::D3,
			format: TextureFormat::R32Float,
			usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
			view_formats: &[TextureFormat::R32Float],
		});

		let view = texture.create_view(&TextureViewDescriptor::default());

		let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			label: Some(&format!("texture{}_layout", slot_index)),
			entries: &[BindGroupLayoutEntry {
				binding: 0,
				visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT | ShaderStages::VERTEX,
				ty: BindingType::StorageTexture {
					access: StorageTextureAccess::ReadWrite,
					format: TextureFormat::R32Float,
					view_dimension: TextureViewDimension::D3,
				},
				count: None,
			}],
		});

		let bind_group = device.create_bind_group(&BindGroupDescriptor {
			label: Some(&format!("texture{}_bind_group", slot_index)),
			layout: &layout,
			entries: &[BindGroupEntry {
				binding: 0,
				resource: BindingResource::TextureView(&view),
			}],
		});

		Self {
			texture,
			view,
			layout,
			bind_group,
			dirty: true,
			stride,
			slot_index,
		}
	}

	pub fn update(&mut self, queue: &Queue, data: &[f32]) {
		assert_eq!(data.len(), self.stride * self.stride * self.stride);
		let size = Extent3d {
			width: self.stride as u32,
			height: self.stride as u32,
			depth_or_array_layers: self.stride as u32,
		};

		queue.write_texture(
			TexelCopyTextureInfoBase {
				texture: &self.texture,
				mip_level: 0,
				origin: Origin3d::ZERO,
				aspect: TextureAspect::All,
			},
			bytemuck::cast_slice(data),
			TexelCopyBufferLayout {
				bytes_per_row: Some((self.stride * size_of::<f32>()) as u32),
				rows_per_image: Some(self.stride as u32),
				offset: 0,
			},
			size,
		);
		self.dirty = false;
	}

	pub fn update_if_dirty(&mut self, queue: &Queue, data: &[f32]) {
		if self.dirty {
			self.update(queue, data);
		}
	}

	pub fn bind_group(&self) -> &BindGroup {
		&self.bind_group
	}

	pub fn layout(&self) -> &BindGroupLayout {
		&self.layout
	}

	pub fn mark_dirty(&mut self) {
		self.dirty = true;
	}

	pub fn mark_clean(&mut self) {
		self.dirty = false;
	}
}