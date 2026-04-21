//! File: domain/editor/editor_shell/src/commands/map_interactions.rs
//! Purpose: Map semantic UI interactions to shell commands.

use crate::{
    RoutedShellAction, ShellCommand, ShellProjectionArtifacts, StructuralCommandTarget,
    UiInteraction, UiInteractionResults,
};

pub fn map_interactions_to_shell_commands(
    interactions: &UiInteractionResults,
    routing: &ShellProjectionArtifacts,
) -> Vec<ShellCommand> {
    let mut commands = Vec::new();

    for interaction in &interactions.items {
        if let UiInteraction::Activated(widget_id) = interaction {
            let Some(action) = routing.widget_actions_by_id.get(widget_id) else {
                commands.push(ShellCommand::NoOp);
                continue;
            };

            if !action_has_structural_context_match(*widget_id, action, routing) {
                commands.push(ShellCommand::NoOp);
                continue;
            }

            let command = match action {
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
                RoutedShellAction::SelectOutlinerEntity { entity, context } => {
                    ShellCommand::SelectOutlinerEntity {
                        entity: *entity,
                        target: StructuralCommandTarget {
                            panel_instance_id: context.panel_instance_id,
                            active_tool_surface: context.active_tool_surface,
                            tab_stack_id: context.tab_stack_id,
                        },
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
                            target: StructuralCommandTarget {
                                panel_instance_id: context.panel_instance_id,
                                active_tool_surface: context.active_tool_surface,
                                tab_stack_id: context.tab_stack_id,
                            },
                            projection_epoch: routing.projection_epoch,
                        }
                    } else {
                        ShellCommand::NoOp
                    }
                }
                RoutedShellAction::ActivateInspectorField { index, context } => {
                    ShellCommand::ActivateInspectorField {
                        index: *index,
                        target: StructuralCommandTarget {
                            panel_instance_id: context.panel_instance_id,
                            active_tool_surface: context.active_tool_surface,
                            tab_stack_id: context.tab_stack_id,
                        },
                        projection_epoch: routing.projection_epoch,
                    }
                }
                RoutedShellAction::ActivateTab {
                    tab_stack_id,
                    panel_instance_id,
                } => ShellCommand::ActivateTab {
                    tab_stack_id: *tab_stack_id,
                    panel_instance_id: *panel_instance_id,
                    projection_epoch: routing.projection_epoch,
                },
                RoutedShellAction::FloatPanel {
                    tab_stack_id,
                    panel_instance_id,
                } => ShellCommand::FloatPanel {
                    tab_stack_id: *tab_stack_id,
                    panel_instance_id: *panel_instance_id,
                    projection_epoch: routing.projection_epoch,
                },
            };

            commands.push(command);
        }
    }

    commands
}

fn action_has_structural_context_match(
    widget_id: crate::WidgetId,
    action: &RoutedShellAction,
    routing: &ShellProjectionArtifacts,
) -> bool {
    let expected = match action {
        RoutedShellAction::SelectOutlinerEntity { context, .. }
        | RoutedShellAction::SelectViewportProduct { context, .. }
        | RoutedShellAction::ActivateInspectorField { context, .. } => Some(*context),
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
