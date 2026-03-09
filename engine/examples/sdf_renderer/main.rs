use anyhow::{Result, anyhow};
use bytemuck::{Pod, Zeroable};
use engine::plugins::input::domain::action;
use engine::plugins::render::domain::{
    BuiltinRenderPassExecutor, RenderFeatureGraphSpec, RenderFrameResourceBindings,
    RenderGraphRegistryResource, RenderPassEncodeContext, RenderPassExecutor,
    RenderPassExecutorRegistryResource, RenderPassPrepareContext,
};
use engine::plugins::ui::domain::UiWorldHudStats;
use engine::plugins::{
    DebugMetricsPlugin, GridPlugin, RenderPlugin, ScenePlugin, default_plugins,
};
use engine::prelude::{
    App, CoreSet, InputState, Plugin, Res, ResMut, SceneRuntimeState, Startup, SystemConfigExt,
    Time, Update,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use wgpu::*;
use winit::keyboard::KeyCode;

mod config_and_graph;
mod entry_and_scene;
mod gpu_and_executors;
mod runtime_helpers;
#[cfg(test)]
mod tests;

pub(crate) use config_and_graph::*;
pub(crate) use entry_and_scene::*;
pub(crate) use gpu_and_executors::*;
pub(crate) use runtime_helpers::*;

fn main() -> Result<()> {
    entry_and_scene::run()
}
