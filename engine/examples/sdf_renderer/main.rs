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
    DebugMetricsPlugin, GridPlugin, RenderPlugin, ScenePlugin, UiInputPlugin, UiRenderPlugin,
    default_plugins,
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

include!("main_internal/entry_and_scene.rs");

include!("main_internal/config_and_graph.rs");

include!("main_internal/gpu_and_executors.rs");

include!("main_internal/runtime_helpers.rs");

include!("main_internal/tests.rs");
