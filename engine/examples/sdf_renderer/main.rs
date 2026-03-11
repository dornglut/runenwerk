use anyhow::{Result, anyhow};
use bytemuck::{Pod, Zeroable};
use engine::plugins::input::domain::action;
use engine::plugins::render::domain::{
    BuiltinRenderPassExecutor, RenderFeatureGraphSpec, RenderFrameResourceBindings,
    RenderGraphRegistryResource, RenderPassEncodeContext, RenderPassExecutor,
    RenderPassExecutorRegistryResource, RenderPassPrepareContext,
};
use engine::plugins::ui::domain::UiWorldHudStats;
use engine::plugins::{DebugMetricsPlugin, GridPlugin, RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::{
    App, CoreSet, InputState, Plugin, Res, ResMut, SceneRuntimeState, Startup, SystemConfigExt,
    Time, Update,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use wgpu::*;
use winit::keyboard::KeyCode;

mod config;
mod rendering;
mod runtime;
#[cfg(test)]
mod tests;

pub(crate) use config::*;
pub(crate) use rendering::*;
pub(crate) use runtime::*;

fn main() -> Result<()> {
    runtime::run()
}
