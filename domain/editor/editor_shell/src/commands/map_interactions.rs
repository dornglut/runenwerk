//! File: domain/editor/editor_shell/src/commands/map_interactions.rs
//! Purpose: Map semantic UI interactions to shell commands.

use crate::{
    RoutedShellAction, ShellCommand, ShellProjectionArtifacts, StructuralCommandTarget,
    StructuralWidgetRoutingContext, UiInteraction, UiInteractionResults,
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
                if let Some(RoutedShellAction::AppendEntityTableSearchText { context }) =
                    routing.widget_actions_by_id.get(target)
                {
                    commands.push(ShellCommand::AppendEntityTableSearchText {
                        text: event.text.clone(),
                        target: command_target(*context),
                        projection_epoch: routing.projection_epoch,
                    });
                } else if let Some(RoutedShellAction::EditInspectorFieldText { index, context }) =
                    routing.widget_actions_by_id.get(target)
                {
                    commands.push(ShellCommand::AppendInspectorFieldText {
                        index: *index,
                        text: event.text.clone(),
                        target: command_target(*context),
                        projection_epoch: routing.projection_epoch,
                    });
                }
            }
            UiInteraction::KeyboardInput { target, event } => {
                if !matches!(event.state, KeyState::Pressed | KeyState::Repeated) {
                    continue;
                }
                match routing.widget_actions_by_id.get(target) {
                    Some(RoutedShellAction::AppendEntityTableSearchText { context }) => {
                        if matches!(event.key, Key::Backspace | Key::Delete) {
                            commands.push(ShellCommand::BackspaceEntityTableSearch {
                                target: command_target(*context),
                                projection_epoch: routing.projection_epoch,
                            });
                        }
                    }
                    Some(RoutedShellAction::EditInspectorFieldText { index, context }) => {
                        match event.key {
                            Key::Backspace | Key::Delete => {
                                commands.push(ShellCommand::BackspaceInspectorFieldText {
                                    index: *index,
                                    target: command_target(*context),
                                    projection_epoch: routing.projection_epoch,
                                });
                            }
                            Key::Enter => {
                                commands.push(ShellCommand::CommitInspectorFieldText {
                                    index: *index,
                                    target: command_target(*context),
                                    projection_epoch: routing.projection_epoch,
                                });
                            }
                            Key::Escape => {
                                commands.push(ShellCommand::CancelInspectorFieldText {
                                    index: *index,
                                    target: command_target(*context),
                                    projection_epoch: routing.projection_epoch,
                                });
                            }
                            _ => {}
                        }
                    }
                    _ => {}
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
        RoutedShellAction::ToggleEntityTableSort { sort_key, context } => {
            ShellCommand::ToggleEntityTableSort {
                sort_key: *sort_key,
                target: command_target(*context),
                projection_epoch: routing.projection_epoch,
            }
        }
        RoutedShellAction::SelectOutlinerEntity { entity, context } => {
            ShellCommand::SelectOutlinerEntity {
                entity: *entity,
                target: command_target(*context),
                projection_epoch: routing.projection_epoch,
            }
        }
        RoutedShellAction::SelectViewportProduct {
            viewport_id,
            product_id,
            enabled,
            context,
        } => {
            if *enabled {
                ShellCommand::SelectViewportProduct {
                    viewport_id: *viewport_id,
                    product_id: *product_id,
                    target: command_target(*context),
                    projection_epoch: routing.projection_epoch,
                }
            } else {
                ShellCommand::NoOp
            }
        }
        RoutedShellAction::ActivateInspectorField { index, context } => {
            ShellCommand::ActivateInspectorField {
                index: *index,
                target: command_target(*context),
                projection_epoch: routing.projection_epoch,
            }
        }
        RoutedShellAction::FocusInspectorField { index, context } => {
            ShellCommand::FocusInspectorField {
                index: *index,
                target: command_target(*context),
                projection_epoch: routing.projection_epoch,
            }
        }
        RoutedShellAction::EditInspectorFieldText { .. } => ShellCommand::NoOp,
        RoutedShellAction::SelectEntityTableRow { .. }
        | RoutedShellAction::AppendEntityTableSearchText { .. }
        | RoutedShellAction::BackspaceEntityTableSearch { .. } => ShellCommand::NoOp,
    }
}

fn command_for_table_row(
    widget_id: crate::WidgetId,
    row_index: usize,
    routing: &ShellProjectionArtifacts,
) -> ShellCommand {
    let Some(RoutedShellAction::SelectEntityTableRow { entities, context }) =
        routing.widget_actions_by_id.get(&widget_id)
    else {
        return ShellCommand::NoOp;
    };

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

    ShellCommand::SelectEntityTableEntity {
        entity,
        target: command_target(*context),
        projection_epoch: routing.projection_epoch,
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
        RoutedShellAction::SelectOutlinerEntity { context, .. }
        | RoutedShellAction::SelectEntityTableRow { context, .. }
        | RoutedShellAction::AppendEntityTableSearchText { context }
        | RoutedShellAction::BackspaceEntityTableSearch { context }
        | RoutedShellAction::ToggleEntityTableSort { context, .. }
        | RoutedShellAction::SelectViewportProduct { context, .. }
        | RoutedShellAction::ActivateInspectorField { context, .. }
        | RoutedShellAction::FocusInspectorField { context, .. }
        | RoutedShellAction::EditInspectorFieldText { context, .. } => Some(*context),
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
