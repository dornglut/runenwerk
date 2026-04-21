use crate::plugins::render::*;
use crate::plugins::scene::SceneResource;
use crate::runtime::{ResMut, WorldMut};
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, BTreeSet};

type ExtractedRenderStateMap<'a> = BTreeMap<TypeId, &'a dyn Any>;

pub(crate) fn frame_render_prepare_system(
    mut world: WorldMut,
    mut scene_resource: ResMut<SceneResource>,
) -> anyhow::Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        clear_prepared_frame(&mut world);
        return Ok(());
    };

    let Some(mut shader_registry) = world.remove_resource::<ShaderRegistryResource>() else {
        clear_prepared_frame(&mut world);
        return Ok(());
    };

    let _ = shader_registry.poll_updates();
    let shader_reload_messages = shader_registry.drain_message_lines();
    if !shader_reload_messages.is_empty() {
        for msg in shader_reload_messages {
            manager
                .overlay_runtime
                .ui
                .log_lines
                .push(format!("[world] {msg}"));
        }
        crate::plugins::render::runtime::debug_eval::clamp_lines(
            &mut manager.overlay_runtime.ui.log_lines,
            manager.overlay_runtime.ui.max_lines,
        );
        manager.overlay_runtime.ui.log_scroll_lines_from_bottom = 0;
    }

    let target_size = {
        let (window_w, window_h) = manager.overlay_runtime.ui.screen_size;
        (
            window_w.max(1.0).round() as u32,
            window_h.max(1.0).round() as u32,
        )
    };

    let (flow_registry_revision, compiled_flows, execution_feature_ids, flows) = {
        let flow_registry = match world.resource::<RenderFlowRegistryResource>() {
            Ok(registry) => registry,
            Err(_) => {
                world.insert_resource(shader_registry);
                clear_prepared_frame(&mut world);
                return Ok(());
            }
        };
        let compiled_flows = flow_registry.compiled_flows();
        let execution_feature_ids = collect_execution_feature_ids(compiled_flows);
        let extracted = collect_flow_declared_state_resources(&world, compiled_flows);
        let flows = build_prepared_flow_inputs(compiled_flows, &extracted, target_size)?;
        (
            flow_registry.revision(),
            compiled_flows.len(),
            execution_feature_ids,
            flows,
        )
    };

    let (frame_index, prepare_epoch) = {
        if let Ok(prepared_resource) = world.resource_mut::<PreparedRenderFrameResource>() {
            (
                prepared_resource.allocate_frame_index(),
                prepared_resource.allocate_prepare_epoch(),
            )
        } else {
            (0, 0)
        }
    };

    let contributions = build_frame_feature_contributions(
        &world,
        manager.world.active.label().to_string(),
        manager.active_overlay().label().to_string(),
        &execution_feature_ids,
    );

    let prepared = PreparedRenderFrame {
        context: PreparedFrameContext {
            frame_index,
            flow_registry_revision,
            shader_registry_revision: shader_registry.revision(),
            prepare_epoch,
        },
        surface: PreparedSurfaceInfo {
            target_size_px: target_size,
        },
        views: vec![PreparedViewFrame::main(target_size)],
        flows,
        contributions,
        shader: PreparedShaderSnapshot {
            registry_revision: shader_registry.revision(),
        },
    };

    if let Ok(prepared_resource) = world.resource_mut::<PreparedRenderFrameResource>() {
        prepared_resource.publish(prepared);
    } else {
        let mut prepared_resource = PreparedRenderFrameResource::default();
        prepared_resource.publish(prepared);
        world.insert_resource(prepared_resource);
    }

    world.insert_resource(shader_registry);

    if compiled_flows == 0 {
        clear_prepared_frame(&mut world);
    }

    Ok(())
}

pub(crate) fn clear_prepared_frame(world: &mut WorldMut) {
    if let Ok(prepared_resource) = world.resource_mut::<PreparedRenderFrameResource>() {
        prepared_resource.clear();
    }
}

pub(crate) fn build_frame_feature_contributions(
    world: &ecs::World,
    world_scene_label: String,
    overlay_scene_label: String,
    execution_feature_ids: &[RenderFeatureId],
) -> PreparedFrameContributions {
    let mut contributions = PreparedFrameContributions::default();

    let scene_policy = feature_policy(
        world,
        SCENE_ROUTE_RENDER_FEATURE_ID,
        FeatureFallbackPolicy::EmptyContribution,
    );
    contributions.insert_scene_route(
        world_scene_label,
        overlay_scene_label,
        FeatureContributionStatus::Ready,
        scene_policy,
    );

    if let Ok(resource) = world.resource::<PreparedUiFrameResource>() {
        let ui_policy = feature_policy(world, UI_RENDER_FEATURE_ID, resource.fallback_policy);
        contributions.insert_ui(resource.payload.clone(), resource.status, ui_policy);
    }

    if let Ok(resource) = world.resource::<PreparedWorldFeatureResource>() {
        let world_policy = feature_policy(
            world,
            WORLD_DRAW_RENDER_FEATURE_ID,
            resource.fallback_policy,
        );
        contributions.insert_world(resource.payload.clone(), resource.status, world_policy);
    }

    if let Ok(resource) = world.resource::<PreparedDrawFeatureResource>() {
        let world_feature_id = WORLD_DRAW_RENDER_FEATURE_ID;
        let should_publish_draw = !matches!(resource.status, FeatureContributionStatus::Missing)
            || contributions.feature(&world_feature_id).is_none();
        if should_publish_draw {
            let draw_policy = feature_policy(world, world_feature_id, resource.fallback_policy);
            contributions.insert_draw(resource.payload.clone(), resource.status, draw_policy);
        }
    }

    if let Ok(resource) = world.resource::<PreparedCaveFeatureResource>() {
        let cave_policy = feature_policy(
            world,
            CAVE_INTERIOR_RENDER_FEATURE_ID,
            resource.fallback_policy,
        );
        contributions.insert_caves(resource.payload.clone(), resource.status, cave_policy);
    }

    if let Ok(resource) = world.resource::<PreparedDetailFeatureResource>() {
        let detail_policy =
            feature_policy(world, DETAIL_RENDER_FEATURE_ID, resource.fallback_policy);
        contributions.insert_detail(resource.payload.clone(), resource.status, detail_policy);
    }

    if let Ok(resource) = world.resource::<PreparedProceduralWorldFeatureResource>() {
        let procedural_policy = feature_policy(
            world,
            PROCEDURAL_WORLD_RENDER_FEATURE_ID,
            resource.fallback_policy,
        );
        contributions.insert_procedural_world(
            resource.payload.clone(),
            resource.status,
            procedural_policy,
        );
    }

    if let Ok(resource) = world.resource::<PreparedMaterialFeatureResource>() {
        let material_policy =
            feature_policy(world, MATERIAL_RENDER_FEATURE_ID, resource.fallback_policy);
        contributions.insert_material(resource.payload.clone(), resource.status, material_policy);
    }

    if let Ok(resource) = world.resource::<PreparedDeformationFeatureResource>() {
        let deformation_policy = feature_policy(
            world,
            DEFORMATION_RENDER_FEATURE_ID,
            resource.fallback_policy,
        );
        contributions.insert_deformation(
            resource.payload.clone(),
            resource.status,
            deformation_policy,
        );
    }

    if let Ok(resource) = world.resource::<PreparedWindFieldFeatureResource>() {
        let wind_policy = feature_policy(
            world,
            WIND_FIELDS_RENDER_FEATURE_ID,
            resource.fallback_policy,
        );
        contributions.insert_wind_fields(resource.payload.clone(), resource.status, wind_policy);
    }

    for feature_id in execution_feature_ids {
        if contributions.feature(feature_id).is_some() {
            continue;
        }
        let fallback_policy =
            feature_policy(world, *feature_id, FeatureFallbackPolicy::SkipFeaturePasses);
        contributions.insert_missing(*feature_id, fallback_policy);
    }

    if let Ok(feature_registry) = world.resource::<RenderFeatureRegistryResource>() {
        for feature_id in feature_registry.resolved_order() {
            if contributions.feature(feature_id).is_some() {
                continue;
            }
            let fallback_policy = feature_registry
                .descriptor(feature_id)
                .map(|descriptor| descriptor.fallback_policy)
                .unwrap_or(FeatureFallbackPolicy::SkipFeaturePasses);
            contributions.insert_missing(*feature_id, fallback_policy);
        }
    }

    contributions
}

fn feature_policy(
    world: &ecs::World,
    feature_id: RenderFeatureId,
    fallback: FeatureFallbackPolicy,
) -> FeatureFallbackPolicy {
    world
        .resource::<RenderFeatureRegistryResource>()
        .ok()
        .and_then(|registry| registry.descriptor(&feature_id))
        .map(|descriptor| descriptor.fallback_policy)
        .unwrap_or(fallback)
}

fn collect_execution_feature_ids(
    compiled_flows: &[CompiledRenderFlowPlan],
) -> Vec<RenderFeatureId> {
    let mut ids = BTreeSet::<RenderFeatureId>::new();

    for flow in compiled_flows {
        for pass in &flow.execution.passes {
            let feature_id = match pass {
                CompiledPassExecutionPlan::Compute(value) => value.feature_id,
                CompiledPassExecutionPlan::Fullscreen(value) => value.feature_id,
                CompiledPassExecutionPlan::Graphics(value) => value.feature_id,
                CompiledPassExecutionPlan::Copy(value) => value.feature_id,
                CompiledPassExecutionPlan::Present(value) => value.feature_id,
                CompiledPassExecutionPlan::BuiltinUiComposite(value) => Some(value.feature_id),
            };

            if let Some(feature_id) = feature_id {
                ids.insert(feature_id);
            }
        }
    }

    ids.into_iter().collect()
}

fn collect_flow_declared_state_resources<'a>(
    world: &'a ecs::World,
    compiled_flows: &[CompiledRenderFlowPlan],
) -> ExtractedRenderStateMap<'a> {
    let mut values = ExtractedRenderStateMap::new();
    let mut type_ids = BTreeSet::<TypeId>::new();

    for flow in compiled_flows {
        for declaration in &flow.resources.state_resources {
            type_ids.insert(declaration.type_id);
        }
    }

    for type_id in type_ids {
        if let Some(resource) = world.resource_by_type_id(type_id) {
            values.insert(type_id, resource);
        }
    }

    values
}

fn build_prepared_flow_inputs(
    compiled_flows: &[CompiledRenderFlowPlan],
    extracted_state: &ExtractedRenderStateMap<'_>,
    surface_size: (u32, u32),
) -> anyhow::Result<BTreeMap<RenderFlowId, PreparedFlowInputs>> {
    let mut outputs = BTreeMap::<RenderFlowId, PreparedFlowInputs>::new();

    for flow in compiled_flows {
        let mut projected_uniform_bytes = BTreeMap::<RenderResourceId, Vec<u8>>::new();

        for pass in &flow.pass_order {
            for binding in &pass.node().uniform_bindings {
                if !flow.resources.has_state_resource(binding.state_type_id()) {
                    anyhow::bail!(
                        "uniform projection for flow '{:?}' pass '{:?}' requires undeclared state '{}'",
                        flow.flow_id,
                        pass.pass_id(),
                        binding.state_type_name()
                    );
                }

                if !flow.resources.has_uniform_buffer(binding.uniform_id()) {
                    anyhow::bail!(
                        "uniform projection for flow '{:?}' pass '{:?}' references unknown uniform buffer '{:?}'",
                        flow.flow_id,
                        pass.pass_id(),
                        binding.uniform_id()
                    );
                }

                let state = extracted_state
					.get(&binding.state_type_id())
					.copied()
					.ok_or_else(|| {
						anyhow::anyhow!(
                            "uniform projection for flow '{:?}' pass '{:?}' requires missing ecs state '{}'",
                            flow.flow_id,
                            pass.pass_id(),
                            binding.state_type_name()
                        )
					})?;

                let bytes = binding.project_bytes(state, surface_size).ok_or_else(|| {
                    anyhow::anyhow!(
                        "uniform projection for flow '{:?}' pass '{:?}' failed for state '{}'",
                        flow.flow_id,
                        pass.pass_id(),
                        binding.state_type_name()
                    )
                })?;

                let key = *binding.uniform_id();
                if let Some(existing) = projected_uniform_bytes.get(&key) {
                    if existing != &bytes {
                        anyhow::bail!(
                            "uniform projection conflict for buffer '{:?}' in flow '{:?}'",
                            key,
                            flow.flow_id
                        );
                    }
                    continue;
                }
                projected_uniform_bytes.insert(key, bytes);
            }
        }

        let mut projected_dispatch_workgroups = BTreeMap::<RenderPassId, [u32; 3]>::new();
        for pass in &flow.pass_order {
            if !matches!(pass.node().kind, RenderPassKind::Compute) {
                continue;
            }
            let dispatch = project_dispatch_for_pass(pass.node(), extracted_state)?;
            projected_dispatch_workgroups.insert(pass.pass_id(), dispatch);
        }

        let required_state_types = flow
            .resources
            .state_resources
            .iter()
            .map(|value| PreparedStateTypeInfo {
                type_name: value.type_name,
            })
            .collect::<Vec<_>>();

        outputs.insert(
            flow.flow_id,
            PreparedFlowInputs {
                projected_uniform_bytes,
                projected_dispatch_workgroups,
                required_state_types,
            },
        );
    }

    Ok(outputs)
}

fn project_dispatch_for_pass(
    node: &crate::plugins::render::RenderPassNode,
    extracted_state: &ExtractedRenderStateMap<'_>,
) -> anyhow::Result<[u32; 3]> {
    let dispatch = match &node.compute_dispatch {
        Some(ComputeDispatchDescriptor::Fixed(value)) => *value,
        Some(ComputeDispatchDescriptor::State(binding)) => {
            let state = extracted_state
                .get(&binding.state_type_id())
                .copied()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "compute pass '{:?}' dispatch_state requires missing ecs resource '{}'",
                        node.id,
                        binding.state_type_name()
                    )
                })?;
            binding.project_dispatch(state).ok_or_else(|| {
                anyhow::anyhow!(
                    "compute pass '{:?}' failed to project dispatch_state for '{}'",
                    node.id,
                    binding.state_type_name()
                )
            })?
        }
        None => {
            anyhow::bail!(
                "compute pass '{:?}' must declare explicit dispatch_workgroups(...) or dispatch_state(...)",
                node.id
            );
        }
    };

    if dispatch[0] == 0 || dispatch[1] == 0 || dispatch[2] == 0 {
        anyhow::bail!(
            "compute pass '{:?}' resolved invalid dispatch dimensions ({}, {}, {})",
            node.id,
            dispatch[0],
            dispatch[1],
            dispatch[2]
        );
    }

    Ok(dispatch)
}
