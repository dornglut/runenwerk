use std::num::NonZeroU64;
use wgpu::*;

pub struct VoxelBuffer {
	pub buffer: Buffer,
	pub layout: BindGroupLayout,
	pub bind_group: BindGroup,
	pub dirty: bool,
	pub stride: usize,
	pub slot_index: u32,
}

impl VoxelBuffer {
	pub fn new(device: &Device, stride: usize, slot_index: u32) -> Self {
		let size = (stride * stride * stride * size_of::<u32>()) as u64;

		let buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some(&format!("voxel_buffer_{}", slot_index)),
			size,
			usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
			mapped_at_creation: false,
		});

		let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			label: Some(&format!("voxel_buffer_{}_layout", slot_index)),
			entries: &[BindGroupLayoutEntry {
				binding: 0,
				visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
				ty: BindingType::Buffer {
					ty: BufferBindingType::Storage { read_only: false },
					has_dynamic_offset: false,
					min_binding_size: Some(NonZeroU64::new(size).unwrap()),
				},
				count: None,
			}],
		});

		let bind_group = device.create_bind_group(&BindGroupDescriptor {
			label: Some(&format!("voxel_buffer_{}_bind_group", slot_index)),
			layout: &layout,
			entries: &[BindGroupEntry {
				binding: 0,
				resource: buffer.as_entire_binding(),
			}],
		});

		Self {
			buffer,
			layout,
			bind_group,
			dirty: true,
			stride,
			slot_index,
		}
	}

	pub fn update(&mut self, queue: &Queue, data: &[u32]) {
		if self.dirty {
			queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
			self.dirty = false;
		}
	}

	pub fn mark_dirty(&mut self) {
		self.dirty = true;
	}

	pub fn mark_clean(&mut self) {
		self.dirty = false;
	}

	pub fn bind_group(&self) -> &BindGroup {
		&self.bind_group
	}

	pub fn layout(&self) -> &BindGroupLayout {
		&self.layout
	}
}