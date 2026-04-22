//! File: domain/editor/editor_shell/src/ids/widget_ids.rs
//! Purpose: Stable widget ids for first editor shell composition.

use crate::{PanelHostId, TabStackId, WidgetId};

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
pub const BODY_FLOATING_SPLIT_WIDGET_ID: WidgetId = WidgetId(24);

pub const OUTLINER_PANEL_WIDGET_ID: WidgetId = WidgetId(30);
pub const OUTLINER_TITLE_WIDGET_ID: WidgetId = WidgetId(31);
pub const OUTLINER_LIST_WIDGET_ID: WidgetId = WidgetId(32);
pub const OUTLINER_BODY_WIDGET_ID: WidgetId = WidgetId(33);
pub const OUTLINER_SCROLL_WIDGET_ID: WidgetId = WidgetId(34);

pub const VIEWPORT_PANEL_WIDGET_ID: WidgetId = WidgetId(40);
pub const VIEWPORT_TITLE_WIDGET_ID: WidgetId = WidgetId(41);
pub const VIEWPORT_STATUS_WIDGET_ID: WidgetId = WidgetId(42);
pub const VIEWPORT_BODY_WIDGET_ID: WidgetId = WidgetId(43);
pub const VIEWPORT_CANVAS_WIDGET_ID: WidgetId = WidgetId(44);
pub const VIEWPORT_CHROME_WIDGET_ID: WidgetId = WidgetId(45);
pub const VIEWPORT_CHROME_CONTENT_WIDGET_ID: WidgetId = WidgetId(46);
pub const VIEWPORT_CANVAS_CONTENT_WIDGET_ID: WidgetId = WidgetId(47);
pub const VIEWPORT_CANVAS_LABEL_WIDGET_ID: WidgetId = WidgetId(48);
pub const VIEWPORT_SURFACE_EMBED_WIDGET_ID: WidgetId = WidgetId(49);
pub const VIEWPORT_PRODUCTS_TITLE_WIDGET_ID: WidgetId = WidgetId(70);
pub const VIEWPORT_PRODUCTS_LIST_WIDGET_ID: WidgetId = WidgetId(71);

pub const INSPECTOR_PANEL_WIDGET_ID: WidgetId = WidgetId(50);
pub const INSPECTOR_TITLE_WIDGET_ID: WidgetId = WidgetId(51);
pub const INSPECTOR_LIST_WIDGET_ID: WidgetId = WidgetId(52);
pub const INSPECTOR_TARGET_WIDGET_ID: WidgetId = WidgetId(53);
pub const INSPECTOR_BODY_WIDGET_ID: WidgetId = WidgetId(54);
pub const INSPECTOR_SCROLL_WIDGET_ID: WidgetId = WidgetId(55);

pub const CONSOLE_PANEL_WIDGET_ID: WidgetId = WidgetId(60);
pub const CONSOLE_TITLE_WIDGET_ID: WidgetId = WidgetId(61);
pub const CONSOLE_LIST_WIDGET_ID: WidgetId = WidgetId(62);
pub const CONSOLE_BODY_WIDGET_ID: WidgetId = WidgetId(63);
pub const CONSOLE_SCROLL_WIDGET_ID: WidgetId = WidgetId(64);
pub const CONSOLE_LINE_WIDGET_ID_BASE: u64 = 20_000;

pub const OUTLINER_ROW_WIDGET_ID_BASE: u64 = 1_000;
pub const INSPECTOR_FIELD_WIDGET_ID_BASE: u64 = 10_000;
pub const VIEWPORT_PRODUCT_BUTTON_WIDGET_ID_BASE: u64 = 30_000;
pub const TAB_STRIP_WIDGET_ID_BASE: u64 = 1_000_000;
pub const TAB_STACK_CONTAINER_WIDGET_ID_BASE: u64 = 1_050_000;
pub const TAB_BUTTON_WIDGET_ID_BASE: u64 = 1_200_000;
pub const TAB_DROP_ZONE_WIDGET_ID_BASE: u64 = 1_400_000;
pub const FLOATING_HOST_WIDGET_ID_BASE: u64 = 1_600_000;
pub const FLOATING_COLUMN_WIDGET_ID: WidgetId = WidgetId(1_800_001);
pub const FLOATING_DROP_ZONE_WIDGET_ID: WidgetId = WidgetId(1_800_002);

const STACK_WIDGET_STRIDE: u64 = 2048;
const HOST_WIDGET_STRIDE: u64 = 16;

pub fn outliner_row_widget_id(index: usize) -> WidgetId {
    WidgetId(OUTLINER_ROW_WIDGET_ID_BASE + index as u64)
}

pub fn inspector_field_widget_id(index: usize) -> WidgetId {
    WidgetId(INSPECTOR_FIELD_WIDGET_ID_BASE + index as u64)
}

pub fn console_line_widget_id(index: usize) -> WidgetId {
    WidgetId(CONSOLE_LINE_WIDGET_ID_BASE + index as u64)
}

pub fn viewport_product_button_widget_id(index: usize) -> WidgetId {
    WidgetId(VIEWPORT_PRODUCT_BUTTON_WIDGET_ID_BASE + index as u64)
}

pub fn tab_strip_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STRIP_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_container_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_CONTAINER_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_button_widget_id(tab_stack_id: TabStackId, tab_index: usize) -> WidgetId {
    WidgetId(
        TAB_BUTTON_WIDGET_ID_BASE + tab_stack_id.raw() * STACK_WIDGET_STRIDE + tab_index as u64,
    )
}

pub fn tab_drop_zone_widget_id(tab_stack_id: TabStackId, insert_index: usize) -> WidgetId {
    WidgetId(
        TAB_DROP_ZONE_WIDGET_ID_BASE
            + tab_stack_id.raw() * STACK_WIDGET_STRIDE
            + insert_index as u64,
    )
}

pub fn floating_host_widget_id(host_id: PanelHostId) -> WidgetId {
    WidgetId(FLOATING_HOST_WIDGET_ID_BASE + host_id.raw() * HOST_WIDGET_STRIDE)
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

pub fn viewport_product_button_index(widget_id: WidgetId) -> Option<usize> {
    if widget_id.0 < VIEWPORT_PRODUCT_BUTTON_WIDGET_ID_BASE {
        return None;
    }

    let raw = widget_id.0 - VIEWPORT_PRODUCT_BUTTON_WIDGET_ID_BASE;
    usize::try_from(raw).ok()
}
