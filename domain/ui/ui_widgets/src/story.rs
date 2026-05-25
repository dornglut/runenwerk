//! File: domain/ui/ui_widgets/src/story.rs
//! Purpose: Generic primitive widget story adapters.

use crate::{
    NumericInputConfig, TableColumn, TableRow, TreeRow, UiNode, VisibleWidgetScanRequirement,
    VisibleWidgetState, WidgetId, button, numeric_input, select, table, tabs, text_input, toggle,
    tree,
};
use ui_text::TextStyle;
use ui_theme::ThemeTokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrimitiveWidgetStoryKind {
    Button,
    TextInput,
    Toggle,
    NumericInput,
    Tabs,
    Select,
    Table,
    Tree,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveWidgetStory {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: PrimitiveWidgetStoryKind,
    pub root: UiNode,
    pub scan_requirement: VisibleWidgetScanRequirement,
}

impl PrimitiveWidgetStory {
    fn new(
        id: &'static str,
        label: &'static str,
        kind: PrimitiveWidgetStoryKind,
        root: UiNode,
    ) -> Self {
        Self {
            id,
            label,
            kind,
            root,
            scan_requirement: VisibleWidgetScanRequirement::strict_interactive([
                VisibleWidgetState::Default,
                VisibleWidgetState::Focused,
            ]),
        }
    }
}

pub fn primitive_widget_stories() -> Vec<PrimitiveWidgetStory> {
    let style = TextStyle::default();
    let theme = ThemeTokens::default();

    vec![
        PrimitiveWidgetStory::new(
            "ui.primitive.button.default",
            "Button / Default",
            PrimitiveWidgetStoryKind::Button,
            button(WidgetId(10_001), "Apply", style.clone(), theme.clone()),
        ),
        PrimitiveWidgetStory::new(
            "ui.primitive.text_input.default",
            "Text Input / Default",
            PrimitiveWidgetStoryKind::TextInput,
            text_input(
                WidgetId(10_002),
                "Player",
                "Name",
                style.clone(),
                theme.clone(),
            ),
        ),
        PrimitiveWidgetStory::new(
            "ui.primitive.toggle.checked",
            "Toggle / Checked",
            PrimitiveWidgetStoryKind::Toggle,
            toggle(WidgetId(10_003), "Snap", true, style.clone(), theme.clone()),
        ),
        PrimitiveWidgetStory::new(
            "ui.primitive.numeric_input.default",
            "Numeric Input / Default",
            PrimitiveWidgetStoryKind::NumericInput,
            numeric_input(
                WidgetId(10_004),
                NumericInputConfig::new(1.0, 0.1, Some(0.0), Some(2.0), 2),
                style.clone(),
                theme.clone(),
            ),
        ),
        PrimitiveWidgetStory::new(
            "ui.primitive.tabs.default",
            "Tabs / Default",
            PrimitiveWidgetStoryKind::Tabs,
            tabs(
                WidgetId(10_005),
                ["Layout", "Theme"],
                0,
                style.clone(),
                theme.clone(),
            ),
        ),
        PrimitiveWidgetStory::new(
            "ui.primitive.select.default",
            "Select / Default",
            PrimitiveWidgetStoryKind::Select,
            select(
                WidgetId(10_006),
                ["Scene", "Material"],
                Some(0),
                "Choose",
                style.clone(),
                theme.clone(),
            ),
        ),
        PrimitiveWidgetStory::new(
            "ui.primitive.table.default",
            "Table / Default",
            PrimitiveWidgetStoryKind::Table,
            table(
                WidgetId(10_007),
                [
                    TableColumn::new("Name", 96.0),
                    TableColumn::new("Kind", 72.0),
                ],
                [TableRow::new(["Cube", "Mesh"])],
                style.clone(),
                style.clone(),
                theme.clone(),
            ),
        ),
        PrimitiveWidgetStory::new(
            "ui.primitive.tree.default",
            "Tree / Default",
            PrimitiveWidgetStoryKind::Tree,
            tree(
                WidgetId(10_008),
                [
                    TreeRow::new("Scene", 0, true),
                    TreeRow::new("Cube", 1, false),
                ],
                style,
                theme,
            ),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_widget_stories_have_executable_roots() {
        let stories = primitive_widget_stories();

        assert_eq!(stories.len(), 8);
        assert!(stories.iter().all(|story| !story.id.trim().is_empty()));
        assert!(stories.iter().all(|story| !story.label.trim().is_empty()));
        assert!(stories.iter().all(|story| story.root.id.0 >= 10_001));
        assert!(
            stories
                .iter()
                .all(|story| story.scan_requirement.require_layout_bounds)
        );
    }
}
