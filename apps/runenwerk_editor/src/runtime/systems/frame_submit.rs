use editor_shell::{
    ComputedLayoutMap, UiNode, UiNodeKind, WorkspaceState, viewport_embed_slot_for,
};
use editor_viewport::ViewportSurfacePresentationSlot;
use engine::WindowState;
use engine::plugins::render::{
    EditorPickingTarget, UiFontAtlasResource, UiFrameProducerId, UiFrameRoute, UiFrameSubmission,
    UiFrameSubmissionOrder, UiFrameSubmissionRegistryResource,
};
use engine::runtime::{Res, ResMut};
use scene::LocalTransform;
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

const EDITOR_SHELL_UI_PRODUCER_ID: UiFrameProducerId = ui_frame_producer_id(1001);
const DEBUG_HARDCODED_UI_FRAME_ENV: &str = "RUNENWERK_EDITOR_DEBUG_UI_FRAME";
const VIEWPORT_DEBUG_STAGE_ENV: &str = "RUNENWERK_EDITOR_VIEWPORT_DEBUG_STAGE";
const VIEWPORT_ROOT_OPAQUE_ENV: &str = "RUNENWERK_EDITOR_VIEWPORT_ROOT_OPAQUE";
const VIEWPORT_BRANCH_TRACE_ENV: &str = "RUNENWERK_EDITOR_VIEWPORT_BRANCH_TRACE";

const fn ui_frame_producer_id(raw: u64) -> UiFrameProducerId {
    match UiFrameProducerId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("ui frame producer id constants must be non-zero"),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn submit_editor_frame_system(
    window: Res<WindowState>,
    debug_metrics: Res<engine::DebugMetricsState>,
    mut host: ResMut<EditorHostResource>,
    mut viewport_render_states: ResMut<ViewportRenderStateResource>,
    viewport_observations: Res<ViewportArtifactObservationResource>,
    viewport_instances: Res<ViewportInstanceRegistryResource>,
    mut viewport_layout_map: ResMut<ViewportLayoutMapResource>,
    mut tool_surface_bindings: ResMut<ToolSurfaceRuntimeBindingRegistryResource>,
    mut mounted_surfaces: ResMut<MountedSurfaceRegistryResource>,
    atlas: Res<UiFontAtlasResource>,
    viewport_picking_results: Res<ViewportPickingResultsResource>,
    mut submissions: ResMut<UiFrameSubmissionRegistryResource>,
) {
    let bounds = window_bounds(&window);
    let shell_scale = effective_shell_scale(window.scale_factor);
    host.apply_pending_editor_definition_activations();
    let EditorHostResource {
        app,
        shell_state,
        theme,
    } = &mut *host;
    let shell_theme = scaled_shell_theme(theme, window.scale_factor);
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
        let expression = app.build_shell_expression_frame_with_surface_resources(
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
        );
        (
            expression.metadata.source_version,
            expression.into_ui_frame(),
        )
    };
    let rendered_viewport_bounds = primary_viewport_bounds_from_frame(&frame);
    let viewport_bounds = active_viewport_id
        .and_then(|viewport_id| viewport_bounds_from_frame(&frame, viewport_id.0))
        .or_else(|| rendered_viewport_bounds.first().copied())
        .or_else(|| {
            viewport_bounds(
                shell_state.last_tree(),
                shell_state.last_bounds(),
                shell_state.runtime(),
            )
        })
        .or_else(|| viewport_bounds_from_render_states(&viewport_render_states))
        .unwrap_or(bounds);
    viewport_layout_map.clear();
    populate_viewport_layout_map_from_shell_tree(
        shell_state,
        &mut viewport_layout_map,
        viewport_bounds,
    );
    tool_surface_bindings
        .rebuild_from_layout_map_with_instances(&viewport_layout_map, &viewport_instances);
    sync_viewport_render_states_from_bindings(
        app,
        shell_state.workspace_state(),
        &mut viewport_render_states,
        &tool_surface_bindings,
        shell_scale,
        viewport_debug_stage(),
        root_background_opaque_enabled(),
    );
    mounted_surfaces.sync_from_workspace_state(shell_state.workspace_state());
    if app.debug_logs_enabled() {
        for rebind in tool_surface_bindings.latest_rebinds() {
            app.append_console_line(format!(
                "[viewport.binding] rebind tool_surface={} from_viewport={} to_viewport={}",
                rebind.tool_surface_id.raw(),
                rebind.from_viewport_id.0,
                rebind.to_viewport_id.0
            ));
        }
    }

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
            Some(
                viewport_render
                    .branch_trace_snapshot((window.size_px.0.max(1), window.size_px.1.max(1))),
            )
        } else {
            None
        };

        if app.debug_logs_enabled() {
            if viewport_render.should_report_scale_change() {
                app.append_console_line(format!(
                    "[ui] shell scale={:.3} window_scale={:.3} expression_version={}",
                    shell_scale, window.scale_factor, expression_source_version.0
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

    submissions.replace(
        UiFrameSubmission::new(EDITOR_SHELL_UI_PRODUCER_ID)
            .with_route(UiFrameRoute::Screen)
            .with_order(UiFrameSubmissionOrder::new(10, 0))
            .with_frame(frame),
    );
}

fn sync_viewport_render_states_from_bindings(
    app: &crate::editor_app::RunenwerkEditorApp,
    workspace_state: &WorkspaceState,
    viewport_render_states: &mut ViewportRenderStateResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    shell_scale: f32,
    default_debug_stage: EditorViewportDebugStage,
    default_root_background_opaque: bool,
) {
    let mut viewport_ids = std::collections::BTreeSet::new();
    for binding in tool_surface_bindings.bindings() {
        let mut render_state = viewport_render_states
            .state_for(binding.viewport_id)
            .map(|previous| previous.render_state.clone())
            .or_else(|| {
                workspace_state
                    .tool_surface(binding.tool_surface_id)
                    .and_then(|surface| surface.viewport_settings)
                    .map(EditorViewportRenderState::from_viewport_settings)
            })
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
        render_state.set_effective_shell_scale(shell_scale);
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

fn window_bounds(window: &WindowState) -> UiRect {
    let width = window.size_px.0.max(1) as f32;
    let height = window.size_px.1.max(1) as f32;
    UiRect::new(0.0, 0.0, width, height)
}

fn viewport_bounds_from_frame(frame: &UiFrame, viewport_id: u64) -> Option<UiRect> {
    frame
        .surfaces
        .iter()
        .flat_map(|surface| surface.layers.iter())
        .flat_map(|layer| layer.primitives.iter())
        .find_map(|primitive| {
            let UiPrimitive::ViewportSurfaceEmbed(embed) = primitive else {
                return None;
            };
            if embed.viewport_id == viewport_id
                && embed.slot == viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary)
            {
                Some(embed.rect)
            } else {
                None
            }
        })
}

fn primary_viewport_bounds_from_frame(frame: &UiFrame) -> Vec<UiRect> {
    frame
        .surfaces
        .iter()
        .flat_map(|surface| surface.layers.iter())
        .flat_map(|layer| layer.primitives.iter())
        .filter_map(|primitive| {
            let UiPrimitive::ViewportSurfaceEmbed(embed) = primitive else {
                return None;
            };
            (embed.slot == viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary))
                .then_some(embed.rect)
        })
        .collect()
}

fn populate_viewport_layout_map_from_shell_tree(
    shell_state: &RunenwerkEditorShellState,
    viewport_layout_map: &mut ViewportLayoutMapResource,
    fallback_bounds: UiRect,
) {
    let (Some(tree), Some(bounds), Some(artifacts)) = (
        shell_state.last_tree(),
        shell_state.last_bounds(),
        shell_state.last_projection_artifacts(),
    ) else {
        return;
    };
    let layouts = shell_state.runtime().compute_layout(tree, bounds);
    collect_viewport_layout_entries(
        &tree.root,
        &layouts,
        &artifacts.widget_structural_context_by_id,
        viewport_layout_map,
        fallback_bounds,
    );
}

fn collect_viewport_layout_entries(
    node: &UiNode,
    layouts: &ComputedLayoutMap,
    structural_contexts: &std::collections::BTreeMap<
        editor_shell::WidgetId,
        editor_shell::StructuralWidgetRoutingContext,
    >,
    viewport_layout_map: &mut ViewportLayoutMapResource,
    fallback_bounds: UiRect,
) {
    if let UiNodeKind::ViewportSurfaceEmbed(embed) = &node.kind
        && embed.slot == viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary)
        && let Some(structural_context) = structural_contexts.get(&node.id).copied()
    {
        viewport_layout_map.upsert_entry(ViewportLayoutEntry {
            viewport_id: editor_viewport::ViewportId(embed.viewport_id),
            host_widget_id: node.id,
            structural_context,
            bounds: layouts
                .get(&node.id)
                .map(|layout| layout.bounds)
                .unwrap_or(fallback_bounds),
        });
    }
    for child in &node.children {
        collect_viewport_layout_entries(
            child,
            layouts,
            structural_contexts,
            viewport_layout_map,
            fallback_bounds,
        );
    }
}

fn viewport_bounds_from_render_states(states: &ViewportRenderStateResource) -> Option<UiRect> {
    states
        .entries()
        .find(|entry| entry.bounds.width > f32::EPSILON && entry.bounds.height > f32::EPSILON)
        .map(|entry| entry.bounds)
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

fn viewport_bounds(
    tree: Option<&editor_shell::UiTree>,
    bounds: Option<UiRect>,
    runtime: &editor_shell::UiRuntime,
) -> Option<UiRect> {
    let tree = tree?;
    let bounds = bounds?;
    let layouts = runtime.compute_layout(tree, bounds);
    layouts
        .get(&editor_shell::VIEWPORT_SURFACE_EMBED_WIDGET_ID)
        .map(|layout| layout.bounds)
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
    use editor_core::{ChangeOrigin, CommandId, EntityId, SelectionTarget};
    use editor_scene::{
        SceneCommandIntent, SceneMaterialAssignmentState, SceneMaterialPalette, SceneMaterialSlot,
        SceneMaterialSlotId, SceneMeshMaterialRegionId, SceneModelMeshMaterialRegionSourceId,
        SceneModelMeshMaterialSlotAssignment, SceneModelMeshSourceId, SceneQuat, SceneTransform,
        SceneVec3, SdfBooleanIntent, SdfPrimitiveKind, SdfPrimitiveMaterialSlotAssignment,
        SdfPrimitiveSourceId, SdfPrimitiveSpec,
    };
    use editor_shell::{PanelInstanceId, TabStackId, ToolSurfaceInstanceId, WidgetId};
    use editor_viewport::ViewportId;
    use scene::Vec3Value;
    use ui_render_data::ViewportSurfaceEmbedPrimitive;

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
            tool_surface_id: ToolSurfaceInstanceId::try_from_raw(surface).unwrap(),
            panel_instance_id: PanelInstanceId::try_from_raw(panel).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(stack).unwrap(),
            viewport_id,
            host_widget_id: WidgetId(10_000 + surface),
            bounds,
            generation: 1,
        }
    }

    #[test]
    fn primary_viewport_bounds_from_frame_collects_split_viewport_embeds() {
        let first = UiRect::new(10.0, 20.0, 300.0, 200.0);
        let second = UiRect::new(330.0, 20.0, 300.0, 200.0);
        let mut layer = UiLayer::new(UiLayerId(0));
        for (order, rect) in [first, second].into_iter().enumerate() {
            layer.push(UiPrimitive::ViewportSurfaceEmbed(
                ViewportSurfaceEmbedPrimitive::new(
                    1,
                    viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary),
                    rect,
                    UiRect::new(0.0, 0.0, 1.0, 1.0),
                    UiPaint::rgba(1.0, 1.0, 1.0, 1.0),
                    UiSortKey::new(0, 0, order as u32),
                ),
            ));
        }
        layer.push(UiPrimitive::ViewportSurfaceEmbed(
            ViewportSurfaceEmbedPrimitive::new(
                1,
                viewport_embed_slot_for(ViewportSurfacePresentationSlot::Overlay),
                UiRect::new(700.0, 20.0, 300.0, 200.0),
                UiRect::new(0.0, 0.0, 1.0, 1.0),
                UiPaint::rgba(1.0, 1.0, 1.0, 1.0),
                UiSortKey::new(0, 0, 3),
            ),
        ));

        let frame = UiFrame::with_surfaces(vec![UiSurface::with_layers(
            UiSurfaceId(0),
            UiRect::new(0.0, 0.0, 1280.0, 720.0).size(),
            vec![layer],
        )]);

        assert_eq!(
            primary_viewport_bounds_from_frame(&frame),
            vec![first, second]
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
        let mut allocator = editor_shell::WorkspaceIdentityAllocator::new();
        let workspace = editor_shell::WorkspaceState::bootstrap_current_layout(
            editor_shell::WorkspaceId::try_from_raw(1).unwrap(),
            &mut allocator,
        );

        sync_viewport_render_states_from_bindings(
            &app,
            &workspace,
            &mut render_states,
            &bindings,
            1.0,
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
            &workspace,
            &mut render_states,
            &bindings,
            1.0,
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
            SelectionTarget::Entity(selected),
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
