//! Crate: editor_definition
//! Purpose: Editor-specific UI definition bindings without shell runtime execution.

pub mod availability;
pub mod binding;
pub mod command;
pub mod document;
pub mod form_editor_ui;
pub mod menu;
pub mod registry;
pub mod shortcut;
pub mod surface;
pub mod theme;
pub mod toolbar;
pub mod validate;
pub mod workspace;

pub use availability::*;
pub use binding::*;
pub use command::*;
pub use document::*;
pub use form_editor_ui::*;
pub use menu::*;
pub use registry::*;
pub use shortcut::*;
pub use surface::*;
pub use theme::*;
pub use toolbar::*;
pub use validate::*;
pub use workspace::*;

#[cfg(test)]
mod fixture_tests {
    use super::*;

    #[test]
    fn checked_in_editor_binding_fixture_parses() {
        let bindings: EditorDefinitionBindings = ron::from_str(include_str!(
            "../../../../assets/editor/ui/editor_bindings.ron"
        ))
        .expect("checked-in editor binding fixture should parse");
        assert_eq!(
            bindings.toolbar.template.as_str(),
            "runenwerk.editor.toolbar"
        );
        assert_eq!(bindings.surface_templates.len(), 5);
    }
}
