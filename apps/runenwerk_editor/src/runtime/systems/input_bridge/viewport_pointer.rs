use super::*;

#[derive(Debug, Clone, Copy)]
pub(super) struct ViewportPointerRoute {
    pub(super) tool_surface_id: editor_shell::ToolSurfaceInstanceId,
    pub(super) viewport_id: ViewportId,
    pub(super) host_widget_id: editor_shell::WidgetId,
    pub(super) structural_context: editor_shell::StructuralWidgetRoutingContext,
    pub(super) local_position: UiPoint,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_editor_viewport_input_for_target(
    target_id: ui_composition::PresentationTargetId,
    event: &UiInputEvent,
    dispatch: Option<&editor_shell::UiInputDispatchResult>,
    ui_consumed: bool,
    host: &mut EditorHostResource,
    bridge: &mut EditorInputBridgeState,
    picking_results: &mut ViewportPickingResultsResource,
    viewport_render_states: &ViewportRenderStateResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    viewport_render_commands: &mut ViewportRenderStateCommandQueueResource,
) {
    let UiInputEvent::Pointer(pointer) = event else {
        return;
    };
    if pointer.packet.source_kind != PointerSourceKind::Mouse {
        return;
    }

    let position = pointer.position;
    if let Some(binding) =
        tool_surface_bindings.binding_containing_cursor_for_target(target_id, position)
    {
        bridge
            .interaction_for_target_mut(target_id)
            .last_target_viewport = Some(binding.viewport_id);
    }

    match (pointer.kind, pointer.button) {
        (PointerEventKind::Move, _) => {
            let interaction = bridge.interaction_for_target(target_id);
            let captured_scene_interaction = matches!(
                interaction.pointer_owner,
                EditorPointerOwner::ViewportTool { .. } | EditorPointerOwner::ViewportCamera { .. }
            );
            let pointer_routes_to_scene = viewport_pointer_route(
                &host.shell_state,
                tool_surface_bindings,
                target_id,
                dispatch,
                position,
            )
            .is_some();
            if captured_scene_interaction || pointer_routes_to_scene || !ui_consumed {
                update_editor_picking_for_target(
                    target_id,
                    position,
                    host,
                    picking_results,
                    tool_surface_bindings,
                    viewport_render_states,
                );
            } else {
                clear_editor_picking_for_target(
                    target_id,
                    position,
                    picking_results,
                    tool_surface_bindings,
                );
            }
            match interaction.pointer_owner {
                EditorPointerOwner::ViewportTool { tool_surface_id } => {
                    if pointer.delta.x != 0.0
                        && viewport_capture_active_for_surface(
                            &host.shell_state,
                            tool_surface_bindings,
                            target_id,
                            tool_surface_id,
                        )
                        && let Some(mounted_unit_id) = host
                            .shell_state
                            .mounted_unit_id_for_tool_surface(tool_surface_id)
                        && let Err(error) = host.app.dispatch_viewport_interaction_for_mounted_unit(
                            mounted_unit_id,
                            ViewportInteractionCommand::PointerDragAxis {
                                amount: pointer.delta.x,
                            },
                        )
                    {
                        eprintln!("viewport axis drag failed: {error}");
                    }
                }
                EditorPointerOwner::ViewportCamera {
                    viewport_id,
                    button,
                } if pointer.delta != ui_math::UiVector::ZERO => {
                    if let Some(binding) = active_camera_viewport_binding(
                        tool_surface_bindings,
                        target_id,
                        Some(viewport_id),
                    ) {
                        let command = match button {
                            EditorCameraPointerButton::Middle => {
                                ViewportRenderStateCommand::PanCamera {
                                    viewport_id: binding.viewport_id,
                                    delta: pointer.delta,
                                }
                            }
                            EditorCameraPointerButton::Secondary => {
                                ViewportRenderStateCommand::OrbitCamera {
                                    viewport_id: binding.viewport_id,
                                    delta: pointer.delta,
                                }
                            }
                        };
                        viewport_render_commands.push(command);
                    }
                }
                _ => {}
            }
        }
        (PointerEventKind::Scroll, _) if !ui_consumed => {
            let scroll_delta = pointer.delta.y;
            if scroll_delta.abs() > f32::EPSILON
                && let Some(binding) =
                    fallback_viewport_binding(tool_surface_bindings, target_id, position)
            {
                bridge
                    .interaction_for_target_mut(target_id)
                    .last_target_viewport = Some(binding.viewport_id);
                viewport_render_commands.push(ViewportRenderStateCommand::ZoomCamera {
                    viewport_id: binding.viewport_id,
                    scroll_delta,
                });
            }
        }
        (PointerEventKind::Down, Some(PointerButton::Primary)) => {
            if let Some(route) = viewport_pointer_route(
                &host.shell_state,
                tool_surface_bindings,
                target_id,
                dispatch,
                position,
            ) {
                update_editor_picking_for_target(
                    target_id,
                    position,
                    host,
                    picking_results,
                    tool_surface_bindings,
                    viewport_render_states,
                );
                bridge.interaction_for_target_mut(target_id).pointer_owner =
                    EditorPointerOwner::ViewportTool {
                        tool_surface_id: route.tool_surface_id,
                    };
                dispatch_viewport_pointer_down(host, picking_results, position, route);
            } else {
                bridge.interaction_for_target_mut(target_id).pointer_owner =
                    EditorPointerOwner::None;
                if host.app.debug_logs_enabled() {
                    host.app.append_console_input(format!(
                        "[input] pointer-down routed to shell only: target={} cursor=({:.1},{:.1})",
                        target_id.raw(),
                        position.x,
                        position.y
                    ));
                }
            }
        }
        (PointerEventKind::Down, Some(PointerButton::Middle)) => {
            if let Some(route) = viewport_pointer_route(
                &host.shell_state,
                tool_surface_bindings,
                target_id,
                dispatch,
                position,
            ) {
                let interaction = bridge.interaction_for_target_mut(target_id);
                interaction.active_camera_viewport = Some(route.viewport_id);
                interaction.last_target_viewport = Some(route.viewport_id);
                interaction.pointer_owner = EditorPointerOwner::ViewportCamera {
                    viewport_id: route.viewport_id,
                    button: EditorCameraPointerButton::Middle,
                };
            } else if dispatch.and_then(|value| value.target).is_some() {
                let interaction = bridge.interaction_for_target_mut(target_id);
                interaction.active_camera_viewport = None;
                interaction.pointer_owner = EditorPointerOwner::UiMiddleScroll;
            } else {
                let interaction = bridge.interaction_for_target_mut(target_id);
                interaction.active_camera_viewport = None;
                interaction.pointer_owner = EditorPointerOwner::None;
            }
        }
        (PointerEventKind::Down, Some(PointerButton::Secondary)) => {
            if let Some(route) = viewport_pointer_route(
                &host.shell_state,
                tool_surface_bindings,
                target_id,
                dispatch,
                position,
            ) {
                let interaction = bridge.interaction_for_target_mut(target_id);
                interaction.active_camera_viewport = Some(route.viewport_id);
                interaction.last_target_viewport = Some(route.viewport_id);
                interaction.pointer_owner = EditorPointerOwner::ViewportCamera {
                    viewport_id: route.viewport_id,
                    button: EditorCameraPointerButton::Secondary,
                };
            } else {
                let interaction = bridge.interaction_for_target_mut(target_id);
                interaction.active_camera_viewport = None;
                interaction.pointer_owner = EditorPointerOwner::None;
            }
        }
        (PointerEventKind::Up, Some(PointerButton::Primary)) => {
            let interaction = bridge.interaction_for_target(target_id);
            if let EditorPointerOwner::ViewportTool { tool_surface_id } = interaction.pointer_owner
                && let Some(mounted_unit_id) = host
                    .shell_state
                    .mounted_unit_id_for_tool_surface(tool_surface_id)
                && let Err(error) = host.app.dispatch_viewport_interaction_for_mounted_unit(
                    mounted_unit_id,
                    ViewportInteractionCommand::PointerUp,
                )
            {
                eprintln!("viewport pointer-up failed: {error}");
            }
            bridge.interaction_for_target_mut(target_id).pointer_owner = EditorPointerOwner::None;
        }
        (PointerEventKind::Up, Some(PointerButton::Middle | PointerButton::Secondary)) => {
            let interaction = bridge.interaction_for_target_mut(target_id);
            interaction.active_camera_viewport = None;
            interaction.pointer_owner = EditorPointerOwner::None;
        }
        _ => {}
    }

    bridge
        .interaction_for_target_mut(target_id)
        .last_pointer_position = (position.x, position.y);
}

pub(crate) fn clear_editor_viewport_interaction_for_target(
    target_id: ui_composition::PresentationTargetId,
    bridge: &mut EditorInputBridgeState,
    picking_results: &mut ViewportPickingResultsResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
) {
    let last_pointer = bridge
        .interaction_for_target(target_id)
        .last_pointer_position;
    clear_editor_picking_for_target(
        target_id,
        UiPoint::new(last_pointer.0, last_pointer.1),
        picking_results,
        tool_surface_bindings,
    );
    bridge.clear_interaction_for_target(target_id);
}

pub(super) fn viewport_pointer_route(
    shell_state: &RunenwerkEditorShellState,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    target_id: ui_composition::PresentationTargetId,
    dispatch: Option<&editor_shell::UiInputDispatchResult>,
    position: UiPoint,
) -> Option<ViewportPointerRoute> {
    if let Some(host_widget_id) = dispatch.and_then(|value| value.target) {
        let binding = viewport_scene_binding_for_widget(
            shell_state,
            tool_surface_bindings,
            target_id,
            host_widget_id,
        )?;
        if !binding.bounds.contains(position) {
            return None;
        }

        let structural_context =
            structural_context_for_widget(shell_state, target_id, host_widget_id)?;
        return Some(ViewportPointerRoute {
            tool_surface_id: binding.tool_surface_id,
            viewport_id: binding.viewport_id,
            host_widget_id,
            structural_context,
            local_position: UiPoint::new(
                position.x - binding.bounds.x,
                position.y - binding.bounds.y,
            ),
        });
    }

    let binding = fallback_viewport_binding(tool_surface_bindings, target_id, position)?;
    let host_widget_id = binding.host_widget_id;
    let structural_context = structural_context_for_widget(shell_state, target_id, host_widget_id)
        .unwrap_or(editor_shell::StructuralWidgetRoutingContext {
            mounted_unit_id: shell_state.mounted_unit_id_for_tool_surface(binding.tool_surface_id),
            panel_instance_id: binding.panel_instance_id,
            active_tool_surface: Some(binding.tool_surface_id),
            tab_stack_id: binding.tab_stack_id,
        });
    Some(ViewportPointerRoute {
        tool_surface_id: binding.tool_surface_id,
        viewport_id: binding.viewport_id,
        host_widget_id,
        structural_context,
        local_position: UiPoint::new(position.x - binding.bounds.x, position.y - binding.bounds.y),
    })
}

pub(super) fn viewport_capture_active_for_surface(
    shell_state: &RunenwerkEditorShellState,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    target_id: ui_composition::PresentationTargetId,
    tool_surface_id: editor_shell::ToolSurfaceInstanceId,
) -> bool {
    if let Some(captured_widget) = shell_state
        .runtime_for_target(target_id)
        .and_then(|runtime| runtime.state().captured_widget)
    {
        return viewport_scene_binding_for_widget(
            shell_state,
            tool_surface_bindings,
            target_id,
            captured_widget,
        )
        .map(|binding| binding.tool_surface_id == tool_surface_id)
        .unwrap_or(false);
    }

    tool_surface_bindings
        .binding_for_tool_surface(tool_surface_id)
        .is_some_and(|binding| binding.presentation_target_id == target_id)
}

pub(super) fn active_camera_viewport_binding(
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    target_id: ui_composition::PresentationTargetId,
    active_viewport_id: Option<ViewportId>,
) -> Option<crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord> {
    active_viewport_id.and_then(|viewport_id| {
        viewport_binding_by_id_for_target(tool_surface_bindings, target_id, viewport_id)
    })
}

pub(super) fn dispatch_viewport_pointer_down(
    host: &mut EditorHostResource,
    picking_results: &ViewportPickingResultsResource,
    position: UiPoint,
    route: ViewportPointerRoute,
) {
    let expression = build_viewport_picking_product_frame(
        route.viewport_id,
        picking_results,
        host.app.runtime().current_scene_reality_version(),
    );
    let hit = viewport_hit_from_picking_product(&expression);
    let picking = picking_results.result_for(route.viewport_id);
    let selection_before = host.app.runtime().selected_entity();

    if host.app.debug_logs_enabled() {
        host.app.append_console_input(format!(
            "[input] viewport pointer-down viewport={} tool_surface={} widget={} panel={} tab_stack={} structural_tool_surface={:?} cursor=({:.1},{:.1}) local=({:.1},{:.1}) hit={} dist={:.3} expr_frame={} sel_before={:?}",
            route.viewport_id.0,
            route.tool_surface_id.raw(),
            route.host_widget_id.0,
            route.structural_context.panel_instance_id.raw(),
            route.structural_context.tab_stack_id.raw(),
            route.structural_context.active_tool_surface.map(|value| value.raw()),
            position.x,
            position.y,
            route.local_position.x,
            route.local_position.y,
            picking
                .map(|value| picking_target_label(value.hit.target))
                .unwrap_or_else(|| "none".to_string()),
            picking.map(|value| value.hit.distance).unwrap_or(f32::INFINITY),
            expression.expression.metadata.frame_id.0,
            selection_before
        ));
    }

    let Some(mounted_unit_id) = route.structural_context.mounted_unit_id else {
        return;
    };
    let result = host.app.dispatch_viewport_interaction_for_mounted_unit(
        mounted_unit_id,
        ViewportInteractionCommand::PointerDown { hit },
    );
    if let Err(error) = result {
        eprintln!("viewport pointer-down failed: {error}");
        return;
    }

    if host.app.debug_logs_enabled() {
        host.app.append_console_input(format!(
            "[input] viewport command=PointerDown sel_after={:?}",
            host.app.runtime().selected_entity()
        ));
    }
}

pub(super) fn editor_axis_label(axis: EditorGizmoAxis) -> &'static str {
    match axis {
        EditorGizmoAxis::X => "X",
        EditorGizmoAxis::Y => "Y",
        EditorGizmoAxis::Z => "Z",
    }
}

pub(super) fn picking_target_label(target: EditorPickingTarget) -> String {
    match target {
        EditorPickingTarget::None => "none".to_string(),
        EditorPickingTarget::Grid => "grid".to_string(),
        EditorPickingTarget::Entity(entity) => format!("entity:{entity}"),
        EditorPickingTarget::ComponentHandle {
            entity,
            component_type,
        } => format!("component:{entity}:{component_type}"),
        EditorPickingTarget::GizmoAxis(axis) => format!("gizmo:{}", editor_axis_label(axis)),
    }
}
