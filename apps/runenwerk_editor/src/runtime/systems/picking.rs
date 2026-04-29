use editor_core::{EntityId, ToolId};
use engine::plugins::render::{EditorGizmoAxis, EditorPickingHit, EditorPickingTarget};
use engine::runtime::{Res, ResMut};
use glam::{Vec2, Vec3, vec2, vec3};
use scene::{LocalTransform, Vec3Value};
use ui_math::{UiPoint, UiRect};

use crate::editor_runtime::{EditorPrimitive, RunenwerkEditorRuntime};
use crate::runtime::resources::{
    EditorHostResource, EditorViewportCamera, editor_viewport_camera,
    editor_viewport_camera_fov_y_radians,
};
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportPickingResultsResource,
};
use crate::shell::TRANSLATE_TOOL_ID;

const GRID_EPSILON: f32 = 1e-5;
const GIZMO_AXIS_LENGTH: f32 = 1.25;
const GIZMO_AXIS_PICK_RADIUS_PX: f32 = 10.0;
const HIT_DISTANCE_EPSILON: f32 = 0.0001;

#[derive(Debug, Clone, Copy)]
struct PickingRay {
    origin: Vec3,
    direction: Vec3,
}

#[derive(Debug, Clone, Copy)]
struct AxisScreenHit {
    axis: EditorGizmoAxis,
    ray_distance: f32,
    screen_distance_px: f32,
}

pub fn produce_editor_picking_system(
    input: Res<engine::plugins::InputState>,
    mut host: ResMut<EditorHostResource>,
    mut viewport_picking_results: ResMut<ViewportPickingResultsResource>,
    tool_surface_bindings: Res<ToolSurfaceRuntimeBindingRegistryResource>,
) {
    let cursor = UiPoint::new(input.mouse_position.0, input.mouse_position.1);
    let routed_viewport = routed_viewport_bounds(&host, &tool_surface_bindings, cursor);
    if let Some((viewport_id, viewport_bounds)) = routed_viewport {
        let previous_hit = viewport_picking_results
            .result_for(viewport_id)
            .map(|value| value.hit)
            .unwrap_or_else(EditorPickingHit::none);
        let next_hit = if let Some(ray) = viewport_ray(cursor, viewport_bounds) {
            compose_picking_hit(
                host.app.runtime(),
                host.app.runtime().session().active_tool(),
                host.app.runtime().selected_entity(),
                cursor,
                viewport_bounds,
                ray,
            )
        } else {
            EditorPickingHit::none()
        };
        let cursor_viewport_bounds = (
            viewport_bounds.x,
            viewport_bounds.y,
            viewport_bounds.width,
            viewport_bounds.height,
        );
        viewport_picking_results.set_viewport_result(
            viewport_id,
            (cursor.x, cursor.y),
            cursor_viewport_bounds,
            next_hit,
        );

        if host.app.debug_logs_enabled() && hit_changed(previous_hit, next_hit) {
            host.app.append_console_line(format!(
                "[pick] viewport={} cursor=({:.1},{:.1}) local=({:.1},{:.1}) hit={} dist={:.3}",
                viewport_id.0,
                cursor.x,
                cursor.y,
                cursor.x - viewport_bounds.x,
                cursor.y - viewport_bounds.y,
                picking_target_label(next_hit.target),
                next_hit.distance
            ));
        }
    } else {
        viewport_picking_results.clear_all_hits((cursor.x, cursor.y));
    }
}

fn compose_picking_hit(
    runtime: &RunenwerkEditorRuntime,
    active_tool: Option<ToolId>,
    selected_entity: Option<EntityId>,
    cursor: UiPoint,
    viewport_bounds: UiRect,
    ray: PickingRay,
) -> EditorPickingHit {
    if active_tool == Some(TRANSLATE_TOOL_ID)
        && let Some(selected) = selected_entity
        && let Some(transform) = entity_transform(runtime, selected)
        && let Some(axis_hit) = pick_gizmo_axis(cursor, viewport_bounds, transform.translation)
    {
        return EditorPickingHit {
            target: EditorPickingTarget::GizmoAxis(axis_hit.axis),
            distance: axis_hit.ray_distance.max(0.0),
        };
    }

    let entity_hit = pick_entity_hit(runtime, ray);
    let grid_hit = pick_grid_hit(ray);
    choose_primary_hit(entity_hit, grid_hit)
}

fn choose_primary_hit(
    entity_hit: Option<EditorPickingHit>,
    grid_hit: Option<EditorPickingHit>,
) -> EditorPickingHit {
    match (entity_hit, grid_hit) {
        // Selection UX priority: when an entity is under the cursor, prefer entity hits over
        // grid fallback hits to avoid near-surface misses around floor intersections.
        (Some(entity), Some(_grid)) => entity,
        (Some(entity), None) => entity,
        (None, Some(grid)) => grid,
        (None, None) => EditorPickingHit::none(),
    }
}

fn pick_entity_hit(runtime: &RunenwerkEditorRuntime, ray: PickingRay) -> Option<EditorPickingHit> {
    let mut best: Option<EditorPickingHit> = None;

    for entity in runtime.document().entity_ids() {
        let Some(ecs_entity) = runtime.ids().resolve_entity(entity) else {
            continue;
        };
        let Some(transform) = runtime.world().get::<LocalTransform>(ecs_entity).copied() else {
            continue;
        };
        let Some(primitive) = runtime.world().get::<EditorPrimitive>(ecs_entity).copied() else {
            continue;
        };

        let half_extents =
            scaled_half_extents(primitive.box_extents_for_picking(), transform.scale);
        let center = transform.translation.to_glam();
        let min = center - half_extents;
        let max = center + half_extents;

        let Some(distance) = ray_aabb_first_hit(ray.origin, ray.direction, min, max) else {
            continue;
        };
        let candidate = EditorPickingHit {
            target: EditorPickingTarget::Entity(entity.0),
            distance,
        };

        let replace = best
            .map(|current| candidate.distance < current.distance)
            .unwrap_or(true);
        if replace {
            best = Some(candidate);
        }
    }

    best
}

fn pick_grid_hit(ray: PickingRay) -> Option<EditorPickingHit> {
    if ray.direction.y.abs() <= GRID_EPSILON {
        return None;
    }

    let distance = -ray.origin.y / ray.direction.y;
    if distance < 0.0 {
        return None;
    }

    Some(EditorPickingHit {
        target: EditorPickingTarget::Grid,
        distance,
    })
}

fn pick_gizmo_axis(
    cursor: UiPoint,
    viewport_bounds: UiRect,
    center: Vec3Value,
) -> Option<AxisScreenHit> {
    let camera = editor_viewport_camera();
    let center_world = center.to_glam();
    let center_screen = project_world_to_screen(
        center_world,
        camera,
        viewport_bounds,
        editor_viewport_camera_fov_y_radians(),
    )?;
    let cursor_vec = vec2(cursor.x, cursor.y);

    let mut best: Option<AxisScreenHit> = None;
    for (axis, direction) in [
        (EditorGizmoAxis::X, vec3(1.0, 0.0, 0.0)),
        (EditorGizmoAxis::Y, vec3(0.0, 1.0, 0.0)),
        (EditorGizmoAxis::Z, vec3(0.0, 0.0, 1.0)),
    ] {
        let end_world = center_world + direction * GIZMO_AXIS_LENGTH;
        let Some(end_screen) = project_world_to_screen(
            end_world,
            camera,
            viewport_bounds,
            editor_viewport_camera_fov_y_radians(),
        ) else {
            continue;
        };

        let screen_distance = point_segment_distance(cursor_vec, center_screen, end_screen);
        if screen_distance > GIZMO_AXIS_PICK_RADIUS_PX {
            continue;
        }

        let ray_distance = (center_world - camera.position)
            .dot(camera.forward)
            .max(0.0);
        let candidate = AxisScreenHit {
            axis,
            ray_distance,
            screen_distance_px: screen_distance,
        };

        let replace = best
            .map(|current| candidate.screen_distance_px < current.screen_distance_px)
            .unwrap_or(true);
        if replace {
            best = Some(candidate);
        }
    }

    best
}

fn point_segment_distance(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let length_sq = segment.length_squared();
    if length_sq <= f32::EPSILON {
        return point.distance(start);
    }

    let t = ((point - start).dot(segment) / length_sq).clamp(0.0, 1.0);
    let closest = start + segment * t;
    point.distance(closest)
}

fn project_world_to_screen(
    world_point: Vec3,
    camera: EditorViewportCamera,
    viewport_bounds: UiRect,
    fov_y: f32,
) -> Option<Vec2> {
    if viewport_bounds.width <= f32::EPSILON || viewport_bounds.height <= f32::EPSILON {
        return None;
    }

    let relative = world_point - camera.position;
    let x_cam = relative.dot(camera.right);
    let y_cam = relative.dot(camera.up);
    let z_cam = relative.dot(camera.forward);
    if z_cam <= 0.001 {
        return None;
    }

    let tan_half_fov = (fov_y * 0.5).tan().max(0.0001);
    let aspect = (viewport_bounds.width / viewport_bounds.height.max(1.0)).max(0.01);
    let ndc_x = x_cam / (z_cam * tan_half_fov * aspect);
    let ndc_y = y_cam / (z_cam * tan_half_fov);
    if ndc_x.abs() > 1.5 || ndc_y.abs() > 1.5 {
        return None;
    }

    Some(vec2(
        viewport_bounds.x + (ndc_x * 0.5 + 0.5) * viewport_bounds.width,
        viewport_bounds.y + (0.5 - ndc_y * 0.5) * viewport_bounds.height,
    ))
}

fn viewport_ray(cursor: UiPoint, viewport_bounds: UiRect) -> Option<PickingRay> {
    let width = viewport_bounds.width;
    let height = viewport_bounds.height;
    if width <= f32::EPSILON || height <= f32::EPSILON {
        return None;
    }

    let local_x = ((cursor.x - viewport_bounds.x) / width).clamp(0.0, 1.0);
    let local_y = ((cursor.y - viewport_bounds.y) / height).clamp(0.0, 1.0);
    let ndc = vec2(local_x * 2.0 - 1.0, 1.0 - local_y * 2.0);

    let camera = editor_viewport_camera();
    let tan_half_fov = (editor_viewport_camera_fov_y_radians() * 0.5)
        .tan()
        .max(0.0001);
    let aspect = width / height;
    let direction = (camera.forward
        + camera.right * ndc.x * aspect * tan_half_fov
        + camera.up * ndc.y * tan_half_fov)
        .normalize_or_zero();
    if direction.length_squared() <= f32::EPSILON {
        return None;
    }

    Some(PickingRay {
        origin: camera.position,
        direction,
    })
}

fn ray_aabb_first_hit(origin: Vec3, direction: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let mut t_min = 0.0_f32;
    let mut t_max = f32::INFINITY;

    for axis in 0..3 {
        let o = origin[axis];
        let d = direction[axis];
        let min_value = min[axis];
        let max_value = max[axis];

        if d.abs() <= f32::EPSILON {
            if o < min_value || o > max_value {
                return None;
            }
            continue;
        }

        let inv_d = 1.0 / d;
        let mut t0 = (min_value - o) * inv_d;
        let mut t1 = (max_value - o) * inv_d;
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        t_min = t_min.max(t0);
        t_max = t_max.min(t1);
        if t_max < t_min {
            return None;
        }
    }

    Some(t_min.max(0.0))
}

fn scaled_half_extents(half_extents: Vec3Value, scale: Vec3Value) -> Vec3 {
    let safe_scale = vec3(scale.x.abs(), scale.y.abs(), scale.z.abs());
    vec3(
        half_extents.x.max(0.05) * safe_scale.x.max(0.0001),
        half_extents.y.max(0.05) * safe_scale.y.max(0.0001),
        half_extents.z.max(0.05) * safe_scale.z.max(0.0001),
    )
}

fn entity_transform(runtime: &RunenwerkEditorRuntime, entity: EntityId) -> Option<LocalTransform> {
    let ecs_entity = runtime.ids().resolve_entity(entity)?;
    runtime.world().get::<LocalTransform>(ecs_entity).copied()
}

fn picking_target_label(target: EditorPickingTarget) -> String {
    match target {
        EditorPickingTarget::None => "none".to_string(),
        EditorPickingTarget::Grid => "grid".to_string(),
        EditorPickingTarget::Entity(entity) => format!("entity:{entity}"),
        EditorPickingTarget::ComponentHandle {
            entity,
            component_type,
        } => format!("component:{entity}:{component_type}"),
        EditorPickingTarget::GizmoAxis(axis) => format!("gizmo:{}", axis.as_str()),
    }
}

fn hit_changed(previous: EditorPickingHit, next: EditorPickingHit) -> bool {
    previous.target != next.target
        || (previous.distance - next.distance).abs() > HIT_DISTANCE_EPSILON
}

fn routed_viewport_bounds(
    host: &EditorHostResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    cursor: UiPoint,
) -> Option<(editor_viewport::ViewportId, UiRect)> {
    if let Some(binding) = resolve_binding_for_widget(
        &host.shell_state,
        tool_surface_bindings,
        editor_shell::VIEWPORT_SURFACE_EMBED_WIDGET_ID,
    )
    .filter(|binding| binding.bounds.contains(cursor))
    {
        return Some((binding.viewport_id, binding.bounds));
    }

    let runtime_state = host.shell_state.runtime().state();
    if let Some(binding) = runtime_state.captured_widget.and_then(|widget| {
        resolve_binding_for_widget(&host.shell_state, tool_surface_bindings, widget)
    }) {
        return Some((binding.viewport_id, binding.bounds));
    }

    if let Some(binding) = runtime_state.hovered_widget.and_then(|widget| {
        resolve_binding_for_widget(&host.shell_state, tool_surface_bindings, widget)
    }) {
        return Some((binding.viewport_id, binding.bounds));
    }

    // Fallback for transient hover/capture dropouts: route by current cursor containment.
    let cursor_binding = tool_surface_bindings
        .bindings()
        .filter(|binding| binding.bounds.contains(cursor))
        .min_by(|left, right| {
            let left_area = left.bounds.width * left.bounds.height;
            let right_area = right.bounds.width * right.bounds.height;
            left_area.total_cmp(&right_area)
        })?;
    Some((cursor_binding.viewport_id, cursor_binding.bounds))
}

fn resolve_binding_for_widget(
    shell_state: &crate::shell::RunenwerkEditorShellState,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    widget_id: editor_shell::WidgetId,
) -> Option<crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord> {
    structural_context_for_widget(shell_state, widget_id)
        .and_then(|context| tool_surface_bindings.resolve_structural_context(context))
}

fn structural_context_for_widget(
    shell_state: &crate::shell::RunenwerkEditorShellState,
    widget_id: editor_shell::WidgetId,
) -> Option<editor_shell::StructuralWidgetRoutingContext> {
    shell_state
        .last_projection_artifacts()
        .and_then(|artifacts| artifacts.widget_structural_context_by_id.get(&widget_id))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_app::RunenwerkEditorApp;
    use crate::editor_runtime::{bootstrap_mvp_scene_if_empty, register_mvp_component_types};
    use crate::runtime::viewport::{ViewportLayoutEntry, ViewportLayoutMapResource};
    use editor_viewport::ViewportId;
    use ui_theme::ThemeTokens;

    fn seeded_runtime() -> RunenwerkEditorRuntime {
        let mut app = RunenwerkEditorApp::new();
        register_mvp_component_types(app.runtime_mut());
        bootstrap_mvp_scene_if_empty(app.runtime_mut()).expect("mvp bootstrap should succeed");
        app.runtime
    }

    fn seeded_host_with_projection() -> EditorHostResource {
        let mut host = EditorHostResource::default();
        let view_model = crate::shell::build_editor_shell_view_model(&host.app);
        let build = editor_shell::build_editor_shell(
            &view_model,
            &ThemeTokens::default(),
            host.shell_state.workspace_state(),
        );
        host.shell_state
            .set_last_projection_artifacts(build.projection_artifacts);
        host
    }

    fn bind_viewport_surface(
        host: &EditorHostResource,
        viewport_id: ViewportId,
        bounds: UiRect,
    ) -> ToolSurfaceRuntimeBindingRegistryResource {
        let structural_context = host
            .shell_state
            .last_projection_artifacts()
            .and_then(|artifacts| {
                artifacts
                    .widget_structural_context_by_id
                    .get(&editor_shell::VIEWPORT_SURFACE_EMBED_WIDGET_ID)
                    .copied()
            })
            .expect("viewport embed structural context should exist");
        let mut layout_map = ViewportLayoutMapResource::default();
        layout_map.upsert_entry(ViewportLayoutEntry {
            viewport_id,
            host_widget_id: editor_shell::VIEWPORT_SURFACE_EMBED_WIDGET_ID,
            structural_context,
            bounds,
        });
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.rebuild_from_layout_map(&layout_map);
        bindings
    }

    #[test]
    fn compose_hit_returns_entity_for_primitive_intersection() {
        let runtime = seeded_runtime();
        let entity = runtime
            .document()
            .entity_ids()
            .next()
            .expect("seeded runtime should contain one entity");
        let transform = entity_transform(&runtime, entity).expect("entity should have transform");
        let camera = editor_viewport_camera();
        let direction = (transform.translation.to_glam() - camera.position).normalize_or_zero();

        let hit = compose_picking_hit(
            &runtime,
            None,
            None,
            UiPoint::new(640.0, 360.0),
            UiRect::new(0.0, 0.0, 1280.0, 720.0),
            PickingRay {
                origin: camera.position,
                direction,
            },
        );

        assert_eq!(hit.target, EditorPickingTarget::Entity(entity.0));
        assert!(hit.distance >= 0.0);
    }

    #[test]
    fn compose_hit_returns_grid_when_no_entity_intersection() {
        let mut runtime = seeded_runtime();
        let entity = runtime
            .document()
            .entity_ids()
            .next()
            .expect("seeded runtime should contain one entity");
        runtime
            .remove_component_for_editor_entity::<EditorPrimitive>(entity)
            .expect("primitive should be removable");

        let camera = editor_viewport_camera();
        let hit = compose_picking_hit(
            &runtime,
            None,
            None,
            UiPoint::new(640.0, 360.0),
            UiRect::new(0.0, 0.0, 1280.0, 720.0),
            PickingRay {
                origin: camera.position,
                direction: camera.forward,
            },
        );

        assert_eq!(hit.target, EditorPickingTarget::Grid);
        assert!(hit.distance >= 0.0);
    }

    #[test]
    fn compose_hit_returns_none_for_parallel_ray_without_entity() {
        let mut runtime = seeded_runtime();
        let entity = runtime
            .document()
            .entity_ids()
            .next()
            .expect("seeded runtime should contain one entity");
        runtime
            .remove_component_for_editor_entity::<EditorPrimitive>(entity)
            .expect("primitive should be removable");

        let hit = compose_picking_hit(
            &runtime,
            None,
            None,
            UiPoint::new(0.0, 0.0),
            UiRect::new(0.0, 0.0, 1280.0, 720.0),
            PickingRay {
                origin: vec3(0.0, 2.0, 0.0),
                direction: vec3(1.0, 0.0, 0.0),
            },
        );

        assert_eq!(hit.target, EditorPickingTarget::None);
        assert!(hit.distance.is_infinite());
    }

    #[test]
    fn routed_viewport_prefers_canonical_viewport_embed_binding() {
        let mut host = seeded_host_with_projection();
        host.shell_state.runtime_mut().state_mut().hovered_widget = None;
        host.shell_state.runtime_mut().state_mut().captured_widget = None;
        let expected_bounds = UiRect::new(90.0, 60.0, 900.0, 520.0);
        let bindings = bind_viewport_surface(&host, ViewportId(7), expected_bounds);

        let routed = routed_viewport_bounds(&host, &bindings, UiPoint::new(120.0, 240.0));

        assert_eq!(routed, Some((ViewportId(7), expected_bounds)));
    }

    #[test]
    fn routed_viewport_falls_back_to_cursor_containment_when_projection_is_missing() {
        let host = EditorHostResource::default();
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.upsert_binding(crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord {
            tool_surface_id: editor_shell::ToolSurfaceInstanceId::new(33),
            panel_instance_id: editor_shell::PanelInstanceId::new(11),
            tab_stack_id: editor_shell::TabStackId::new(22),
            viewport_id: ViewportId(9),
            host_widget_id: editor_shell::WidgetId(77),
            bounds: UiRect::new(100.0, 200.0, 300.0, 250.0),
            generation: 1,
        });

        let routed = routed_viewport_bounds(&host, &bindings, UiPoint::new(250.0, 320.0));

        assert_eq!(
            routed,
            Some((ViewportId(9), UiRect::new(100.0, 200.0, 300.0, 250.0))),
        );
    }

    #[test]
    fn routed_viewport_returns_none_when_cursor_is_outside_viewport_bounds() {
        let mut host = seeded_host_with_projection();
        host.shell_state.runtime_mut().state_mut().hovered_widget = None;
        host.shell_state.runtime_mut().state_mut().captured_widget = None;
        let expected_bounds = UiRect::new(90.0, 60.0, 900.0, 520.0);
        let bindings = bind_viewport_surface(&host, ViewportId(7), expected_bounds);

        let routed = routed_viewport_bounds(&host, &bindings, UiPoint::new(8.0, 8.0));

        assert_eq!(routed, None);
    }
}
