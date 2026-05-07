use editor_shell::{ComputedLayoutMap, UiNode, UiNodeKind, viewport_embed_slot_for};
use editor_viewport::{
    ArtifactObservationFrame, ExpressionDimensions, ProducerHealth, ProductAvailabilityState,
    ViewportId, ViewportSurfacePresentationSlot,
};
use engine::WindowState;
use engine::plugins::render::{
    EditorPickingTarget, UiFontAtlasResource, UiFrameProducerId, UiFrameRoute, UiFrameSubmission,
    UiFrameSubmissionOrder, UiFrameSubmissionRegistryResource,
    ViewportSurfaceBindingRegistryResource,
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
    EditorHostResource, EditorViewportDebugStage, EditorViewportRenderState, effective_shell_scale,
    scaled_shell_theme,
};
use crate::runtime::viewport::{
    MAIN_VIEWPORT_ID, MountedSurfaceRegistryResource, ToolSurfaceRuntimeBindingRecord,
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportLayoutEntry, ViewportLayoutMapResource, ViewportPickingResultsResource,
    ViewportPresentationStateResource, ViewportProductRegistryResource, ViewportRenderStateEntry,
    ViewportRenderStateResource, ViewportSurfaceSetResource, build_surface_binding_registry,
    ensure_editor_main_surface_set, expression_dimensions_for_bounds, initial_presentation_state,
    initial_product_descriptors, resolve_structural_viewport_products,
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
    mut host: ResMut<EditorHostResource>,
    mut viewport_render: ResMut<EditorViewportRenderState>,
    mut viewport_render_states: ResMut<ViewportRenderStateResource>,
    viewport_observations: Res<ViewportArtifactObservationResource>,
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
    if active_viewport_id.is_some() {
        seed_viewport_binding_for_active_workspace(shell_state, &mut tool_surface_bindings, bounds);
    }
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
        .or_else(|| viewport_bounds_from_render_state(&viewport_render))
        .unwrap_or(bounds);
    let viewport_bounds_changed = populate_viewport_render_state(
        app,
        &mut viewport_render,
        viewport_bounds,
        &rendered_viewport_bounds,
    );
    viewport_layout_map.clear();
    populate_viewport_layout_map_from_shell_tree(
        shell_state,
        &mut viewport_layout_map,
        viewport_bounds,
    );
    tool_surface_bindings.rebuild_from_layout_map(&viewport_layout_map);
    sync_viewport_render_states_from_bindings(&mut viewport_render_states, &tool_surface_bindings);
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
    let viewport_valid = viewport_is_valid(viewport_bounds);
    let shader_loaded = true;
    let debug_stage = viewport_debug_stage();
    let root_background_opaque = root_background_opaque_enabled();
    viewport_render.update_visibility_diagnostics(viewport_valid, shader_loaded);
    let debug_stage_changed = viewport_render.set_debug_stage(debug_stage);
    let root_probe_changed = viewport_render.set_root_background_opaque(root_background_opaque);
    let shell_scale_changed = viewport_render.set_effective_shell_scale(shell_scale);
    let contradiction_active =
        picking_hits_entity_or_component(&viewport_picking_results, active_viewport_id)
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
        if shell_scale_changed && viewport_render.should_report_scale_change() {
            app.append_console_line(format!(
                "[ui] shell scale={:.3} window_scale={:.3} expression_version={}",
                shell_scale, window.scale_factor, expression_source_version.0
            ));
        }

        if viewport_bounds_changed && viewport_render.should_report_bounds_change() {
            app.append_console_line(format!(
                "[viewport] bounds=({:.1},{:.1},{:.1},{:.1})",
                viewport_bounds.x, viewport_bounds.y, viewport_bounds.width, viewport_bounds.height
            ));
            if viewport_bounds.width <= f32::EPSILON || viewport_bounds.height <= f32::EPSILON {
                app.append_console_line(
                    "[viewport] warning: viewport canvas bounds are zero-sized".to_string(),
                );
            }
        }

        if root_probe_changed
            || debug_stage_changed
            || viewport_render.should_report_debug_state_change()
        {
            app.append_console_line(format!(
                "[viewport] root-occlusion={} debug-stage={} viewport_valid={} shader_loaded={} primitive_visible={}",
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
            contradiction_reasons(&viewport_render)
        );
        if let Some(snapshot) = branch_trace_snapshot {
            line.push_str(" | ");
            line.push_str(&snapshot.summary_line());
        }
        app.append_console_line(line);
    }

    submissions.replace(
        UiFrameSubmission::new(EDITOR_SHELL_UI_PRODUCER_ID)
            .with_route(UiFrameRoute::Screen)
            .with_order(UiFrameSubmissionOrder::new(10, 0))
            .with_frame(frame),
    );
}

fn seed_viewport_binding_for_active_workspace(
    shell_state: &RunenwerkEditorShellState,
    tool_surface_bindings: &mut ToolSurfaceRuntimeBindingRegistryResource,
    bounds: ui_math::UiRect,
) {
    let Some((panel, surface, tab_stack)) = shell_state
        .workspace_state()
        .panels()
        .filter_map(|panel| {
            let surface_id = panel.active_tool_surface?;
            let surface = shell_state.workspace_state().tool_surface(surface_id)?;
            if surface.tool_surface_kind != editor_shell::ToolSurfaceKind::Viewport {
                return None;
            }
            let tab_stack = shell_state
                .workspace_state()
                .tab_stacks()
                .find(|stack| stack.ordered_panels.contains(&panel.id))?;
            Some((panel, surface, tab_stack))
        })
        .next()
    else {
        return;
    };

    if tool_surface_bindings
        .binding_for_tool_surface(surface.id)
        .is_some()
    {
        return;
    }

    tool_surface_bindings.upsert_binding(ToolSurfaceRuntimeBindingRecord {
        tool_surface_id: surface.id,
        panel_instance_id: panel.id,
        tab_stack_id: tab_stack.id,
        viewport_id: crate::runtime::viewport::viewport_id_for_tool_surface(surface.id),
        host_widget_id: editor_shell::VIEWPORT_SURFACE_EMBED_WIDGET_ID,
        bounds,
        generation: tool_surface_bindings.generation().saturating_add(1),
    });
}

fn sync_viewport_render_states_from_bindings(
    viewport_render_states: &mut ViewportRenderStateResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
) {
    let mut viewport_ids = std::collections::BTreeSet::new();
    for binding in tool_surface_bindings.bindings() {
        viewport_ids.insert(binding.viewport_id);
        viewport_render_states.upsert_state(ViewportRenderStateEntry {
            viewport_id: binding.viewport_id,
            tool_surface_id: Some(binding.tool_surface_id),
            bounds: binding.bounds,
        });
    }
    viewport_render_states.retain_viewports(|viewport_id| viewport_ids.contains(&viewport_id));
}

pub fn sync_viewport_presentation_products_system(
    host: Res<EditorHostResource>,
    viewport_render: Res<EditorViewportRenderState>,
    viewport_render_states: Res<ViewportRenderStateResource>,
    mut viewport_surface_sets: ResMut<ViewportSurfaceSetResource>,
    tool_surface_bindings: Res<ToolSurfaceRuntimeBindingRegistryResource>,
    mut viewport_products_registry: ResMut<ViewportProductRegistryResource>,
    mut viewport_presentations: ResMut<ViewportPresentationStateResource>,
    mut viewport_observations: ResMut<ViewportArtifactObservationResource>,
    mut viewport_surface_bindings: ResMut<ViewportSurfaceBindingRegistryResource>,
) {
    let canonical_viewport_ids =
        canonical_viewport_ids_for_sync(&viewport_surface_sets, &tool_surface_bindings);
    for viewport_id in &canonical_viewport_ids {
        ensure_editor_main_surface_set(&mut viewport_surface_sets, *viewport_id);
    }
    let source_version = host.app.runtime().current_scene_reality_version();
    for viewport_id in &canonical_viewport_ids {
        let product_dimensions = product_dimensions_for_viewport(
            *viewport_id,
            &viewport_render_states,
            &viewport_render,
        );
        let descriptors = initial_product_descriptors(product_dimensions, source_version);
        viewport_products_registry.update_viewport_descriptors(*viewport_id, descriptors.clone());

        let mut presentation_state = viewport_presentations
            .state_for(*viewport_id)
            .cloned()
            .unwrap_or_else(|| initial_presentation_state(*viewport_id));
        if !descriptors
            .iter()
            .any(|descriptor| descriptor.id == presentation_state.selected_primary_product_id)
        {
            presentation_state.select_primary_product(
                initial_presentation_state(*viewport_id).selected_primary_product_id,
            );
        }
        viewport_presentations.upsert_state(presentation_state.clone());
        viewport_observations.upsert_frame(build_artifact_observation_frame(
            &descriptors,
            &presentation_state,
            source_version,
        ));
    }

    let viewport_id_set = canonical_viewport_ids
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    viewport_products_registry.retain_viewports(|viewport_id| {
        viewport_id == MAIN_VIEWPORT_ID || viewport_id_set.contains(&viewport_id)
    });
    viewport_presentations.retain_viewports(|viewport_id| {
        viewport_id == MAIN_VIEWPORT_ID || viewport_id_set.contains(&viewport_id)
    });
    viewport_observations.retain_viewports(|viewport_id| {
        viewport_id == MAIN_VIEWPORT_ID || viewport_id_set.contains(&viewport_id)
    });
    viewport_surface_sets.retain_viewports(|viewport_id| {
        viewport_id == MAIN_VIEWPORT_ID || viewport_id_set.contains(&viewport_id)
    });

    viewport_surface_bindings.replace_registry(build_surface_binding_registry(
        &viewport_surface_sets,
        &viewport_presentations,
    ));
}

fn canonical_viewport_ids_for_sync(
    viewport_surface_sets: &ViewportSurfaceSetResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
) -> Vec<ViewportId> {
    let mut viewport_ids = tool_surface_bindings
        .bindings()
        .map(|binding| binding.viewport_id)
        .collect::<std::collections::BTreeSet<_>>();
    if viewport_ids.is_empty() {
        viewport_ids.extend(viewport_surface_sets.viewport_ids());
    }
    viewport_ids.into_iter().collect()
}

fn product_dimensions_for_viewport(
    viewport_id: ViewportId,
    viewport_render_states: &ViewportRenderStateResource,
    fallback_render_state: &EditorViewportRenderState,
) -> ExpressionDimensions {
    viewport_render_states
        .state_for(viewport_id)
        .map(|state| expression_dimensions_for_bounds(state.bounds))
        .unwrap_or_else(|| {
            ExpressionDimensions::new(
                fallback_render_state.viewport_bounds_px.2.max(1.0).round() as u32,
                fallback_render_state.viewport_bounds_px.3.max(1.0).round() as u32,
            )
        })
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

fn viewport_bounds_from_render_state(state: &EditorViewportRenderState) -> Option<UiRect> {
    let (x, y, width, height) = state.viewport_bounds_px;
    if width > f32::EPSILON && height > f32::EPSILON {
        Some(UiRect::new(x, y, width, height))
    } else {
        None
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
    rendered_viewport_bounds: &[UiRect],
) -> bool {
    let mut render_bounds = vec![viewport_bounds_tuple(viewport_bounds)];
    for bounds in rendered_viewport_bounds.iter().copied() {
        let bounds = viewport_bounds_tuple(bounds);
        if !render_bounds
            .iter()
            .any(|existing| viewport_bounds_tuple_eq(*existing, bounds))
        {
            render_bounds.push(bounds);
        }
    }
    let bounds_changed = render_state.set_viewport_bounds_list(&render_bounds);

    let runtime = app.runtime();
    if let Some((transform, primitive)) = selected_or_first_editor_primitive(runtime) {
        render_state.set_primitive(transform.translation, primitive);
    } else {
        render_state.clear_primitive();
    }

    bounds_changed
}

fn viewport_bounds_tuple(bounds: UiRect) -> (f32, f32, f32, f32) {
    (bounds.x, bounds.y, bounds.width, bounds.height)
}

fn viewport_bounds_tuple_eq(left: (f32, f32, f32, f32), right: (f32, f32, f32, f32)) -> bool {
    (left.0 - right.0).abs() <= f32::EPSILON
        && (left.1 - right.1).abs() <= f32::EPSILON
        && (left.2 - right.2).abs() <= f32::EPSILON
        && (left.3 - right.3).abs() <= f32::EPSILON
}

fn selected_or_first_editor_primitive(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
) -> Option<(LocalTransform, EditorPrimitive)> {
    if let Some(selected) = runtime.selected_entity()
        && let Some(result) = entity_primitive(runtime, selected)
    {
        return Some(result);
    }

    runtime
        .document()
        .entity_ids()
        .find_map(|entity| entity_primitive(runtime, entity))
}

fn entity_primitive(
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

fn build_artifact_observation_frame(
    descriptors: &[editor_viewport::ExpressionProductDescriptor],
    presentation_state: &editor_viewport::ViewportPresentationState,
    source_version: editor_core::RealityVersion,
) -> ArtifactObservationFrame {
    let mut frame = ArtifactObservationFrame::new(presentation_state.viewport_id, source_version);
    frame.available_products = descriptors.to_vec();
    frame.selected_primary_product_id = Some(presentation_state.selected_primary_product_id);
    frame.selected_overlay_product_ids = presentation_state.selected_overlay_product_ids.clone();

    for descriptor in descriptors {
        frame
            .availability_by_product
            .insert(descriptor.id, ProductAvailabilityState::Available);
        frame
            .producer_health_by_product
            .insert(descriptor.id, ProducerHealth::Healthy);
    }

    if let std::collections::btree_map::Entry::Vacant(e) = frame
        .availability_by_product
        .entry(presentation_state.selected_primary_product_id)
    {
        e.insert(ProductAvailabilityState::Unavailable);
        frame.producer_health_by_product.insert(
            presentation_state.selected_primary_product_id,
            ProducerHealth::Unavailable,
        );
    }

    frame
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::viewport::{
        MAIN_VIEWPORT_ID, ViewportSurfaceHandle, ViewportSurfaceSlot, viewport_id_for_tool_surface,
    };
    use editor_shell::{PanelInstanceId, TabStackId, ToolSurfaceInstanceId, WidgetId};
    use ui_render_data::ViewportSurfaceEmbedPrimitive;

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
    fn viewport_presentation_sync_uses_runtime_bindings_over_bootstrap_surface_set() {
        let mut surface_sets = ViewportSurfaceSetResource::default();
        surface_sets.set_surface(
            MAIN_VIEWPORT_ID,
            ViewportSurfaceSlot::PrimaryColor,
            ViewportSurfaceHandle::new("flow", "bootstrap"),
        );
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        let first = viewport_id_for_tool_surface(ToolSurfaceInstanceId::try_from_raw(1).unwrap());
        let second = viewport_id_for_tool_surface(ToolSurfaceInstanceId::try_from_raw(2).unwrap());
        bindings.upsert_binding(binding(1, 1, 1, first, UiRect::new(0.0, 0.0, 320.0, 240.0)));
        bindings.upsert_binding(binding(
            2,
            2,
            2,
            second,
            UiRect::new(320.0, 0.0, 480.0, 240.0),
        ));

        assert_eq!(
            canonical_viewport_ids_for_sync(&surface_sets, &bindings),
            vec![first, second],
        );
    }

    #[test]
    fn viewport_render_states_follow_tool_surface_bindings() {
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        let first = viewport_id_for_tool_surface(ToolSurfaceInstanceId::try_from_raw(1).unwrap());
        let second = viewport_id_for_tool_surface(ToolSurfaceInstanceId::try_from_raw(2).unwrap());
        bindings.upsert_binding(binding(1, 1, 1, first, UiRect::new(0.0, 0.0, 320.0, 240.0)));
        bindings.upsert_binding(binding(
            2,
            2,
            2,
            second,
            UiRect::new(320.0, 0.0, 480.0, 240.0),
        ));
        let mut render_states = ViewportRenderStateResource::default();

        sync_viewport_render_states_from_bindings(&mut render_states, &bindings);

        assert_eq!(
            render_states.state_for(first).map(|state| state.bounds),
            Some(UiRect::new(0.0, 0.0, 320.0, 240.0)),
        );
        assert_eq!(
            render_states.state_for(second).map(|state| state.bounds),
            Some(UiRect::new(320.0, 0.0, 480.0, 240.0)),
        );
    }
}
