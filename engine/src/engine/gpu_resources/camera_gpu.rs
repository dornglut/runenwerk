use crate::engine::entities::camera::Camera;
use bytemuck::{bytes_of, Pod, Zeroable};
use std::num::NonZeroU64;
use wgpu::util::align_to;
use wgpu::*;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
	pub position: [f32; 3], // Camera position
	pub _padding: f32,
	pub forward: [f32; 3],
	pub _padding2: f32,
	pub up: [f32; 3],
	pub _padding3: f32,
	pub right: [f32; 3],
	pub _padding4: f32,
	pub fov: f32,
	pub aspect_ratio: f32,
	pub near: f32,
	pub far: f32,
}


pub struct CameraGPU {
	pub buffer: Buffer,
	pub layout: BindGroupLayout,
	pub bind_group: BindGroup,
}

impl CameraGPU {
	pub fn new(device: &Device) -> Self {
		let size = size_of::<CameraUniform>() as u64; // 4x4 matrix of f32
		let aligned_size = align_to(size, 256); // Align to 256 bytes for uniform buffer

		let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			label: Some("camera_bind_group_layout"),
			entries: &[BindGroupLayoutEntry {
				binding: 0,
				visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
				ty: BindingType::Buffer {
					ty: BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: Some(NonZeroU64::new(aligned_size).unwrap()),
				},
				count: None,
			}],
		});

		let buffer = device.create_buffer(&BufferDescriptor {
			label: Some("camera_buffer"),
			size: aligned_size,
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let binding = device.create_bind_group(&BindGroupDescriptor {
			label: Some("camera_bind_group"),
			layout: &layout,
			entries: &[BindGroupEntry {
				binding: 0,
				resource: buffer.as_entire_binding(),
			}],
		});

		Self {
			buffer,
			layout,
			bind_group: binding,
		}
	}

	pub fn update(&self, queue: &Queue, camera: &Camera) {
		let uniform = CameraUniform {
			position: camera.position.to_array(),
			_padding: 0.0,
			forward: camera.forward.to_array(),
			_padding2: 0.0,
			up: camera.up.to_array(),
			_padding3: 0.0,
			right: camera.right.to_array(),
			_padding4: 0.0,
			fov: camera.fov.to_radians(),
			aspect_ratio: camera.aspect_ratio,
			near: camera.near,
			far: camera.far,
		};

		queue.write_buffer(&self.buffer, 0, bytes_of(&uniform));
	}

	pub fn bind_group(&self) -> &BindGroup {
		&self.bind_group
	}
}

