//! File: domain/editor/editor_shell/src/composition/mod.rs
//! Purpose: Build editor shell UI trees from shell view models.

pub mod build_editor_shell;
pub mod build_inspector_panel;
pub mod build_outliner_panel;
pub mod build_toolbar;
pub mod build_viewport_panel;

pub use build_editor_shell::build_editor_shell;
pub use build_inspector_panel::build_inspector_panel;
pub use build_outliner_panel::build_outliner_panel;
pub use build_toolbar::build_toolbar;
pub use build_viewport_panel::build_viewport_panel;