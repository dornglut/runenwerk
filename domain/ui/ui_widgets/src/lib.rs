//! File: domain/ui/ui_widgets/src/lib.rs
//! Crate: ui_widgets
//! Purpose: First widget constructors and helpers over ui_tree node contracts.

pub use ::ui_tree::*;

mod button;
mod label;
mod numeric_input;
mod panel;
mod scroll;
mod split;
mod stack;
mod tabs;
mod text_input;
mod toggle;
mod viewport_surface_embed;

pub use button::button;
pub use label::label;
pub use numeric_input::numeric_input;
pub use panel::panel;
pub use scroll::{hscroll, scroll, vscroll};
pub use split::split;
pub use stack::{hstack, hstack_with_policies, stack, vstack, vstack_with_policies};
pub use tabs::tabs;
pub use text_input::text_input;
pub use toggle::toggle;
pub use viewport_surface_embed::viewport_surface_embed;

#[cfg(test)]
mod tests {
    use super::*;
    use ui_text::TextStyle;
    use ui_theme::ThemeTokens;

    #[test]
    fn constructors_emit_expected_node_kinds() {
        let style = TextStyle::default();
        let theme = ThemeTokens::default();
        assert!(matches!(
            button(WidgetId(1), "Apply", style.clone(), theme.clone()).kind,
            UiNodeKind::Button(_)
        ));
        assert!(matches!(
            text_input(
                WidgetId(2),
                "value",
                "placeholder",
                style.clone(),
                theme.clone()
            )
            .kind,
            UiNodeKind::TextInput(_)
        ));
        assert!(matches!(
            toggle(WidgetId(3), "Snap", true, style.clone(), theme.clone()).kind,
            UiNodeKind::Toggle(_)
        ));
        assert!(matches!(
            numeric_input(
                WidgetId(4),
                1.0,
                0.1,
                Some(0.0),
                Some(2.0),
                2,
                style.clone(),
                theme.clone()
            )
            .kind,
            UiNodeKind::NumericInput(_)
        ));
        assert!(matches!(
            tabs(WidgetId(5), ["A", "B"], 0, style, theme).kind,
            UiNodeKind::Tabs(_)
        ));
        assert!(matches!(
            hscroll(WidgetId(6), ThemeTokens::default(), Vec::new()).kind,
            UiNodeKind::Scroll(_)
        ));
        assert!(matches!(
            vscroll(WidgetId(7), ThemeTokens::default(), Vec::new()).kind,
            UiNodeKind::Scroll(_)
        ));
    }
}
