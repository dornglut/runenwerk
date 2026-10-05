use super::*;

pub(super) fn sync_active_editor_shortcut_bindings(
    input: &engine::plugins::InputState,
    actions: &mut engine::plugins::ActionState,
    host: &EditorHostResource,
    bridge: &mut EditorInputBridgeState,
) {
    let catalog_active = !host
        .shell_state
        .active_editor_definitions()
        .shortcuts()
        .is_empty();
    let resolved =
        match resolve_active_editor_shortcuts(host.shell_state.active_editor_definitions()) {
            Ok(value) => value,
            Err(_) => return,
        };
    let signature = resolved
        .iter()
        .map(|shortcut| {
            (
                shortcut.action_id.clone(),
                shortcut.command_key.clone(),
                shortcut.chord_text.clone(),
            )
        })
        .collect::<Vec<_>>();
    if bridge.active_shortcut_catalog_active == catalog_active
        && bridge.active_shortcut_signature == signature
    {
        return;
    }

    for action_id in bridge.active_shortcut_action_ids.drain(..) {
        actions.clear_action_bindings(input, &action_id);
    }
    bridge.active_shortcut_commands.clear();
    for shortcut in resolved {
        actions.map_chord(input, shortcut.action_id.clone(), shortcut.chord);
        bridge
            .active_shortcut_commands
            .insert(shortcut.action_id.clone(), shortcut.command);
        bridge.active_shortcut_action_ids.push(shortcut.action_id);
    }
    bridge.active_shortcut_catalog_active = catalog_active;
    bridge.active_shortcut_signature = signature;
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch_global_shortcuts(
    actions: &engine::plugins::ActionState,
    host: &mut EditorHostResource,
    bridge: &EditorInputBridgeState,
    viewport_presentations: &mut ViewportPresentationStateResource,
    viewport_observations: &ViewportArtifactObservationResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    cursor: UiPoint,
    preferred_viewport_id: Option<ViewportId>,
    viewport_shortcuts_blocked: bool,
) {
    dispatch_active_editor_shortcuts(
        actions,
        host,
        bridge,
        viewport_presentations,
        viewport_observations,
        tool_surface_bindings,
        cursor,
        preferred_viewport_id,
        viewport_shortcuts_blocked,
    );
    if bridge.active_shortcut_catalog_active {
        return;
    }

    if actions.action_pressed(ACTION_EDITOR_UNDO)
        && let Err(error) = dispatch_shell_command(
            &mut host.app,
            Some(&mut host.shell_state),
            ShellCommand::Undo,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
            Some(tool_surface_bindings),
            None,
        )
    {
        eprintln!("undo shortcut failed: {error}");
    }

    if actions.action_pressed(ACTION_EDITOR_REDO)
        && let Err(error) = dispatch_shell_command(
            &mut host.app,
            Some(&mut host.shell_state),
            ShellCommand::Redo,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
            Some(tool_surface_bindings),
            None,
        )
    {
        eprintln!("redo shortcut failed: {error}");
    }

    if actions.action_pressed(action::UI_SAVE_TEMPLATE)
        && let Err(error) = dispatch_shell_command(
            &mut host.app,
            Some(&mut host.shell_state),
            ShellCommand::SaveScene,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
            Some(tool_surface_bindings),
            None,
        )
    {
        eprintln!("save shortcut failed: {error}");
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch_active_editor_shortcuts(
    actions: &engine::plugins::ActionState,
    host: &mut EditorHostResource,
    bridge: &EditorInputBridgeState,
    viewport_presentations: &mut ViewportPresentationStateResource,
    viewport_observations: &ViewportArtifactObservationResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    cursor: UiPoint,
    preferred_viewport_id: Option<ViewportId>,
    viewport_shortcuts_blocked: bool,
) {
    let pressed = bridge
        .active_shortcut_commands
        .iter()
        .filter_map(|(action_id, command)| actions.action_pressed(action_id).then_some(*command))
        .collect::<Vec<_>>();
    for command in pressed {
        if let Err(error) = dispatch_known_editor_command(
            command,
            host,
            viewport_presentations,
            viewport_observations,
            tool_surface_bindings,
            cursor,
            preferred_viewport_id,
            viewport_shortcuts_blocked,
        ) {
            eprintln!("active editor shortcut failed: {error}");
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch_known_editor_command(
    command: KnownEditorCommand,
    host: &mut EditorHostResource,
    viewport_presentations: &mut ViewportPresentationStateResource,
    viewport_observations: &ViewportArtifactObservationResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    cursor: UiPoint,
    preferred_viewport_id: Option<ViewportId>,
    viewport_shortcuts_blocked: bool,
) -> Result<(), editor_core::EditorMutationError> {
    if let Some(tool) = command.viewport_tool() {
        if viewport_shortcuts_blocked {
            return Ok(());
        }
        return dispatch_viewport_tool_activation(
            tool,
            host,
            viewport_observations,
            tool_surface_bindings,
            cursor,
            preferred_viewport_id,
        );
    }
    let Some(shell_command) = command.to_shell_command() else {
        return Ok(());
    };
    dispatch_shell_command(
        &mut host.app,
        Some(&mut host.shell_state),
        shell_command,
        Some(&mut *viewport_presentations),
        Some(viewport_observations),
        Some(tool_surface_bindings),
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch_viewport_tool_activation(
    tool: editor_shell::ViewportToolKind,
    host: &mut EditorHostResource,
    viewport_observations: &ViewportArtifactObservationResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    cursor: UiPoint,
    preferred_viewport_id: Option<ViewportId>,
) -> Result<(), editor_core::EditorMutationError> {
    let Some(binding) = fallback_viewport_binding(
        tool_surface_bindings,
        host.shell_state.primary_composition_target_id(),
        cursor,
    )
    .or_else(|| {
        preferred_viewport_id.and_then(|viewport_id| {
            viewport_binding_by_id_for_target(
                tool_surface_bindings,
                host.shell_state.primary_composition_target_id(),
                viewport_id,
            )
        })
    }) else {
        return Ok(());
    };
    let Some(mounted_unit_id) = host
        .shell_state
        .mounted_unit_id_for_tool_surface(binding.tool_surface_id)
    else {
        return Ok(());
    };
    let Some(target) = host
        .shell_state
        .structural_command_target_for_mounted_unit(mounted_unit_id)
    else {
        return Ok(());
    };
    if target.active_tool_surface != Some(binding.tool_surface_id)
        || target.panel_instance_id != binding.panel_instance_id
        || target.tab_stack_id != binding.tab_stack_id
    {
        return Ok(());
    }
    let projection_epoch = host.shell_state.current_projection_epoch();
    dispatch_shell_command(
        &mut host.app,
        Some(&mut host.shell_state),
        ShellCommand::ApplySurfaceSessionMutation {
            target,
            mutation: editor_shell::SurfaceSessionMutation::Viewport(
                editor_shell::ViewportSessionMutation::ActivateTool { tool },
            ),
            projection_epoch,
        },
        None,
        Some(viewport_observations),
        Some(tool_surface_bindings),
        Some(projection_epoch),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch_viewport_shortcuts(
    actions: &engine::plugins::ActionState,
    host: &mut EditorHostResource,
    bridge: &EditorInputBridgeState,
    viewport_observations: &ViewportArtifactObservationResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    viewport_render_commands: &mut ViewportRenderStateCommandQueueResource,
    cursor: UiPoint,
    preferred_viewport_id: Option<ViewportId>,
) {
    if !bridge.active_shortcut_catalog_active
        && (actions.action_pressed(ACTION_EDITOR_TOOL_SELECT)
            || actions.action_pressed(action::UI_EDITOR_RESTORE_ALL))
        && let Err(error) = dispatch_viewport_tool_activation(
            editor_shell::ViewportToolKind::Select,
            host,
            viewport_observations,
            tool_surface_bindings,
            cursor,
            preferred_viewport_id,
        )
    {
        eprintln!("select-tool shortcut failed: {error}");
    }

    if !bridge.active_shortcut_catalog_active
        && (actions.action_pressed(ACTION_EDITOR_TOOL_TRANSLATE)
            || actions.action_pressed(action::UI_EDITOR_HIDE_SELECTED))
        && let Err(error) = dispatch_viewport_tool_activation(
            editor_shell::ViewportToolKind::Translate,
            host,
            viewport_observations,
            tool_surface_bindings,
            cursor,
            preferred_viewport_id,
        )
    {
        eprintln!("translate-tool shortcut failed: {error}");
    }

    if !bridge.active_shortcut_catalog_active
        && actions.action_pressed(ACTION_EDITOR_TOOL_ROTATE)
        && let Err(error) = dispatch_viewport_tool_activation(
            editor_shell::ViewportToolKind::Rotate,
            host,
            viewport_observations,
            tool_surface_bindings,
            cursor,
            preferred_viewport_id,
        )
    {
        eprintln!("rotate-tool shortcut failed: {error}");
    }

    if !bridge.active_shortcut_catalog_active
        && actions.action_pressed(ACTION_EDITOR_TOOL_SCALE)
        && let Err(error) = dispatch_viewport_tool_activation(
            editor_shell::ViewportToolKind::Scale,
            host,
            viewport_observations,
            tool_surface_bindings,
            cursor,
            preferred_viewport_id,
        )
    {
        eprintln!("scale-tool shortcut failed: {error}");
    }

    if actions.action_pressed(ACTION_EDITOR_VIEWPORT_FOCUS)
        && let Some(orbit_target) = selected_entity_origin(&host.app)
        && let Some(binding) = viewport_binding_for_focus(
            tool_surface_bindings,
            host.shell_state.primary_composition_target_id(),
            cursor,
            preferred_viewport_id,
        )
    {
        viewport_render_commands.push(ViewportRenderStateCommand::FocusCameraOn {
            viewport_id: binding.viewport_id,
            orbit_target,
        });
    }
}

pub(super) fn selected_entity_origin(
    app: &crate::editor_app::RunenwerkEditorApp,
) -> Option<[f32; 3]> {
    let selected = app.runtime().selected_entity()?;
    let ecs_entity = app.runtime().ids().resolve_entity(selected)?;
    let transform = app
        .runtime()
        .world()
        .get::<LocalTransform>(ecs_entity)
        .copied()?;
    Some([
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
    ])
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_viewport_tool_radial_shortcut(
    actions: &engine::plugins::ActionState,
    host: &mut EditorHostResource,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    cursor: UiPoint,
    viewport_shortcuts_blocked: bool,
) {
    if !actions.action_pressed(ACTION_EDITOR_VIEWPORT_TOOL_RADIAL) || viewport_shortcuts_blocked {
        return;
    }

    let Some(tool_surface_bindings) = tool_surface_bindings else {
        return;
    };
    let Some(binding) = fallback_viewport_binding(
        tool_surface_bindings,
        host.shell_state.primary_composition_target_id(),
        cursor,
    ) else {
        return;
    };

    let target = editor_shell::StructuralCommandTarget {
        mounted_unit_id: host
            .shell_state
            .mounted_unit_id_for_tool_surface(binding.tool_surface_id),
        panel_instance_id: binding.panel_instance_id,
        active_tool_surface: Some(binding.tool_surface_id),
        tab_stack_id: binding.tab_stack_id,
    };
    let projection_epoch = host.shell_state.current_projection_epoch();
    if let Err(error) = dispatch_shell_command(
        &mut host.app,
        Some(&mut host.shell_state),
        ShellCommand::ApplySurfaceSessionMutation {
            target,
            mutation: editor_shell::SurfaceSessionMutation::Viewport(
                editor_shell::ViewportSessionMutation::OpenToolRadialMenu {
                    viewport_id: binding.viewport_id,
                    anchor_position: cursor,
                    opened_by_tab_hold: true,
                },
            ),
            projection_epoch,
        },
        None,
        viewport_observations,
        Some(tool_surface_bindings),
        Some(projection_epoch),
    ) {
        eprintln!("viewport radial shortcut failed: {error}");
    }
}

pub(super) fn viewport_binding_for_focus(
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    presentation_target_id: ui_composition::PresentationTargetId,
    cursor: UiPoint,
    preferred_viewport_id: Option<ViewportId>,
) -> Option<crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord> {
    fallback_viewport_binding(tool_surface_bindings, presentation_target_id, cursor)
        .or_else(|| {
            preferred_viewport_id.and_then(|viewport_id| {
                viewport_binding_by_id_for_target(
                    tool_surface_bindings,
                    presentation_target_id,
                    viewport_id,
                )
            })
        })
        .or_else(|| {
            tool_surface_bindings
                .bindings()
                .find(|binding| binding.presentation_target_id == presentation_target_id)
        })
}
