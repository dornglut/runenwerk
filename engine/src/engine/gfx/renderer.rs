use crate::engine::gfx::CameraGPU;
use anyhow::Result;
// src/renderer.rs
use wgpu::*;

pub trait Renderer { fn render(&self, frame_view: &TextureView, device: &Device, queue: &Queue, camera: &CameraGPU) -> Result<(), SurfaceError>; }
#[derive(Debug)]
pub struct DefaultRenderer {
    pub pipeline: Option<RenderPipeline>,
}

impl DefaultRenderer {
    pub fn new_empty() -> Self {
        Self { pipeline: None }
    }

    pub fn init_pipeline(&mut self, device: &Device, camera_layout: &BindGroupLayout, format: TextureFormat) {
        self.pipeline = Some(Self::new_pipeline(device, camera_layout, format));
    }

    pub fn new_pipeline(device: &Device, camera_layout: &BindGroupLayout, format: TextureFormat) -> RenderPipeline {
        let shader =  device.create_shader_module( include_wgsl!("ray.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Ray-march layout"),
            bind_group_layouts: &[camera_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Ray-march pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState{
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            },
            ),
            multiview: None,
            cache: None,
        })
    }
}

impl Renderer for DefaultRenderer {
    fn render(&self, frame_view: &TextureView, device: &Device, queue: &Queue, camera: &CameraGPU) -> Result<(), SurfaceError> {
        let pipeline = match &self.pipeline {
            Some(pipeline) => pipeline,
            None => return Ok(()),
        };

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor{
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment{
                    view: frame_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &camera.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}