//! File: domain/editor/editor_shell/src/ids/widget_ids.rs
//! Purpose: Stable widget ids for first editor shell composition.

use crate::WidgetId;

pub const ROOT_WIDGET_ID: WidgetId = WidgetId(1);

pub const TOOLBAR_ROOT_WIDGET_ID: WidgetId = WidgetId(10);
pub const TOOLBAR_ROW_WIDGET_ID: WidgetId = WidgetId(11);
pub const TOOLBAR_SELECT_BUTTON_WIDGET_ID: WidgetId = WidgetId(12);
pub const TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID: WidgetId = WidgetId(13);
pub const TOOLBAR_UNDO_BUTTON_WIDGET_ID: WidgetId = WidgetId(14);
pub const TOOLBAR_REDO_BUTTON_WIDGET_ID: WidgetId = WidgetId(15);
pub const TOOLBAR_SAVE_BUTTON_WIDGET_ID: WidgetId = WidgetId(16);
pub const TOOLBAR_LOAD_BUTTON_WIDGET_ID: WidgetId = WidgetId(17);
pub const TOOLBAR_DEBUG_LOGS_BUTTON_WIDGET_ID: WidgetId = WidgetId(18);

pub const BODY_ROOT_WIDGET_ID: WidgetId = WidgetId(20);
pub const LEFT_RIGHT_SPLIT_WIDGET_ID: WidgetId = WidgetId(21);
pub const CENTER_RIGHT_SPLIT_WIDGET_ID: WidgetId = WidgetId(22);
pub const BODY_CONSOLE_SPLIT_WIDGET_ID: WidgetId = WidgetId(23);

pub const OUTLINER_PANEL_WIDGET_ID: WidgetId = WidgetId(30);
pub const OUTLINER_TITLE_WIDGET_ID: WidgetId = WidgetId(31);
pub const OUTLINER_LIST_WIDGET_ID: WidgetId = WidgetId(32);

pub const VIEWPORT_PANEL_WIDGET_ID: WidgetId = WidgetId(40);
pub const VIEWPORT_TITLE_WIDGET_ID: WidgetId = WidgetId(41);
pub const VIEWPORT_STATUS_WIDGET_ID: WidgetId = WidgetId(42);

pub const INSPECTOR_PANEL_WIDGET_ID: WidgetId = WidgetId(50);
pub const INSPECTOR_TITLE_WIDGET_ID: WidgetId = WidgetId(51);
pub const INSPECTOR_LIST_WIDGET_ID: WidgetId = WidgetId(52);
pub const INSPECTOR_TARGET_WIDGET_ID: WidgetId = WidgetId(53);

pub const CONSOLE_PANEL_WIDGET_ID: WidgetId = WidgetId(60);
pub const CONSOLE_TITLE_WIDGET_ID: WidgetId = WidgetId(61);
pub const CONSOLE_LIST_WIDGET_ID: WidgetId = WidgetId(62);
pub const CONSOLE_LINE_WIDGET_ID_BASE: u64 = 20_000;

pub const OUTLINER_ROW_WIDGET_ID_BASE: u64 = 1_000;
pub const INSPECTOR_FIELD_WIDGET_ID_BASE: u64 = 10_000;

pub fn outliner_row_widget_id(index: usize) -> WidgetId {
    WidgetId(OUTLINER_ROW_WIDGET_ID_BASE + index as u64)
}

pub fn inspector_field_widget_id(index: usize) -> WidgetId {
    WidgetId(INSPECTOR_FIELD_WIDGET_ID_BASE + index as u64)
}

pub fn console_line_widget_id(index: usize) -> WidgetId {
    WidgetId(CONSOLE_LINE_WIDGET_ID_BASE + index as u64)
}

pub fn outliner_row_index(widget_id: WidgetId) -> Option<usize> {
    if widget_id.0 < OUTLINER_ROW_WIDGET_ID_BASE {
        return None;
    }

    let raw = widget_id.0 - OUTLINER_ROW_WIDGET_ID_BASE;
    usize::try_from(raw).ok()
}

pub fn inspector_field_index(widget_id: WidgetId) -> Option<usize> {
    if widget_id.0 < INSPECTOR_FIELD_WIDGET_ID_BASE {
        return None;
    }

    let raw = widget_id.0 - INSPECTOR_FIELD_WIDGET_ID_BASE;
    usize::try_from(raw).ok()
}
