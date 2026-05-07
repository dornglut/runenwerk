//! File: domain/editor/editor_shell/src/composition/mod.rs
//! Purpose: Build editor shell UI trees from shell view models.

pub mod build_console_panel;
pub mod build_editor_shell;
pub mod build_entity_table_panel;
pub mod build_inspector_panel;
pub mod build_outliner_panel;
pub mod build_toolbar;
pub mod build_viewport_panel;
pub mod shell_chrome_definition;
pub mod toolbar_definition;

pub use build_console_panel::build_console_panel;
pub use build_editor_shell::{
    ActiveTabDragVisualState, DockingInteractionVisualState, DockingPreviewDropTarget,
    EditorShellBuildResult, RoutedShellAction, ShellProjectionArtifacts, build_editor_shell_frame,
    build_editor_shell_frame_with_docking_visual_state,
};
pub use build_entity_table_panel::build_entity_table_panel;
pub use build_inspector_panel::build_inspector_panel;
pub use build_outliner_panel::build_outliner_panel;
pub use build_toolbar::build_toolbar;
pub use build_viewport_panel::build_viewport_panel;
pub use shell_chrome_definition::build_defined_tab_strip_from_frame;
pub use toolbar_definition::{
    build_defined_toolbar, build_defined_toolbar_menu_popup, route_slot_for_toolbar_name,
};
