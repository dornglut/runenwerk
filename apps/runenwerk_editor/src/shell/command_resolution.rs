//! File: apps/runenwerk_editor/src/shell/command_resolution.rs
//! Purpose: Compatibility adapters from authored editor command keys to catalog-backed shell actions.

use std::collections::BTreeMap;

use editor_definition::EditorMenuItemDefinition;
use editor_shell::RoutedShellAction;

use crate::shell::{
    ActiveEditorDefinitionCatalogs, EditorCommandAvailabilityContext, KnownEditorCommand,
    editor_command_catalog,
};

pub fn is_known_editor_command_key(key: &str) -> bool {
    editor_command_catalog().is_known_key(key)
}

pub fn active_route_actions_by_target(
    catalogs: &ActiveEditorDefinitionCatalogs,
    can_undo: bool,
    can_redo: bool,
) -> BTreeMap<String, RoutedShellAction> {
    let catalog = editor_command_catalog();
    let context = EditorCommandAvailabilityContext { can_undo, can_redo };
    let mut routes = BTreeMap::new();

    for descriptor in catalog.descriptors() {
        for route_target in descriptor.route_targets() {
            routes.insert(
                route_target.to_string(),
                descriptor.routed_shell_action(context),
            );
        }
    }

    for binding in catalogs
        .command_bindings()
        .values()
        .flat_map(|set| set.bindings.iter())
    {
        if let Some(command) = catalog.command_for_key(&binding.command) {
            let action = command.to_routed_shell_action(can_undo, can_redo);
            routes.insert(binding.route_target.clone(), action.clone());
            routes.insert(binding.command.clone(), action);
        }
    }

    let mut menu_commands = Vec::new();
    for menu in catalogs.menus().values() {
        collect_menu_commands(&menu.items, &mut menu_commands);
    }
    for command in menu_commands {
        if let Some(command_key) = KnownEditorCommand::from_key(command) {
            routes.insert(
                command.to_string(),
                command_key.to_routed_shell_action(can_undo, can_redo),
            );
        }
    }

    routes
}

fn collect_menu_commands<'a>(items: &'a [EditorMenuItemDefinition], output: &mut Vec<&'a str>) {
    for item in items {
        if let Some(command) = item.command.as_deref() {
            output.push(command);
        }
        collect_menu_commands(&item.children, output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_route_actions_cover_every_catalog_route_target_without_shell_fallback() {
        let routes = active_route_actions_by_target(
            &ActiveEditorDefinitionCatalogs::default(),
            false,
            false,
        );

        for descriptor in editor_command_catalog().descriptors() {
            for route_target in descriptor.route_targets() {
                assert!(
                    routes.contains_key(route_target),
                    "catalog route target '{route_target}' should have a routed shell action"
                );
            }
        }
    }

    #[test]
    fn unavailable_catalog_commands_project_disabled_routes() {
        let routes = active_route_actions_by_target(
            &ActiveEditorDefinitionCatalogs::default(),
            false,
            false,
        );

        assert!(matches!(
            routes.get("editor.toolbar.file.save_as"),
            Some(RoutedShellAction::RunToolbarCommand { enabled: false, .. })
        ));
        assert!(matches!(
            routes.get("editor.toolbar.window.load_custom_workspace"),
            Some(RoutedShellAction::RunToolbarCommand { enabled: false, .. })
        ));
    }
}
