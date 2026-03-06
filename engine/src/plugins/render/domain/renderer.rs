use super::PipelineKey;
use super::frame_graph::{FrameGraph, PassHandle, PassKind};
use super::render_executor_registry::{
    BuiltinRenderPassExecutor, RenderFrameDataRegistry, RenderPassEncodeContext,
    RenderPassExecutorRegistryResource, RenderPassPrepareContext,
};
use super::render_graph_registry::{
    RegisteredPassKind, RegisteredPipelineRef, RenderGraphRegistryResource,
};
use super::shader_manager::{ShaderHandle, ShaderRegistryResource};
use crate::plugins::ui::domain::{FileFontProvider, TextRenderer};
use crate::plugins::ui::domain::{UiDrawCmd, UiDrawList};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use wgpu::util::DeviceExt;
use wgpu::*;

include!("renderer_internal/core_types_and_executors.rs");

include!("renderer_internal/graph_and_logging.rs");

include!("renderer_internal/setup_and_ui.rs");

include!("renderer_internal/render_flow.rs");

include!("renderer_internal/tests.rs");
