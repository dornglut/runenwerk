//! Crate: ui_definition
//! Purpose: Authored UI definitions, validation, normalization, and retained UI formation.

pub mod availability;
pub mod diagnostic;
pub mod embed;
pub mod form;
pub mod identity;
pub mod menu;
pub mod node;
pub mod normalize;
pub mod slot;
pub mod source;
pub mod template;
pub mod validate;
pub mod value;

pub use availability::*;
pub use diagnostic::*;
pub use embed::*;
pub use form::*;
pub use identity::*;
pub use menu::*;
pub use node::*;
pub use normalize::*;
pub use slot::*;
pub use source::*;
pub use template::*;
pub use validate::*;
pub use value::*;

#[cfg(test)]
mod fixture_tests {
    use super::*;

    const UI_FIXTURES: &[&str] = &[
        include_str!("../../../../assets/editor/ui/toolbar.ron"),
        include_str!("../../../../assets/editor/ui/shell_chrome.ron"),
        include_str!("../../../../assets/editor/ui/surfaces/inspector.ron"),
        include_str!("../../../../assets/editor/ui/surfaces/outliner.ron"),
        include_str!("../../../../assets/editor/ui/surfaces/entity_table.ron"),
        include_str!("../../../../assets/editor/ui/surfaces/console.ron"),
        include_str!("../../../../assets/editor/ui/surfaces/viewport.ron"),
    ];

    #[test]
    fn checked_in_ui_fixtures_parse_validate_and_normalize() {
        for source in UI_FIXTURES {
            let template: AuthoredUiTemplate =
                ron::from_str(source).expect("checked-in UI fixture should parse");
            let normalized = normalize_authored_template(template);
            assert!(
                !normalized.has_errors(),
                "fixture should normalize without errors: {:?}",
                normalized.diagnostics
            );
        }
    }
}
