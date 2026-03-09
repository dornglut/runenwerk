mod frame_graph;
mod gfx;
mod pipeline_key;
mod render_executor_registry;
mod render_frame_bindings;
mod render_graph_registry;
mod renderer;
mod shader_manager;
mod wgpu_ctx;

pub mod domain;

mod plugin;
mod submit;

pub use plugin::RenderPlugin;
