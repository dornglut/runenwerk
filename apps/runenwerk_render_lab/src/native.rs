use super::*;

#[derive(Debug, Clone, Copy, runen_ecs::Resource)]
struct RenderLabFlowId(engine::plugins::render::RenderFlowId);

#[derive(runen_ecs::SystemParam)]
struct RenderLabFramePublicationResources<'w> {
    targets: ResMut<'w, RenderDynamicTextureTargetRequestRegistryResource>,
    frame_requests: ResMut<'w, PreparedRenderFrameRequestResource>,
    contributions: ResMut<'w, RenderDeterministicFrameContributionResource>,
}

struct RenderLabPlugin;

impl Plugin for RenderLabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderLabCamera>();
        app.add_systems(Update, camera::update_render_lab_camera_system);
        app.add_systems(Update, approve_render_lab_close_system);
        app.add_systems(
            RenderPrepare,
            publish_render_lab_frame_system.before(RenderRuntimeSet::FramePrepare),
        );
    }
}

pub fn run_native() -> Result<()> {
    let mut app = App::new();
    app.set_title("Runenwerk Render Lab — RL2 native interaction");
    app.with_frame_pacing(FramePacingPolicyResource::continuous_capped(60));
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.add_plugin(RenderLabPlugin);
    let flow = render_lab_flow()?;
    app.insert_resource(RenderLabFlowId(flow.id()));
    app.add_render_flow(flow);
    app.run()
}

pub(super) fn render_lab_flow() -> Result<RenderFlow> {
    RenderFlow::new(RL2_FLOW_ID)
        .with_target_alias(RL2_RADIANCE_ALIAS, RenderTargetAliasKind::Texture)?
        .with_surface_color()?
        .fullscreen_pass(RL2_PASS_ID)
        .main_surface_only()
        .shader_asset("assets/shaders/runenwerk_render_lab_radiance.wgsl")
        .sample_texture_load(runen_gpu::GpuBindingKey::try_new(0, 0)?, RL2_RADIANCE_ALIAS)
        .clear_color([0.0, 0.0, 0.0, 1.0])
        .write_surface_color()?
        .finish()
        .present_pass(RL2_PRESENT_ID)?
        .main_surface_only()
        .surface_color()?
        .finish()
        .validate()
}

fn approve_render_lab_close_system(
    mut window: ResMut<WindowState>,
    mut windows: ResMut<WindowStateRegistryResource>,
) {
    if window.close_intent_pending {
        window.request_close();
    }
    if let Some(primary_window_id) = windows.primary_window_id()
        && let Some(primary_window) = windows.record_mut(primary_window_id)
        && primary_window.close_intent_pending
    {
        primary_window.request_close();
    }
}

fn publish_render_lab_frame_system(
    camera: Res<RenderLabCamera>,
    flow_id: Res<RenderLabFlowId>,
    window: Res<WindowState>,
    publication: RenderLabFramePublicationResources<'_>,
) -> Result<()> {
    let RenderLabFramePublicationResources {
        mut targets,
        mut frame_requests,
        mut contributions,
    } = publication;
    let (width, height) = (window.size_px.0.max(1), window.size_px.1.max(1));
    let producer_id = engine::plugins::render::RenderFrameProducerId::try_from_raw(RL2_PRODUCER_ID)
        .expect("Render Lab producer id is non-zero");
    let target_key = RenderDynamicTextureTargetKey::new(RL2_TARGET_NAMESPACE, RL2_TARGET_ID);
    targets.remove_contribution(producer_id);
    targets.replace_contribution(
        producer_id,
        [RenderDynamicTextureTargetDescriptor::new(
            target_key.clone(),
            width,
            height,
            RenderTextureTargetFormat::R32Float,
            RenderTextureTargetUsage {
                color_attachment: false,
                depth_attachment: false,
                sampled: true,
                storage: false,
                copy_src: false,
                copy_dst: true,
            },
            RenderTextureSampleMode::NonFilterableFloat,
            RenderDynamicTextureRetention::RetainWhileRequested,
        )],
    )?;
    let invocation =
        PreparedFlowInvocationRequest::new(format!("{RL2_FLOW_ID}.main"), flow_id.0, "main")
            .bind_dynamic_texture_alias(RL2_RADIANCE_ALIAS, target_key.clone())?;
    frame_requests.remove_contribution(producer_id);
    frame_requests.replace_contribution(producer_id, [], [invocation])?;
    let fixture =
        founding_fixture_with_observation_and_extent(camera.observation_to_scene(), width, height)?;
    contributions.remove(producer_id);
    contributions.replace(RenderDeterministicFrameContribution {
        producer_id,
        render_surface_id: RenderSurfaceId::primary(),
        scene: fixture.scene,
        request: fixture.request,
        semantic_inputs: fixture.semantic_inputs,
        availability: fixture.availability,
        output_index: 0,
        target_key,
    });
    Ok(())
}
