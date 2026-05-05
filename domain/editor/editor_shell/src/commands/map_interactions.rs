//! File: domain/editor/editor_shell/src/commands/map_interactions.rs
//! Purpose: Map semantic UI interactions to shell commands.

use crate::{
    RoutedShellAction, ShellCommand, ShellProjectionArtifacts, StructuralCommandTarget,
    StructuralWidgetRoutingContext, SurfaceLocalAction, UiInteraction, UiInteractionResults,
};
use ui_input::{Key, KeyState};

pub fn map_interactions_to_shell_commands(
    interactions: &UiInteractionResults,
    routing: &ShellProjectionArtifacts,
) -> Vec<ShellCommand> {
    let mut commands = Vec::new();

    for interaction in &interactions.items {
        match interaction {
            UiInteraction::Activated(widget_id) => {
                commands.push(command_for_activation(*widget_id, routing));
            }
            UiInteraction::TableRowSelected { target, row_index } => {
                commands.push(command_for_table_row(*target, *row_index, routing));
            }
            UiInteraction::TextInput { target, event } => {
                if let Some(RoutedShellAction::DispatchSurfaceLocalAction {
                    provider_id,
                    tool_surface_instance_id,
                    action,
                    context,
                }) = routing.widget_actions_by_id.get(target)
                    && matches!(
                        action,
                        SurfaceLocalAction::AppendEntityTableSearchText { .. }
                            | SurfaceLocalAction::EditInspectorFieldText { .. }
                    )
                {
                    commands.push(ShellCommand::DispatchSurfaceLocalAction {
                        provider_id: *provider_id,
                        tool_surface_instance_id: *tool_surface_instance_id,
                        target: command_target(*context),
                        action: surface_text_action(action, event.text.clone()),
                        projection_epoch: routing.projection_epoch,
                    });
                }
            }
            UiInteraction::KeyboardInput { target, event } => {
                if !matches!(event.state, KeyState::Pressed | KeyState::Repeated) {
                    continue;
                }
                if let Some(RoutedShellAction::DispatchSurfaceLocalAction {
                    provider_id,
                    tool_surface_instance_id,
                    action,
                    context,
                }) = routing.widget_actions_by_id.get(target)
                {
                    match (&event.key, action) {
                        (
                            Key::Backspace | Key::Delete,
                            SurfaceLocalAction::AppendEntityTableSearchText { .. },
                        ) => commands.push(ShellCommand::DispatchSurfaceLocalAction {
                            provider_id: *provider_id,
                            tool_surface_instance_id: *tool_surface_instance_id,
                            target: command_target(*context),
                            action: SurfaceLocalAction::BackspaceEntityTableSearch,
                            projection_epoch: routing.projection_epoch,
                        }),
                        (
                            Key::Backspace | Key::Delete,
                            SurfaceLocalAction::EditInspectorFieldText { index, .. },
                        ) => commands.push(ShellCommand::DispatchSurfaceLocalAction {
                            provider_id: *provider_id,
                            tool_surface_instance_id: *tool_surface_instance_id,
                            target: command_target(*context),
                            action: SurfaceLocalAction::BackspaceInspectorFieldText {
                                index: *index,
                            },
                            projection_epoch: routing.projection_epoch,
                        }),
                        (Key::Enter, SurfaceLocalAction::EditInspectorFieldText { index, .. }) => {
                            commands.push(ShellCommand::DispatchSurfaceLocalAction {
                                provider_id: *provider_id,
                                tool_surface_instance_id: *tool_surface_instance_id,
                                target: command_target(*context),
                                action: SurfaceLocalAction::CommitInspectorFieldText {
                                    index: *index,
                                },
                                projection_epoch: routing.projection_epoch,
                            })
                        }
                        (Key::Escape, SurfaceLocalAction::EditInspectorFieldText { index, .. }) => {
                            commands.push(ShellCommand::DispatchSurfaceLocalAction {
                                provider_id: *provider_id,
                                tool_surface_instance_id: *tool_surface_instance_id,
                                target: command_target(*context),
                                action: SurfaceLocalAction::CancelInspectorFieldText {
                                    index: *index,
                                },
                                projection_epoch: routing.projection_epoch,
                            })
                        }
                        _ => {}
                    }
                }
            }
            UiInteraction::HoveredChanged { .. }
            | UiInteraction::PressedChanged { .. }
            | UiInteraction::FocusChanged(_)
            | UiInteraction::Toggled { .. }
            | UiInteraction::NumericStepped { .. }
            | UiInteraction::TabSelected { .. }
            | UiInteraction::SelectChanged { .. }
            | UiInteraction::TreeRowSelected { .. }
            | UiInteraction::TreeRowToggled { .. } => {}
        }
    }

    commands
}

fn command_for_activation(
    widget_id: crate::WidgetId,
    routing: &ShellProjectionArtifacts,
) -> ShellCommand {
    let Some(action) = routing.widget_actions_by_id.get(&widget_id) else {
        return ShellCommand::NoOp;
    };

    if !action_has_structural_context_match(widget_id, action, routing) {
        return ShellCommand::NoOp;
    }

    match action {
        RoutedShellAction::ActivateSelectTool => ShellCommand::ActivateSelectTool,
        RoutedShellAction::ActivateTranslateTool => ShellCommand::ActivateTranslateTool,
        RoutedShellAction::Undo { enabled } => {
            if *enabled {
                ShellCommand::Undo
            } else {
                ShellCommand::NoOp
            }
        }
        RoutedShellAction::Redo { enabled } => {
            if *enabled {
                ShellCommand::Redo
            } else {
                ShellCommand::NoOp
            }
        }
        RoutedShellAction::SaveScene { enabled } => {
            if *enabled {
                ShellCommand::SaveScene
            } else {
                ShellCommand::NoOp
            }
        }
        RoutedShellAction::LoadScene { enabled } => {
            if *enabled {
                ShellCommand::LoadScene
            } else {
                ShellCommand::NoOp
            }
        }
        RoutedShellAction::ToggleDebugLogs => ShellCommand::ToggleDebugLogs,
        RoutedShellAction::ActivateTab {
            tab_stack_id,
            panel_instance_id,
        } => ShellCommand::SetTabStackActivePanel {
            tab_stack_id: *tab_stack_id,
            panel_instance_id: *panel_instance_id,
            projection_epoch: routing.projection_epoch,
        },
        RoutedShellAction::DispatchSurfaceLocalAction {
            provider_id,
            tool_surface_instance_id,
            action,
            context,
        } => ShellCommand::DispatchSurfaceLocalAction {
            provider_id: *provider_id,
            tool_surface_instance_id: *tool_surface_instance_id,
            target: command_target(*context),
            action: action.clone(),
            projection_epoch: routing.projection_epoch,
        },
    }
}

fn surface_text_action(action: &SurfaceLocalAction, text: String) -> SurfaceLocalAction {
    match action {
        SurfaceLocalAction::AppendEntityTableSearchText { .. } => {
            SurfaceLocalAction::AppendEntityTableSearchText { text }
        }
        SurfaceLocalAction::EditInspectorFieldText { index, .. } => {
            SurfaceLocalAction::EditInspectorFieldText {
                index: *index,
                text,
            }
        }
        _ => action.clone(),
    }
}

fn command_for_table_row(
    widget_id: crate::WidgetId,
    row_index: usize,
    routing: &ShellProjectionArtifacts,
) -> ShellCommand {
    match routing.widget_actions_by_id.get(&widget_id) {
        Some(RoutedShellAction::DispatchSurfaceLocalAction {
            provider_id,
            tool_surface_instance_id,
            action: SurfaceLocalAction::SelectEntityTableRow { entities },
            context,
        }) => {
            if routing
                .widget_structural_context_by_id
                .get(&widget_id)
                .copied()
                != Some(*context)
            {
                return ShellCommand::NoOp;
            }

            let Some(entity) = entities.get(row_index).copied() else {
                return ShellCommand::NoOp;
            };

            ShellCommand::DispatchSurfaceLocalAction {
                provider_id: *provider_id,
                tool_surface_instance_id: *tool_surface_instance_id,
                target: command_target(*context),
                action: SurfaceLocalAction::SelectEntityTableEntity { entity },
                projection_epoch: routing.projection_epoch,
            }
        }
        _ => ShellCommand::NoOp,
    }
}

fn command_target(context: StructuralWidgetRoutingContext) -> StructuralCommandTarget {
    StructuralCommandTarget {
        panel_instance_id: context.panel_instance_id,
        active_tool_surface: context.active_tool_surface,
        tab_stack_id: context.tab_stack_id,
    }
}

fn action_has_structural_context_match(
    widget_id: crate::WidgetId,
    action: &RoutedShellAction,
    routing: &ShellProjectionArtifacts,
) -> bool {
    let expected = match action {
        RoutedShellAction::DispatchSurfaceLocalAction { context, .. } => Some(*context),
        _ => None,
    };

    match expected {
        Some(context) => {
            routing
                .widget_structural_context_by_id
                .get(&widget_id)
                .copied()
                == Some(context)
        }
        None => true,
    }
}
