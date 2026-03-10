use super::*;

// Owner: Cavern Hunt SDF Renderer - Resource and Executor Setup
pub(crate) fn setup_render_resources(world: &mut World) -> Result<()> {
    let mut frame_bindings = world
        .resource_mut::<RenderFrameResourceBindings>()
        .map_err(|_| anyhow!("RenderPlugin must be installed before Cavern Hunt client plugin"))?;
    if !frame_bindings.contains_resource::<CavernSdfWorldFrame>() {
        frame_bindings.register_resource::<CavernSdfWorldFrame>();
    }
    drop(frame_bindings);

    let spec = build_feature_graph_spec()?;
    world
        .resource_mut::<RenderGraphRegistryResource>()?
        .register_feature_graph(spec);

    let shared = Arc::new(Mutex::new(CavernGpuSharedState::default()));
    let mut executors = world.resource_mut::<RenderPassExecutorRegistryResource>()?;
    executors.register_custom(
        COMPUTE_EXECUTOR_ID,
        Arc::new(CavernComputeExecutor::new(Arc::clone(&shared))),
    );
    executors.register_custom(
        COMPOSE_EXECUTOR_ID,
        Arc::new(CavernComposeExecutor::new(shared)),
    );
    Ok(())
}
