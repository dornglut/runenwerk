//! File: domain/editor/editor_shell/src/commands/map_interactions.rs
//! Purpose: Map semantic UI interactions to shell commands.

use crate::{
    EntityTableSurfaceAction, InspectorSurfaceAction, OutlinerSurfaceAction, RoutedShellAction,
    ShellCommand, ShellProjectionArtifacts, StructuralCommandTarget,
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
            UiInteraction::SelectChanged { target, index } => {
                commands.push(command_for_select_change(*target, *index, routing));
            }
            UiInteraction::Toggled { target, checked } => {
                commands.push(command_for_toggle(*target, *checked, routing));
            }
            UiInteraction::NumericStepped { target, value } => {
                commands.push(command_for_numeric_step(*target, *value, routing));
            }
            UiInteraction::TabSelected { target, index } => {
                commands.push(command_for_tab_selection(*target, *index, routing));
            }
            UiInteraction::TableRowSelected { target, row_index } => {
                commands.push(command_for_table_row(*target, *row_index, routing));
            }
            UiInteraction::TreeRowSelected { target, row_index } => {
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
                        SurfaceLocalAction::EntityTable(
                            EntityTableSurfaceAction::AppendSearchText { .. },
                        ) | SurfaceLocalAction::Inspector(
                            InspectorSurfaceAction::EditFieldText { .. },
                        )
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
                            SurfaceLocalAction::EntityTable(
                                EntityTableSurfaceAction::AppendSearchText { .. },
                            ),
                        ) => commands.push(ShellCommand::DispatchSurfaceLocalAction {
                            provider_id: *provider_id,
                            tool_surface_instance_id: *tool_surface_instance_id,
                            target: command_target(*context),
                            action: SurfaceLocalAction::EntityTable(
                                EntityTableSurfaceAction::BackspaceSearch,
                            ),
                            projection_epoch: routing.projection_epoch,
                        }),
                        (
                            Key::Backspace | Key::Delete,
                            SurfaceLocalAction::Inspector(InspectorSurfaceAction::EditFieldText {
                                index,
                                ..
                            }),
                        ) => commands.push(ShellCommand::DispatchSurfaceLocalAction {
                            provider_id: *provider_id,
                            tool_surface_instance_id: *tool_surface_instance_id,
                            target: command_target(*context),
                            action: SurfaceLocalAction::Inspector(
                                InspectorSurfaceAction::BackspaceFieldText { index: *index },
                            ),
                            projection_epoch: routing.projection_epoch,
                        }),
                        (
                            Key::Enter,
                            SurfaceLocalAction::Inspector(InspectorSurfaceAction::EditFieldText {
                                index,
                                ..
                            }),
                        ) => commands.push(ShellCommand::DispatchSurfaceLocalAction {
                            provider_id: *provider_id,
                            tool_surface_instance_id: *tool_surface_instance_id,
                            target: command_target(*context),
                            action: SurfaceLocalAction::Inspector(
                                InspectorSurfaceAction::CommitFieldText { index: *index },
                            ),
                            projection_epoch: routing.projection_epoch,
                        }),
                        (
                            Key::Escape,
                            SurfaceLocalAction::Inspector(InspectorSurfaceAction::EditFieldText {
                                index,
                                ..
                            }),
                        ) => commands.push(ShellCommand::DispatchSurfaceLocalAction {
                            provider_id: *provider_id,
                            tool_surface_instance_id: *tool_surface_instance_id,
                            target: command_target(*context),
                            action: SurfaceLocalAction::Inspector(
                                InspectorSurfaceAction::CancelFieldText { index: *index },
                            ),
                            projection_epoch: routing.projection_epoch,
                        }),
                        _ => {}
                    }
                }
            }
            UiInteraction::HoveredChanged { .. }
            | UiInteraction::PressedChanged { .. }
            | UiInteraction::FocusChanged(_)
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
        RoutedShellAction::ActivateRotateTool => ShellCommand::ActivateRotateTool,
        RoutedShellAction::ActivateScaleTool => ShellCommand::ActivateScaleTool,
        RoutedShellAction::ToggleToolbarMenu { menu } => {
            ShellCommand::ToggleToolbarMenu { menu: *menu }
        }
        RoutedShellAction::ToggleTabStackActionMenu {
            tab_stack_id,
            anchor_widget_id,
        } => ShellCommand::ToggleTabStackActionMenu {
            tab_stack_id: *tab_stack_id,
            anchor_widget_id: *anchor_widget_id,
        },
        RoutedShellAction::ToggleTabStackSurfaceMenu {
            tab_stack_id,
            anchor_widget_id,
        } => ShellCommand::ToggleTabStackSurfaceMenu {
            tab_stack_id: *tab_stack_id,
            anchor_widget_id: *anchor_widget_id,
        },
        RoutedShellAction::ToggleTabStackCreateSurfaceMenu {
            tab_stack_id,
            anchor_widget_id,
        } => ShellCommand::ToggleTabStackCreateSurfaceMenu {
            tab_stack_id: *tab_stack_id,
            anchor_widget_id: *anchor_widget_id,
        },
        RoutedShellAction::SwitchPanelToolSurfaceKindTo {
            panel_instance_id,
            tool_surface_kind,
        } => ShellCommand::SwitchPanelToolSurfaceKind {
            panel_instance_id: *panel_instance_id,
            tool_surface_kind: *tool_surface_kind,
            projection_epoch: routing.projection_epoch,
        },
        RoutedShellAction::RunToolbarCommand { command, enabled } => {
            if *enabled {
                ShellCommand::RunToolbarCommand { command: *command }
            } else {
                ShellCommand::NoOp
            }
        }
        RoutedShellAction::SwitchWorkspaceProfile {
            profile_id,
            enabled,
        } => {
            if *enabled {
                ShellCommand::SwitchWorkspaceProfile {
                    profile_id: *profile_id,
                }
            } else {
                ShellCommand::NoOp
            }
        }
        RoutedShellAction::CloseWorkspaceProfile {
            profile_id,
            enabled,
        } => {
            if *enabled {
                ShellCommand::CloseWorkspaceProfile {
                    profile_id: *profile_id,
                }
            } else {
                ShellCommand::NoOp
            }
        }
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
        RoutedShellAction::CreatePanelTab {
            tab_stack_id,
            tool_surface_kind,
        } => ShellCommand::CreatePanelTab {
            tab_stack_id: *tab_stack_id,
            tool_surface_kind: *tool_surface_kind,
            projection_epoch: routing.projection_epoch,
        },
        RoutedShellAction::ClosePanelTab {
            tab_stack_id,
            panel_instance_id,
        } => ShellCommand::ClosePanelTab {
            tab_stack_id: *tab_stack_id,
            panel_instance_id: *panel_instance_id,
            projection_epoch: routing.projection_epoch,
        },
        RoutedShellAction::SplitTabStackArea {
            tab_stack_id,
            axis,
            tool_surface_kind,
        } => ShellCommand::SplitTabStackArea {
            tab_stack_id: *tab_stack_id,
            axis: *axis,
            tool_surface_kind: *tool_surface_kind,
            projection_epoch: routing.projection_epoch,
        },
        RoutedShellAction::DuplicateTabStackArea { tab_stack_id } => {
            ShellCommand::DuplicateTabStackArea {
                tab_stack_id: *tab_stack_id,
                projection_epoch: routing.projection_epoch,
            }
        }
        RoutedShellAction::CloseTabStackArea { tab_stack_id } => ShellCommand::CloseTabStackArea {
            tab_stack_id: *tab_stack_id,
            projection_epoch: routing.projection_epoch,
        },
        RoutedShellAction::ResetTabStackArea {
            tab_stack_id,
            tool_surface_kind,
        } => ShellCommand::ResetTabStackArea {
            tab_stack_id: *tab_stack_id,
            tool_surface_kind: *tool_surface_kind,
            projection_epoch: routing.projection_epoch,
        },
        RoutedShellAction::LockTabStackAreaType {
            tab_stack_id,
            locked_tool_surface_kind,
        } => ShellCommand::LockTabStackAreaType {
            tab_stack_id: *tab_stack_id,
            locked_tool_surface_kind: *locked_tool_surface_kind,
            projection_epoch: routing.projection_epoch,
        },
        RoutedShellAction::SwitchPanelToolSurfaceKind { .. } => ShellCommand::NoOp,
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

fn command_for_select_change(
    widget_id: crate::WidgetId,
    index: usize,
    routing: &ShellProjectionArtifacts,
) -> ShellCommand {
    match routing.widget_actions_by_id.get(&widget_id) {
        Some(RoutedShellAction::SwitchPanelToolSurfaceKind {
            panel_instance_id: Some(panel_instance_id),
            tool_surface_kinds,
            ..
        }) => {
            let Some(tool_surface_kind) = tool_surface_kinds.get(index).copied() else {
                return ShellCommand::NoOp;
            };
            ShellCommand::SwitchPanelToolSurfaceKind {
                panel_instance_id: *panel_instance_id,
                tool_surface_kind,
                projection_epoch: routing.projection_epoch,
            }
        }
        Some(RoutedShellAction::DispatchSurfaceLocalAction {
            provider_id,
            tool_surface_instance_id,
            action:
                SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SelectComponentFilter {
                    filters,
                }),
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
            let Some(filter) = filters.get(index).copied() else {
                return ShellCommand::NoOp;
            };
            ShellCommand::DispatchSurfaceLocalAction {
                provider_id: *provider_id,
                tool_surface_instance_id: *tool_surface_instance_id,
                target: command_target(*context),
                action: SurfaceLocalAction::EntityTable(
                    EntityTableSurfaceAction::SetComponentFilter { filter },
                ),
                projection_epoch: routing.projection_epoch,
            }
        }
        Some(RoutedShellAction::DispatchSurfaceLocalAction {
            provider_id,
            tool_surface_instance_id,
            action:
                SurfaceLocalAction::Inspector(InspectorSurfaceAction::SelectFieldEnum {
                    index: field_index,
                    options,
                }),
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
            let Some(value) = options.get(index).cloned() else {
                return ShellCommand::NoOp;
            };
            ShellCommand::DispatchSurfaceLocalAction {
                provider_id: *provider_id,
                tool_surface_instance_id: *tool_surface_instance_id,
                target: command_target(*context),
                action: SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldEnum {
                    index: *field_index,
                    value,
                }),
                projection_epoch: routing.projection_epoch,
            }
        }
        _ => ShellCommand::NoOp,
    }
}

fn command_for_toggle(
    widget_id: crate::WidgetId,
    checked: bool,
    routing: &ShellProjectionArtifacts,
) -> ShellCommand {
    match routing.widget_actions_by_id.get(&widget_id) {
        Some(RoutedShellAction::LockTabStackAreaType {
            tab_stack_id,
            locked_tool_surface_kind,
        }) => ShellCommand::LockTabStackAreaType {
            tab_stack_id: *tab_stack_id,
            locked_tool_surface_kind: checked.then_some(*locked_tool_surface_kind).flatten(),
            projection_epoch: routing.projection_epoch,
        },
        Some(RoutedShellAction::DispatchSurfaceLocalAction {
            provider_id,
            tool_surface_instance_id,
            action,
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
            let Some(action) = surface_toggle_action(action, checked) else {
                return ShellCommand::NoOp;
            };
            ShellCommand::DispatchSurfaceLocalAction {
                provider_id: *provider_id,
                tool_surface_instance_id: *tool_surface_instance_id,
                target: command_target(*context),
                action,
                projection_epoch: routing.projection_epoch,
            }
        }
        _ => ShellCommand::NoOp,
    }
}

fn command_for_numeric_step(
    widget_id: crate::WidgetId,
    value: f64,
    routing: &ShellProjectionArtifacts,
) -> ShellCommand {
    match routing.widget_actions_by_id.get(&widget_id) {
        Some(RoutedShellAction::DispatchSurfaceLocalAction {
            provider_id,
            tool_surface_instance_id,
            action:
                SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldNumber { index, .. }),
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
            ShellCommand::DispatchSurfaceLocalAction {
                provider_id: *provider_id,
                tool_surface_instance_id: *tool_surface_instance_id,
                target: command_target(*context),
                action: SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldNumber {
                    index: *index,
                    value,
                }),
                projection_epoch: routing.projection_epoch,
            }
        }
        _ => ShellCommand::NoOp,
    }
}

fn command_for_tab_selection(
    widget_id: crate::WidgetId,
    index: usize,
    routing: &ShellProjectionArtifacts,
) -> ShellCommand {
    let Some(RoutedShellAction::ActivateTab { tab_stack_id, .. }) =
        routing.widget_actions_by_id.get(&widget_id)
    else {
        return ShellCommand::NoOp;
    };
    let Some(route) = routing
        .workspace
        .tab_button_route_by_widget_id
        .values()
        .filter(|route| route.tab_stack_id == *tab_stack_id)
        .nth(index)
    else {
        return ShellCommand::NoOp;
    };
    ShellCommand::SetTabStackActivePanel {
        tab_stack_id: route.tab_stack_id,
        panel_instance_id: route.panel_instance_id,
        projection_epoch: routing.projection_epoch,
    }
}

fn surface_text_action(action: &SurfaceLocalAction, text: String) -> SurfaceLocalAction {
    match action {
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::AppendSearchText { .. }) => {
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::AppendSearchText { text })
        }
        SurfaceLocalAction::Inspector(InspectorSurfaceAction::EditFieldText { index, .. }) => {
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::EditFieldText {
                index: *index,
                text,
            })
        }
        _ => action.clone(),
    }
}

fn surface_toggle_action(action: &SurfaceLocalAction, checked: bool) -> Option<SurfaceLocalAction> {
    match action {
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SetSelectedOnly { .. }) => Some(
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SetSelectedOnly {
                selected_only: checked,
            }),
        ),
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SetHierarchyFilter {
            ..
        }) => Some(SurfaceLocalAction::EntityTable(
            EntityTableSurfaceAction::SetHierarchyFilter {
                filter: if checked {
                    crate::EntityTableHierarchyFilter::RootsOnly
                } else {
                    crate::EntityTableHierarchyFilter::All
                },
            },
        )),
        SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldBool { index, .. }) => Some(
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldBool {
                index: *index,
                value: checked,
            }),
        ),
        SurfaceLocalAction::Viewport(crate::ViewportSurfaceAction::ToggleDetails) => Some(
            SurfaceLocalAction::Viewport(crate::ViewportSurfaceAction::ToggleDetails),
        ),
        SurfaceLocalAction::Viewport(crate::ViewportSurfaceAction::ToggleStatistics) => Some(
            SurfaceLocalAction::Viewport(crate::ViewportSurfaceAction::ToggleStatistics),
        ),
        SurfaceLocalAction::Viewport(crate::ViewportSurfaceAction::SetRootBackgroundOpaque {
            viewport_id,
            ..
        }) => Some(SurfaceLocalAction::Viewport(
            crate::ViewportSurfaceAction::SetRootBackgroundOpaque {
                viewport_id: *viewport_id,
                enabled: checked,
            },
        )),
        _ => None,
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
            action:
                SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SelectRow { entities }),
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
                action: SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SelectEntity {
                    entity,
                }),
                projection_epoch: routing.projection_epoch,
            }
        }
        Some(RoutedShellAction::DispatchSurfaceLocalAction {
            provider_id,
            tool_surface_instance_id,
            action: SurfaceLocalAction::Outliner(OutlinerSurfaceAction::SelectRow { entities }),
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
                action: SurfaceLocalAction::Outliner(OutlinerSurfaceAction::SelectEntity {
                    entity,
                }),
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
