use editor_shell::{ComputedLayoutMap, UiNode, UiNodeKind, viewport_embed_slot_for};
use editor_viewport::{ViewportId, ViewportSurfacePresentationSlot};
use engine::PrimaryPresentationMetricsResource;
use engine::plugins::render::{
    EditorPickingTarget, RenderFrameProducerId, SurfaceFrameRoute, SurfaceFrameSubmission,
    SurfaceFrameSubmissionOrder, SurfaceFrameSubmissionRegistryResource, UiFontAtlasResource,
};
use engine::runtime::{NativeWindowLifecycleState, Res, ResMut, WindowStateRegistryResource};
use scene::LocalTransform;
use ui_composition::PresentationTargetId;
use ui_math::UiRect;
use ui_render_data::{
    RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId, UiPaint, UiPrimitive, UiSortKey,
    UiSurface, UiSurfaceId,
};

use crate::editor_runtime::EditorPrimitive;
use crate::runtime::resources::{
    EditorHostResource, EditorViewportDebugStage, EditorViewportModelMeshMaterialSelectionPacket,
    EditorViewportPrimitiveInstance, EditorViewportRenderState, EditorViewportSceneRenderPacket,
    effective_shell_scale, scaled_shell_theme,
};
use crate::runtime::viewport::{
    MountedSurfaceRegistryResource, ToolSurfaceRuntimeBindingRegistryResource,
    ViewportArtifactObservationResource, ViewportInstanceRegistryResource, ViewportLayoutEntry,
    ViewportLayoutMapResource, ViewportPickingResultsResource, ViewportRenderStateEntry,
    ViewportRenderStateResource, resolve_structural_viewport_products,
};
use crate::shell::RunenwerkEditorShellState;

const EDITOR_SHELL_UI_PRODUCER_ID: RenderFrameProducerId = ui_frame_producer_id(1001);
const DEBUG_HARDCODED_UI_FRAME_ENV: &str = "RUNENWERK_EDITOR_DEBUG_UI_FRAME";
const VIEWPORT_DEBUG_STAGE_ENV: &str = "RUNENWERK_EDITOR_VIEWPORT_DEBUG_STAGE";
const VIEWPORT_ROOT_OPAQUE_ENV: &str = "RUNENWERK_EDITOR_VIEWPORT_ROOT_OPAQUE";
const VIEWPORT_BRANCH_TRACE_ENV: &str = "RUNENWERK_EDITOR_VIEWPORT_BRANCH_TRACE";

const fn ui_frame_producer_id(raw: u64) -> RenderFrameProducerId {
    match RenderFrameProducerId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("ui frame producer id constants must be non-zero"),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn submit_editor_frame_system(
    presentation: Res<PrimaryPresentationMetricsResource>,
    debug_metrics: Res<engine::DebugMetricsState>,
    mut host: ResMut<EditorHostResource>,
    mut viewport_render_states: ResMut<ViewportRenderStateResource>,
    viewport_observations: Res<ViewportArtifactObservationResource>,
    viewport_instances: Res<ViewportInstanceRegistryResource>,
    tool_surface_bindings: Res<ToolSurfaceRuntimeBindingRegistryResource>,
    mut mounted_surfaces: ResMut<MountedSurfaceRegistryResource>,
    atlas: Res<UiFontAtlasResource>,
    viewport_picking_results: Res<ViewportPickingResultsResource>,
    mut submissions: ResMut<SurfaceFrameSubmissionRegistryResource>,
) {
    let bounds = presentation_bounds(&presentation);
    let shell_scale = effective_shell_scale(presentation.scale_factor());
    host.apply_pending_editor_definition_activations();
    let EditorHostResource {
        app,
        shell_state,
        theme,
    } = &mut *host;
    let shell_theme = scaled_shell_theme(theme, presentation.scale_factor());
    let primary_target_id = shell_state
        .composition_runtime()
        .composition()
        .definition()
        .targets()
        .first()
        .map(|target| target.id);
    let viewport_products = resolve_structural_viewport_products(
        shell_state,
        &viewport_observations,
        &tool_surface_bindings,
    );
    let active_viewport_id = viewport_products.map(|value| value.viewport_id);
    let (expression_source_version, frame) = if debug_hardcoded_ui_frame_enabled() {
        let expression = editor_shell::ShellUiExpressionFrame::new(
            app.runtime().current_scene_reality_version(),
            build_debug_frame(bounds),
        );
        (
            expression.metadata.source_version,
            expression.into_ui_frame(),
        )
    } else {
        let expression = primary_target_id
            .and_then(|target_id| {
                app.build_shell_expression_frame_for_target_with_surface_resources(
                    shell_state,
                    target_id,
                    bounds,
                    &shell_theme,
                    &*atlas,
                    Some(&viewport_observations),
                    Some(&tool_surface_bindings),
                    Some(&viewport_instances),
                    Some(crate::shell::EditorShellFrameMetrics {
                        fps_ema: debug_metrics.fps_ema,
                        frame_ms_ema: debug_metrics.frame_ms_ema,
                    }),
                )
            })
            .unwrap_or_else(|| {
                app.build_shell_expression_frame_with_surface_resources(
                    shell_state,
                    bounds,
                    &shell_theme,
                    &*atlas,
                    Some(&viewport_observations),
                    Some(&tool_surface_bindings),
                    Some(&viewport_instances),
                    Some(crate::shell::EditorShellFrameMetrics {
                        fps_ema: debug_metrics.fps_ema,
                        frame_ms_ema: debug_metrics.frame_ms_ema,
                    }),
                )
            });
        (
            expression.metadata.source_version,
            expression.into_ui_frame(),
        )
    };
    mounted_surfaces.sync_from_composition(shell_state.composition_runtime());

    let diagnostic_viewport_id =
        active_viewport_id.or_else(|| viewport_render_states.viewport_ids().next());
    if let Some(viewport_id) = diagnostic_viewport_id
        && let Some(entry) = viewport_render_states.state_for_mut(viewport_id)
    {
        let viewport_render = &mut entry.render_state;
        let contradiction_active =
            picking_hits_entity_or_component(&viewport_picking_results, Some(viewport_id))
                && viewport_render.scene_should_be_invisible();
        let should_report_contradiction =
            viewport_render.should_report_visibility_contradiction(contradiction_active);
        let branch_trace_enabled = viewport_branch_trace_enabled();
        let branch_trace_snapshot = if branch_trace_enabled || should_report_contradiction {
            Some(viewport_render.branch_trace_snapshot(presentation.size_px()))
        } else {
            None
        };

        if app.debug_logs_enabled() {
            if viewport_render.should_report_scale_change() {
                app.append_console_line(format!(
                    "[ui] shell scale={:.3} presentation_scale={:.3} expression_version={}",
                    shell_scale,
                    presentation.scale_factor(),
                    expression_source_version.0
                ));
            }

            if viewport_render.should_report_bounds_change() {
                app.append_console_line(format!(
                    "[viewport] viewport={} bounds=({:.1},{:.1},{:.1},{:.1})",
                    viewport_id.0,
                    entry.bounds.x,
                    entry.bounds.y,
                    entry.bounds.width,
                    entry.bounds.height
                ));
                if entry.bounds.width <= f32::EPSILON || entry.bounds.height <= f32::EPSILON {
                    app.append_console_line(
                        "[viewport] warning: viewport canvas bounds are zero-sized".to_string(),
                    );
                }
            }

            if viewport_render.should_report_debug_state_change() {
                app.append_console_line(format!(
                "[viewport] viewport={} root-occlusion={} debug-stage={} viewport_valid={} shader_loaded={} primitive_visible={}",
                viewport_id.0,
                if viewport_render.root_background_opaque { "opaque" } else { "transparent" },
                viewport_render.debug_stage.label(),
                viewport_render.viewport_valid,
                viewport_render.shader_loaded,
                viewport_render.has_primitive,
            ));
            }
        }

        if branch_trace_enabled
            && let Some(snapshot) = branch_trace_snapshot
            && viewport_render.should_report_branch_trace_change(snapshot)
        {
            app.append_console_line(format!("[viewport.branch] {}", snapshot.summary_line()));
        }

        if should_report_contradiction {
            let mut line = format!(
                "[viewport] contradiction: analytic picking hit while render-state indicates invisible ({})",
                contradiction_reasons(viewport_render)
            );
            if let Some(snapshot) = branch_trace_snapshot {
                line.push_str(" | ");
                line.push_str(&snapshot.summary_line());
            }
            app.append_console_line(line);
        }
    }

    let primary_surface_id = primary_target_id
        .and_then(|target_id| shell_state.composition_target_binding(target_id))
        .map(|binding| binding.render_surface_id)
        .unwrap_or_else(engine::plugins::render::backend::RenderSurfaceId::primary);
    submissions.replace_for_surface(
        EDITOR_SHELL_UI_PRODUCER_ID,
        primary_surface_id,
        |producer_id| {
            SurfaceFrameSubmission::new(producer_id)
                .with_route(SurfaceFrameRoute::Screen)
                .with_order(SurfaceFrameSubmissionOrder::new(10, 0))
                .with_frame(frame)
        },
    );
}

#[allow(clippy::too_many_arguments)]
pub fn submit_editor_secondary_native_frames_system(
    window_registry: Res<WindowStateRegistryResource>,
    debug_metrics: Res<engine::DebugMetricsState>,
    mut host: ResMut<EditorHostResource>,
    viewport_observations: Res<ViewportArtifactObservationResource>,
    viewport_instances: Res<ViewportInstanceRegistryResource>,
    tool_surface_bindings: Res<ToolSurfaceRuntimeBindingRegistryResource>,
    atlas: Res<UiFontAtlasResource>,
    mut submissions: ResMut<SurfaceFrameSubmissionRegistryResource>,
) {
    let EditorHostResource {
        app,
        shell_state,
        theme,
    } = &mut *host;
    let primary_target_id = shell_state
        .composition_runtime()
        .composition()
        .definition()
        .targets()
        .first()
        .map(|target| target.id);

    let secondary_targets = shell_state
        .composition_target_bindings()
        .filter(|entry| Some(entry.target_id) != primary_target_id)
        .collect::<Vec<_>>();

    for entry in secondary_targets {
        let Some(record) = window_registry.record(entry.binding.native_window_id) else {
            continue;
        };
        if record.lifecycle_state != NativeWindowLifecycleState::Created {
            continue;
        }
        let target_bounds = UiRect::new(
            0.0,
            0.0,
            record.size_px.0.max(1) as f32,
            record.size_px.1.max(1) as f32,
        );
        let shell_theme = scaled_shell_theme(theme, record.scale_factor);
        let Some(expression) = app.build_shell_expression_frame_for_target_with_surface_resources(
            shell_state,
            entry.target_id,
            target_bounds,
            &shell_theme,
            &*atlas,
            Some(&viewport_observations),
            Some(&tool_surface_bindings),
            Some(&viewport_instances),
            Some(crate::shell::EditorShellFrameMetrics {
                fps_ema: debug_metrics.fps_ema,
                frame_ms_ema: debug_metrics.frame_ms_ema,
            }),
        ) else {
            continue;
        };
        submissions.replace_for_surface(
            EDITOR_SHELL_UI_PRODUCER_ID,
            entry.binding.render_surface_id,
            |producer_id| {
                SurfaceFrameSubmission::new(producer_id)
                    .with_route(SurfaceFrameRoute::Screen)
                    .with_order(SurfaceFrameSubmissionOrder::new(10, 0))
                    .with_frame(expression.into_ui_frame())
            },
        );
    }
}

pub fn sync_editor_primary_viewport_projection_system(
    presentation: Res<PrimaryPresentationMetricsResource>,
    mut host: ResMut<EditorHostResource>,
    viewport_instances: Res<ViewportInstanceRegistryResource>,
    mut viewport_layout_map: ResMut<ViewportLayoutMapResource>,
    mut tool_surface_bindings: ResMut<ToolSurfaceRuntimeBindingRegistryResource>,
    mut viewport_render_states: ResMut<ViewportRenderStateResource>,
) {
    let Some(primary_target_id) = host
        .shell_state
        .composition_runtime()
        .composition()
        .definition()
        .targets()
        .first()
        .map(|target| target.id)
    else {
        viewport_layout_map.clear();
        tool_surface_bindings
            .rebuild_from_layout_map_with_instances(&viewport_layout_map, &viewport_instances);
        viewport_render_states.retain_viewports(|_| false);
        return;
    };
    let target_scales = [(
        primary_target_id,
        effective_shell_scale(presentation.scale_factor()),
    )];
    rebuild_viewport_runtime_projection(
        &mut host,
        &viewport_instances,
        &mut viewport_layout_map,
        &mut tool_surface_bindings,
        &mut viewport_render_states,
        &target_scales,
    );
}

pub fn sync_editor_all_target_viewport_projection_system(
    presentation: Res<PrimaryPresentationMetricsResource>,
    window_registry: Res<WindowStateRegistryResource>,
    mut host: ResMut<EditorHostResource>,
    viewport_instances: Res<ViewportInstanceRegistryResource>,
    mut viewport_layout_map: ResMut<ViewportLayoutMapResource>,
    mut tool_surface_bindings: ResMut<ToolSurfaceRuntimeBindingRegistryResource>,
    mut viewport_render_states: ResMut<ViewportRenderStateResource>,
) {
    let primary_target_id = host
        .shell_state
        .composition_runtime()
        .composition()
        .definition()
        .targets()
        .first()
        .map(|target| target.id);
    let target_ids = host
        .shell_state
        .composition_runtime()
        .composition()
        .definition()
        .targets()
        .iter()
        .map(|target| target.id)
        .collect::<Vec<_>>();
    let target_scales = target_ids
        .into_iter()
        .filter_map(|target_id| {
            if Some(target_id) == primary_target_id {
                return Some((
                    target_id,
                    effective_shell_scale(presentation.scale_factor()),
                ));
            }
            let binding = host.shell_state.composition_target_binding(target_id)?;
            let window = window_registry.record(binding.native_window_id)?;
            (window.lifecycle_state == NativeWindowLifecycleState::Created)
                .then_some((target_id, effective_shell_scale(window.scale_factor)))
        })
        .collect::<Vec<_>>();
    rebuild_viewport_runtime_projection(
        &mut host,
        &viewport_instances,
        &mut viewport_layout_map,
        &mut tool_surface_bindings,
        &mut viewport_render_states,
        &target_scales,
    );
}

fn rebuild_viewport_runtime_projection(
    host: &mut EditorHostResource,
    viewport_instances: &ViewportInstanceRegistryResource,
    viewport_layout_map: &mut ViewportLayoutMapResource,
    tool_surface_bindings: &mut ToolSurfaceRuntimeBindingRegistryResource,
    viewport_render_states: &mut ViewportRenderStateResource,
    target_scales: &[(PresentationTargetId, f32)],
) {
    viewport_layout_map.clear();
    let mut fallback_embeds = Vec::new();
    for (target_id, shell_scale) in target_scales.iter().copied() {
        populate_viewport_layout_map_for_target(
            &host.shell_state,
            target_id,
            viewport_layout_map,
            shell_scale,
            &mut fallback_embeds,
        );
    }
    tool_surface_bindings
        .rebuild_from_layout_map_with_instances(viewport_layout_map, viewport_instances);
    sync_viewport_render_states_from_bindings(
        &host.app,
        viewport_render_states,
        tool_surface_bindings,
        &fallback_embeds,
        viewport_debug_stage(),
        root_background_opaque_enabled(),
    );
    if host.app.debug_logs_enabled() {
        for rebind in tool_surface_bindings.latest_rebinds() {
            host.app.append_console_line(format!(
                "[viewport.binding] rebind tool_surface={} from_viewport={} to_viewport={}",
                rebind.tool_surface_id.raw(),
                rebind.from_viewport_id.0,
                rebind.to_viewport_id.0
            ));
        }
    }
}

fn sync_viewport_render_states_from_bindings(
    app: &crate::editor_app::RunenwerkEditorApp,
    viewport_render_states: &mut ViewportRenderStateResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    fallback_embeds: &[(ViewportId, UiRect, f32)],
    default_debug_stage: EditorViewportDebugStage,
    default_root_background_opaque: bool,
) {
    let mut viewport_ids = std::collections::BTreeSet::new();
    for binding in tool_surface_bindings.bindings() {
        let mut render_state = viewport_render_states
            .state_for(binding.viewport_id)
            .map(|previous| previous.render_state.clone())
            .unwrap_or_else(|| {
                let mut state = EditorViewportRenderState::default();
                state.set_debug_stage(default_debug_stage);
                state.set_root_background_opaque(default_root_background_opaque);
                state
            });
        render_state.set_viewport_bounds((
            binding.bounds.x,
            binding.bounds.y,
            binding.bounds.width,
            binding.bounds.height,
        ));
        render_state.set_effective_shell_scale(binding.effective_shell_scale);
        populate_viewport_render_state(app, &mut render_state, binding.bounds);
        render_state.update_visibility_diagnostics(viewport_is_valid(binding.bounds), true);
        viewport_ids.insert(binding.viewport_id);
        viewport_render_states.upsert_state(ViewportRenderStateEntry {
            viewport_id: binding.viewport_id,
            tool_surface_id: Some(binding.tool_surface_id),
            bounds: binding.bounds,
            render_state,
        });
    }
    for (viewport_id, bounds, shell_scale) in fallback_embeds {
        if viewport_ids.contains(viewport_id) {
            continue;
        }
        let mut render_state = viewport_render_states
            .state_for(*viewport_id)
            .map(|previous| previous.render_state.clone())
            .unwrap_or_else(|| {
                let mut state = EditorViewportRenderState::default();
                state.set_debug_stage(default_debug_stage);
                state.set_root_background_opaque(default_root_background_opaque);
                state
            });
        render_state.set_viewport_bounds((bounds.x, bounds.y, bounds.width, bounds.height));
        render_state.set_effective_shell_scale(*shell_scale);
        populate_viewport_render_state(app, &mut render_state, *bounds);
        render_state.update_visibility_diagnostics(viewport_is_valid(*bounds), true);
        viewport_ids.insert(*viewport_id);
        viewport_render_states.upsert_state(ViewportRenderStateEntry {
            viewport_id: *viewport_id,
            tool_surface_id: None,
            bounds: *bounds,
            render_state,
        });
    }
    viewport_render_states.retain_viewports(|viewport_id| viewport_ids.contains(&viewport_id));
}

fn debug_hardcoded_ui_frame_enabled() -> bool {
    std::env::var(DEBUG_HARDCODED_UI_FRAME_ENV)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn viewport_debug_stage() -> EditorViewportDebugStage {
    std::env::var(VIEWPORT_DEBUG_STAGE_ENV)
        .map(|value| EditorViewportDebugStage::from_env_value(&value))
        .unwrap_or(EditorViewportDebugStage::Scene)
}

fn root_background_opaque_enabled() -> bool {
    std::env::var(VIEWPORT_ROOT_OPAQUE_ENV)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn viewport_branch_trace_enabled() -> bool {
    std::env::var(VIEWPORT_BRANCH_TRACE_ENV)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn build_debug_frame(bounds: UiRect) -> UiFrame {
    let mut layer = UiLayer::new(UiLayerId(0));
    let debug_rect = UiRect::new(
        24.0,
        24.0,
        (bounds.width - 48.0).clamp(80.0, 420.0),
        (bounds.height - 48.0).clamp(80.0, 160.0),
    );
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        debug_rect,
        8.0,
        UiPaint::rgba(0.17, 0.58, 0.36, 0.96),
        UiDrawKey::new(0, None),
        UiSortKey::new(0, 0, 0),
    )));

    UiFrame::with_surfaces(vec![UiSurface::with_layers(
        UiSurfaceId(0),
        bounds.size(),
        vec![layer],
    )])
}

fn presentation_bounds(presentation: &PrimaryPresentationMetricsResource) -> UiRect {
    let size_px = presentation.size_px();
    let width = size_px.0 as f32;
    let height = size_px.1 as f32;
    UiRect::new(0.0, 0.0, width, height)
}

fn populate_viewport_layout_map_for_target(
    shell_state: &RunenwerkEditorShellState,
    presentation_target_id: PresentationTargetId,
    viewport_layout_map: &mut ViewportLayoutMapResource,
    effective_shell_scale: f32,
    fallback_embeds: &mut Vec<(ViewportId, UiRect, f32)>,
) {
    let (Some(tree), Some(bounds), Some(artifacts), Some(runtime)) = (
        shell_state.last_tree_for_target(presentation_target_id),
        shell_state.last_bounds_for_target(presentation_target_id),
        shell_state.last_projection_artifacts_for_target(presentation_target_id),
        shell_state.runtime_for_target(presentation_target_id),
    ) else {
        return;
    };
    let layouts = runtime.compute_layout(tree, bounds);
    collect_viewport_layout_entries(
        &tree.root,
        &layouts,
        &artifacts.widget_structural_context_by_id,
        presentation_target_id,
        viewport_layout_map,
        bounds,
        effective_shell_scale,
        fallback_embeds,
    );
}

#[allow(clippy::too_many_arguments)]
fn collect_viewport_layout_entries(
    node: &UiNode,
    layouts: &ComputedLayoutMap,
    structural_contexts: &std::collections::BTreeMap<
        editor_shell::WidgetId,
        editor_shell::StructuralWidgetRoutingContext,
    >,
    presentation_target_id: PresentationTargetId,
    viewport_layout_map: &mut ViewportLayoutMapResource,
    fallback_bounds: UiRect,
    effective_shell_scale: f32,
    fallback_embeds: &mut Vec<(ViewportId, UiRect, f32)>,
) {
    if let UiNodeKind::ViewportSurfaceEmbed(embed) = &node.kind
        && embed.slot == viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary)
    {
        let bounds = layouts
            .get(&node.id)
            .map(|layout| layout.bounds)
            .unwrap_or(fallback_bounds);
        let viewport_id = editor_viewport::ViewportId(embed.viewport_id);
        fallback_embeds.push((viewport_id, bounds, effective_shell_scale));
        if let Some(structural_context) = structural_contexts.get(&node.id).copied() {
            viewport_layout_map.upsert_entry(ViewportLayoutEntry {
                presentation_target_id,
                viewport_id,
                host_widget_id: node.id,
                structural_context,
                bounds,
                effective_shell_scale,
            });
        }
    }
    for child in &node.children {
        collect_viewport_layout_entries(
            child,
            layouts,
            structural_contexts,
            presentation_target_id,
            viewport_layout_map,
            fallback_bounds,
            effective_shell_scale,
            fallback_embeds,
        );
    }
}

fn viewport_is_valid(bounds: UiRect) -> bool {
    bounds.width > f32::EPSILON && bounds.height > f32::EPSILON
}

fn picking_hits_entity_or_component(
    picking_results: &ViewportPickingResultsResource,
    viewport_id: Option<editor_viewport::ViewportId>,
) -> bool {
    let Some(viewport_id) = viewport_id else {
        return false;
    };
    let Some(picking) = picking_results.result_for(viewport_id) else {
        return false;
    };
    matches!(
        picking.hit.target,
        EditorPickingTarget::Entity(_) | EditorPickingTarget::ComponentHandle { .. }
    )
}

fn contradiction_reasons(state: &EditorViewportRenderState) -> String {
    let mut reasons = Vec::new();
    if state.debug_stage != EditorViewportDebugStage::Scene {
        reasons.push("debug-stage");
    }
    if !state.viewport_valid {
        reasons.push("invalid viewport");
    }
    if !state.shader_loaded {
        reasons.push("fallback shader");
    }
    if !state.has_primitive {
        reasons.push("missing primitive");
    }
    if reasons.is_empty() {
        reasons.push("unknown");
    }
    reasons.join(", ")
}

fn populate_viewport_render_state(
    app: &crate::editor_app::RunenwerkEditorApp,
    render_state: &mut EditorViewportRenderState,
    viewport_bounds: UiRect,
) -> bool {
    let bounds_changed = render_state.set_viewport_bounds(viewport_bounds_tuple(viewport_bounds));

    let runtime = app.runtime();
    let packet = extract_viewport_scene_render_packet_with_material_slots(
        runtime,
        app.viewport_tool_state().hovered_entity,
        |entity| runtime.material_slot_index_for_entity(entity),
    );
    if packet.is_empty() {
        render_state.clear_primitive();
    } else {
        render_state.set_scene_packet(packet);
    }
    let model_mesh_material_regions = runtime
        .scene_material_assignments()
        .model_mesh_assignments()
        .map(|assignment| assignment.material_region)
        .collect::<Vec<_>>();
    render_state.set_model_mesh_material_selection_packet(
        EditorViewportModelMeshMaterialSelectionPacket::from_model_mesh_regions(
            runtime.scene_material_assignments(),
            model_mesh_material_regions,
        ),
    );

    bounds_changed
}

fn viewport_bounds_tuple(bounds: UiRect) -> (f32, f32, f32, f32) {
    (bounds.x, bounds.y, bounds.width, bounds.height)
}

#[cfg(test)]
pub(crate) fn extract_viewport_scene_render_packet(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
    hovered_entity: Option<editor_core::EntityId>,
) -> EditorViewportSceneRenderPacket {
    extract_viewport_scene_render_packet_with_material_slots(runtime, hovered_entity, |_| 0)
}

pub(crate) fn extract_viewport_scene_render_packet_with_material_slots(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
    hovered_entity: Option<editor_core::EntityId>,
    material_slot_index_for_entity: impl Fn(editor_core::EntityId) -> u32,
) -> EditorViewportSceneRenderPacket {
    let selected_entity = runtime.selected_entity();
    EditorViewportSceneRenderPacket::from_primitives(runtime.document().entity_ids().filter_map(
        |entity| {
            let (transform, primitive) = entity_primitive(runtime, entity)?;
            Some(
                EditorViewportPrimitiveInstance::from_transform_and_primitive(
                    entity,
                    transform,
                    primitive,
                    selected_entity == Some(entity),
                    hovered_entity == Some(entity),
                )
                .with_material_slot_index(material_slot_index_for_entity(entity)),
            )
        },
    ))
}

pub(crate) fn entity_primitive(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
    entity: editor_core::EntityId,
) -> Option<(LocalTransform, EditorPrimitive)> {
    let ecs_entity = runtime.ids().resolve_entity(entity)?;
    let transform = runtime.world().get::<LocalTransform>(ecs_entity).copied()?;
    let primitive = runtime
        .world()
        .get::<EditorPrimitive>(ecs_entity)
        .copied()?;
    Some((transform, primitive))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_runtime::{execute_scene_intent, register_mvp_component_types};
    use crate::runtime::viewport::{ToolSurfaceRuntimeBindingRecord, ViewportRenderStateCommand};
    use asset::{asset_id, asset_source_id, asset_source_revision_id};
    use editor_core::{ChangeOrigin, CommandId, EntityId};
    use editor_scene::{
        SceneCommandIntent, SceneMaterialAssignmentState, SceneMaterialPalette, SceneMaterialSlot,
        SceneMaterialSlotId, SceneMeshMaterialRegionId, SceneModelMeshMaterialRegionSourceId,
        SceneModelMeshMaterialSlotAssignment, SceneModelMeshSourceId, SceneQuat,
        SceneSelectionAddress, SceneTransform, SceneVec3, SdfBooleanIntent, SdfPrimitiveKind,
        SdfPrimitiveMaterialSlotAssignment, SdfPrimitiveSourceId, SdfPrimitiveSpec,
    };
    use editor_shell::{PanelInstanceId, TabStackId, ToolSurfaceInstanceId, WidgetId};
    use editor_viewport::ViewportId;
    use scene::Vec3Value;

    fn create_sdf_primitive(
        runtime: &mut crate::editor_runtime::RunenwerkEditorRuntime,
        command_id: u64,
        display_name: &str,
        translation: SceneVec3,
        kind: SdfPrimitiveKind,
    ) -> EntityId {
        let before = runtime.document().entity_ids().collect::<Vec<_>>();
        execute_scene_intent(
            runtime,
            CommandId(command_id),
            SceneCommandIntent::CreateSdfPrimitive {
                parent: None,
                display_name: display_name.to_string(),
                primitive: SdfPrimitiveSpec::new(kind, SdfBooleanIntent::Add).with_transform(
                    SceneTransform::new(translation, SceneQuat::identity(), SceneVec3::one()),
                ),
            },
        )
        .expect("SDF primitive creation should succeed");

        runtime
            .document()
            .entity_ids()
            .find(|entity| !before.contains(entity))
            .expect("SDF primitive creation should register one entity")
    }

    fn binding(
        surface: u64,
        panel: u64,
        stack: u64,
        viewport_id: ViewportId,
        bounds: UiRect,
    ) -> ToolSurfaceRuntimeBindingRecord {
        ToolSurfaceRuntimeBindingRecord {
            presentation_target_id: ui_composition::PresentationTargetId::try_from_raw(1).unwrap(),
            tool_surface_id: ToolSurfaceInstanceId::try_from_raw(surface).unwrap(),
            panel_instance_id: PanelInstanceId::try_from_raw(panel).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(stack).unwrap(),
            viewport_id,
            host_widget_id: WidgetId(10_000 + surface),
            bounds,
            effective_shell_scale: 1.0,
            generation: 1,
        }
    }

    fn host_with_fresh_secondary_target() -> (
        EditorHostResource,
        PresentationTargetId,
        PresentationTargetId,
    ) {
        use crate::shell::{EditorCompositionPolicy, EditorWindowPresentationBinding};
        use editor_shell::{
            EditorFreshTargetRequest, WorkspaceProfileLayoutSource,
            plan_editor_fresh_profile_target,
        };
        use engine::plugins::render::backend::RenderSurfaceId;
        use engine::runtime::NativeWindowId;
        use ui_composition::{CompositionPolicies, TargetProfileId};

        let mut host = EditorHostResource::default();
        let primary_target = host.shell_state.primary_composition_target_id();
        let profile_id = host.shell_state.active_workspace_profile_id();
        let layout = match &host
            .app
            .workbench_host()
            .workspace_profile(profile_id)
            .expect("active workspace profile should be installed")
            .layout_source
        {
            WorkspaceProfileLayoutSource::AuthoredLayout { layout, .. } => layout.clone(),
            other => panic!("fresh target requires normalized authored layout, got {other:?}"),
        };
        let request = EditorFreshTargetRequest::new(profile_id, layout);
        let plan = plan_editor_fresh_profile_target(
            host.shell_state.composition_runtime(),
            &request,
            host.app.workbench_host().tool_surface_registry(),
            host.shell_state.composition_identity_allocator(),
            TargetProfileId::new("runenwerk.editor.desktop")
                .expect("editor desktop target profile should be valid"),
        )
        .expect("fresh secondary target should plan");
        let secondary_target = plan.created_target;
        let identities = plan.identities;
        let policy = EditorCompositionPolicy;
        let prepared = host
            .shell_state
            .composition_runtime()
            .prepare_change(
                plan.change,
                CompositionPolicies {
                    lifecycle: &policy,
                    capability: &policy,
                    target: &policy,
                },
            )
            .expect("fresh secondary target should prepare");

        let secondary_window = host.shell_state.open_editor_window_for_active_workspace();
        let secondary_binding = EditorWindowPresentationBinding {
            native_window_id: NativeWindowId::try_from_raw(2)
                .expect("secondary native window id should be valid"),
            render_surface_id: RenderSurfaceId::try_from_raw(2)
                .expect("secondary render surface id should be valid"),
        };
        assert!(
            host.shell_state
                .bind_editor_window_presentation(secondary_window, secondary_binding)
        );
        host.shell_state
            .commit_prepared_composition(prepared, Some((secondary_target, secondary_binding)))
            .expect("fresh secondary target should commit");
        host.shell_state
            .replace_composition_identity_allocator(identities);

        (host, primary_target, secondary_target)
    }

    #[test]
    fn all_target_projection_preserves_secondary_binding_scale_and_prunes_removed_target() {
        let (mut host, primary_target, secondary_target) = host_with_fresh_secondary_target();
        let mut viewport_instances = ViewportInstanceRegistryResource::default();
        viewport_instances.sync_from_composition(host.shell_state.composition_runtime());

        let atlas = UiFontAtlasResource::default();
        let target_presentations = [
            (
                primary_target,
                UiRect::new(0.0, 0.0, 1280.0, 720.0),
                1.0_f64,
            ),
            (
                secondary_target,
                UiRect::new(0.0, 0.0, 900.0, 600.0),
                1.75_f64,
            ),
        ];
        for (target_id, bounds, scale_factor) in target_presentations {
            let shell_theme = scaled_shell_theme(&host.theme, scale_factor);
            let EditorHostResource {
                app, shell_state, ..
            } = &mut host;
            app.build_shell_expression_frame_for_target_with_surface_resources(
                shell_state,
                target_id,
                bounds,
                &shell_theme,
                &atlas,
                None,
                None,
                Some(&viewport_instances),
                None,
            )
            .unwrap_or_else(|| panic!("target {} should build a shell frame", target_id.raw()));
        }

        let primary_scale = effective_shell_scale(1.0);
        let secondary_scale = effective_shell_scale(1.75);
        let target_scales = [
            (primary_target, primary_scale),
            (secondary_target, secondary_scale),
        ];
        let mut layout = ViewportLayoutMapResource::default();
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        let mut render_states = ViewportRenderStateResource::default();

        rebuild_viewport_runtime_projection(
            &mut host,
            &viewport_instances,
            &mut layout,
            &mut bindings,
            &mut render_states,
            &target_scales,
        );

        assert!(
            layout
                .entries()
                .any(|entry| entry.presentation_target_id == primary_target),
            "primary target should contribute viewport layout"
        );
        assert!(
            layout
                .entries()
                .any(|entry| entry.presentation_target_id == secondary_target),
            "secondary target should contribute viewport layout"
        );

        let projected = bindings.bindings().collect::<Vec<_>>();
        let primary = projected
            .iter()
            .find(|binding| binding.presentation_target_id == primary_target)
            .copied()
            .expect("primary target should retain a viewport binding");
        let secondary = projected
            .iter()
            .find(|binding| binding.presentation_target_id == secondary_target)
            .copied()
            .expect("secondary target should retain a viewport binding");

        assert_ne!(
            primary.tool_surface_id, secondary.tool_surface_id,
            "fresh target must retain a distinct Tool Surface identity"
        );
        assert_ne!(
            primary.viewport_id, secondary.viewport_id,
            "fresh target must retain a distinct viewport identity"
        );
        assert_eq!(primary.effective_shell_scale, primary_scale);
        assert_eq!(secondary.effective_shell_scale, secondary_scale);
        assert_eq!(
            render_states
                .state_for(primary.viewport_id)
                .expect("primary viewport render state should survive")
                .render_state
                .effective_shell_scale,
            primary_scale
        );
        assert_eq!(
            render_states
                .state_for(secondary.viewport_id)
                .expect("secondary viewport render state should survive")
                .render_state
                .effective_shell_scale,
            secondary_scale
        );

        rebuild_viewport_runtime_projection(
            &mut host,
            &viewport_instances,
            &mut layout,
            &mut bindings,
            &mut render_states,
            &[(primary_target, primary_scale)],
        );

        assert!(
            bindings
                .bindings()
                .all(|binding| binding.presentation_target_id == primary_target),
            "removing a secondary target from the accepted projection must prune its binding"
        );
        assert!(
            render_states.state_for(primary.viewport_id).is_some(),
            "primary viewport state must survive secondary-target pruning"
        );
        assert!(
            render_states.state_for(secondary.viewport_id).is_none(),
            "secondary viewport state must be pruned when no longer represented by the projection"
        );
    }

    #[test]
    fn viewport_render_states_follow_tool_surface_bindings() {
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        let first = ViewportId(2);
        let second = ViewportId(3);
        bindings.upsert_binding(binding(1, 1, 1, first, UiRect::new(0.0, 0.0, 320.0, 240.0)));
        bindings.upsert_binding(binding(
            2,
            2,
            2,
            second,
            UiRect::new(320.0, 0.0, 480.0, 240.0),
        ));
        let mut render_states = ViewportRenderStateResource::default();
        let app = crate::editor_app::RunenwerkEditorApp::new();

        sync_viewport_render_states_from_bindings(
            &app,
            &mut render_states,
            &bindings,
            &[],
            EditorViewportDebugStage::Scene,
            false,
        );

        assert_eq!(
            render_states.state_for(first).map(|state| state.bounds),
            Some(UiRect::new(0.0, 0.0, 320.0, 240.0)),
        );
        assert_eq!(
            render_states.state_for(second).map(|state| state.bounds),
            Some(UiRect::new(320.0, 0.0, 480.0, 240.0)),
        );

        render_states.apply_command(ViewportRenderStateCommand::SetDebugStage {
            viewport_id: second,
            debug_stage: EditorViewportDebugStage::PrimitiveAvailability,
        });
        sync_viewport_render_states_from_bindings(
            &app,
            &mut render_states,
            &bindings,
            &[],
            EditorViewportDebugStage::Scene,
            false,
        );

        assert_eq!(
            render_states
                .state_for(second)
                .map(|state| state.render_state.debug_stage),
            Some(EditorViewportDebugStage::PrimitiveAvailability),
        );
    }

    #[test]
    fn extracted_scene_packet_includes_all_primitives_with_selection_and_hover_flags() {
        let mut runtime = crate::editor_runtime::RunenwerkEditorRuntime::new();
        register_mvp_component_types(&mut runtime);
        let hovered = create_sdf_primitive(
            &mut runtime,
            10,
            "Hovered",
            SceneVec3::new(-1.0, 0.0, 0.0),
            SdfPrimitiveKind::Box,
        );
        let selected = create_sdf_primitive(
            &mut runtime,
            11,
            "Selected",
            SceneVec3::new(2.0, 0.0, 0.0),
            SdfPrimitiveKind::Sphere,
        );
        runtime.set_selection_single_with_origin(
            SceneSelectionAddress::entity(runtime.scene_selection().scope(), selected),
            ChangeOrigin::Runtime,
        );

        let packet = extract_viewport_scene_render_packet(&runtime, Some(hovered));
        let primitives = packet.primitives();

        assert_eq!(primitives.len(), 2);
        assert_eq!(primitives[0].entity_id, hovered);
        assert_eq!(primitives[1].entity_id, selected);
        assert!(primitives[0].hovered);
        assert!(!primitives[0].selected);
        assert!(primitives[1].selected);
        assert!(!primitives[1].hovered);
        assert_eq!(primitives[0].translation, Vec3Value::new(-1.0, 0.0, 0.0));
        assert_eq!(
            primitives[1].primitive_kind,
            crate::editor_runtime::EditorPrimitiveKind::Sphere
        );
    }

    #[test]
    fn sdf_assignment_identity_survives_viewport_extraction() {
        let mut runtime = crate::editor_runtime::RunenwerkEditorRuntime::new();
        register_mvp_component_types(&mut runtime);
        let left = create_sdf_primitive(
            &mut runtime,
            20,
            "Left",
            SceneVec3::new(-1.0, 0.0, 0.0),
            SdfPrimitiveKind::Box,
        );
        let right = create_sdf_primitive(
            &mut runtime,
            21,
            "Right",
            SceneVec3::new(1.0, 0.0, 0.0),
            SdfPrimitiveKind::Sphere,
        );
        let slot_two = SceneMaterialSlotId::new(2);
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(slot_two, "Right Slot"),
        ])
        .expect("valid palette");
        let assignments = SceneMaterialAssignmentState::new(
            palette,
            [SdfPrimitiveMaterialSlotAssignment::new(
                SdfPrimitiveSourceId::new(right),
                slot_two,
            )],
        )
        .expect("valid material assignment state");
        runtime.replace_scene_material_assignments(assignments);

        let scene_file = crate::persistence::scene_file_from_runtime(&runtime);
        let mut restored = crate::editor_runtime::RunenwerkEditorRuntime::new();
        register_mvp_component_types(&mut restored);
        crate::persistence::apply_scene_file_to_runtime(&mut restored, &scene_file)
            .expect("scene file reload should preserve SDF material assignments");

        let packet =
            extract_viewport_scene_render_packet_with_material_slots(&restored, None, |entity| {
                restored.material_slot_index_for_entity(entity)
            });
        let primitives = packet.primitives();

        let left_packet = primitives
            .iter()
            .find(|primitive| primitive.entity_id == left)
            .expect("left primitive should survive extraction");
        let right_packet = primitives
            .iter()
            .find(|primitive| primitive.entity_id == right)
            .expect("right primitive should survive extraction");
        assert_eq!(left_packet.material_slot_index, 0);
        assert_eq!(right_packet.material_slot_index, 1);
    }

    #[test]
    fn sdf_two_primitives_render_different_material_slots() {
        let mut runtime = crate::editor_runtime::RunenwerkEditorRuntime::new();
        register_mvp_component_types(&mut runtime);
        let first = create_sdf_primitive(
            &mut runtime,
            30,
            "First",
            SceneVec3::new(-0.75, 0.0, 0.0),
            SdfPrimitiveKind::Box,
        );
        let second = create_sdf_primitive(
            &mut runtime,
            31,
            "Second",
            SceneVec3::new(0.75, 0.0, 0.0),
            SdfPrimitiveKind::Sphere,
        );
        let slot_two = SceneMaterialSlotId::new(2);
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(slot_two, "Second Slot"),
        ])
        .expect("valid palette");
        let assignments = SceneMaterialAssignmentState::new(
            palette,
            [SdfPrimitiveMaterialSlotAssignment::new(
                SdfPrimitiveSourceId::new(second),
                slot_two,
            )],
        )
        .expect("valid material assignment state");
        runtime.replace_scene_material_assignments(assignments);

        let packet =
            extract_viewport_scene_render_packet_with_material_slots(&runtime, None, |entity| {
                runtime.material_slot_index_for_entity(entity)
            });
        let first_slot = packet
            .primitives()
            .iter()
            .find(|primitive| primitive.entity_id == first)
            .expect("first primitive should render")
            .material_slot_index;
        let second_slot = packet
            .primitives()
            .iter()
            .find(|primitive| primitive.entity_id == second)
            .expect("second primitive should render")
            .material_slot_index;

        assert_eq!(first_slot, 0);
        assert_eq!(second_slot, 1);
        assert_ne!(
            first_slot, second_slot,
            "two SDF primitives must reach the renderer with distinct material table slots"
        );
    }

    #[test]
    fn model_mesh_renderable_uses_source_backed_material_slot() {
        let mut runtime = crate::editor_runtime::RunenwerkEditorRuntime::new();
        register_mvp_component_types(&mut runtime);
        let assigned_slot = SceneMaterialSlotId::new(2);
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(assigned_slot, "Imported Body").with_material_asset(asset_id(7)),
        ])
        .expect("valid palette");
        let material_region = SceneModelMeshMaterialRegionSourceId::new(
            SceneModelMeshSourceId::new(asset_id(42), asset_source_id(84))
                .with_source_revision_id(asset_source_revision_id(2))
                .with_source_revision("sha256:source"),
            SceneMeshMaterialRegionId::new("source_material_slot:0")
                .expect("source material slot key should be stable"),
        );
        let assignments = SceneMaterialAssignmentState::new_with_model_mesh_assignments(
            palette,
            [],
            [SceneModelMeshMaterialSlotAssignment::new(
                material_region.clone(),
                assigned_slot,
            )],
        )
        .expect("valid material assignment state");
        runtime.replace_scene_material_assignments(assignments.clone());

        let packet = EditorViewportModelMeshMaterialSelectionPacket::from_model_mesh_regions(
            runtime.scene_material_assignments(),
            [material_region.clone()],
        );
        let selection = packet
            .selections()
            .first()
            .expect("source-backed model/mesh material surface should prepare");
        let prepared_selection = selection.prepared_selection().clone();

        assert_eq!(selection.material_table_index, 1);
        assert_eq!(
            prepared_selection.surface.source.asset_id,
            asset_id(42).raw()
        );
        assert_eq!(
            prepared_selection.surface.source.source_id,
            asset_source_id(84).raw()
        );
        assert_eq!(
            prepared_selection.surface.source.source_revision_id,
            Some(asset_source_revision_id(2).raw())
        );
        assert_eq!(
            prepared_selection.surface.source.source_revision.as_deref(),
            Some("sha256:source")
        );
        assert_eq!(
            prepared_selection.surface.region_key,
            "source_material_slot:0"
        );
        assert!(
            !prepared_selection
                .surface
                .identity_key()
                .contains("renderable_index")
        );

        let mut app = crate::editor_app::RunenwerkEditorApp::new();
        app.runtime_mut()
            .replace_scene_material_assignments(assignments);
        let mut render_state = EditorViewportRenderState::default();
        populate_viewport_render_state(
            &app,
            &mut render_state,
            UiRect::new(0.0, 0.0, 320.0, 240.0),
        );

        assert_eq!(
            render_state
                .model_mesh_material_selection_packet
                .prepared_material_selections(),
            vec![prepared_selection]
        );
    }
}
