// Owner: Game of Life SDF Example - Public RenderFlow API Demo
use crate::rendering::{GameOfLifeRenderState, SHADER_ID, build_render_flow};
use anyhow::{Result, anyhow};
use engine::plugins::render::{RenderFrameResourceBindings, ShaderRegistryResource};
use engine::plugins::{RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::App;
use std::path::PathBuf;

pub(crate) fn run() -> Result<()> {
    let flow = build_render_flow();
    flow.validate()?;

    let mut app = App::new();
    app.set_title("Game of Life SDF - Public RenderFlow API Example");
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.add_render_flow(flow);
    install_game_of_life_runtime_state(&mut app)?;
    register_example_shader(&mut app)?;
    app.run()
}

fn install_game_of_life_runtime_state(app: &mut App) -> Result<()> {
    app.insert_resource(GameOfLifeRenderState::default());
    let frame_bindings =
        app.world_mut()
            .resource_mut::<RenderFrameResourceBindings>()
            .map_err(|_| {
                anyhow!(
                    "RenderFrameResourceBindings missing; RenderPlugin must be installed before runtime state registration"
                )
            })?;
    frame_bindings.register_resource::<GameOfLifeRenderState>();
    Ok(())
}

fn register_example_shader(app: &mut App) -> Result<()> {
    let shader_path = shader_asset_path();
    let shader_registry = app
        .world_mut()
        .resource_mut::<ShaderRegistryResource>()
        .map_err(|_| {
            anyhow!("ShaderRegistryResource missing; RenderPlugin must be installed before shader registration")
        })?;
    shader_registry.register_shader(SHADER_ID, shader_path);
    Ok(())
}

fn shader_asset_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../assets/shaders/game_of_life_sdf.wgsl")
        .to_string_lossy()
        .to_string()
}
