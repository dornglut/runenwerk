use super::PipelineKey;
use anyhow::{Result, bail};
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};

pub trait RenderPassExecutor: Send + Sync {
    fn prepare(&self, _ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        Ok(())
    }

    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()>;
}

include!("render_executor_registry_internal/frame_data.rs");
include!("render_executor_registry_internal/contexts_and_builtin.rs");
include!("render_executor_registry_internal/registry.rs");
include!("render_executor_registry_internal/tests.rs");
