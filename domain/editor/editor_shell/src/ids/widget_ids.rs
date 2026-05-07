//! File: domain/editor/editor_shell/src/ids/widget_ids.rs
//! Purpose: Stable widget ids for first editor shell composition.

use crate::{PanelHostId, TabStackId, WidgetId, WorkspaceProfileId};

pub const ROOT_WIDGET_ID: WidgetId = WidgetId(1);

pub const TOOLBAR_ROOT_WIDGET_ID: WidgetId = WidgetId(10);
pub const TOOLBAR_ROW_WIDGET_ID: WidgetId = WidgetId(11);
pub const TOOLBAR_FILE_MENU_WIDGET_ID: WidgetId = WidgetId(12);
pub const TOOLBAR_EDIT_MENU_WIDGET_ID: WidgetId = WidgetId(13);
pub const TOOLBAR_WINDOW_MENU_WIDGET_ID: WidgetId = WidgetId(14);
pub const TOOLBAR_SEPARATOR_WIDGET_ID: WidgetId = WidgetId(15);
pub const TOOLBAR_SCENE_WORKSPACE_WIDGET_ID: WidgetId = WidgetId(16);
pub const TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID: WidgetId = WidgetId(17);
pub const TOOLBAR_ADD_WORKSPACE_WIDGET_ID: WidgetId = WidgetId(18);
pub const TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID: WidgetId = WidgetId(30_018);
pub const TOOLBAR_WORKSPACE_CLOSE_WIDGET_ID_BASE: u64 = 30_100;
pub const TOOLBAR_MENU_ROW_WIDGET_ID: WidgetId = WidgetId(19);
pub const TOOLBAR_MENU_POPUP_WIDGET_ID: WidgetId = WidgetId(90_997);
pub const TOOLBAR_SELECT_BUTTON_WIDGET_ID: WidgetId = WidgetId(80_001);
pub const TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID: WidgetId = WidgetId(80_002);
pub const TOOLBAR_ROTATE_BUTTON_WIDGET_ID: WidgetId = WidgetId(80_003);
pub const TOOLBAR_SCALE_BUTTON_WIDGET_ID: WidgetId = WidgetId(80_004);
pub const TOOLBAR_UNDO_BUTTON_WIDGET_ID: WidgetId = WidgetId(80_005);
pub const TOOLBAR_REDO_BUTTON_WIDGET_ID: WidgetId = WidgetId(80_006);
pub const TOOLBAR_SAVE_BUTTON_WIDGET_ID: WidgetId = WidgetId(80_007);
pub const TOOLBAR_LOAD_BUTTON_WIDGET_ID: WidgetId = WidgetId(80_008);
pub const TOOLBAR_DEBUG_LOGS_BUTTON_WIDGET_ID: WidgetId = WidgetId(28);
pub const TOOLBAR_SCROLL_WIDGET_ID: WidgetId = WidgetId(29);
pub const TOOLBAR_MENU_ITEM_WIDGET_ID_BASE: u64 = 90_000;
pub const TOOLBAR_ROWS_WIDGET_ID: WidgetId = WidgetId(90_999);

pub const BODY_ROOT_WIDGET_ID: WidgetId = WidgetId(20);
pub const LEFT_RIGHT_SPLIT_WIDGET_ID: WidgetId = WidgetId(21);
pub const CENTER_RIGHT_SPLIT_WIDGET_ID: WidgetId = WidgetId(22);
pub const BODY_CONSOLE_SPLIT_WIDGET_ID: WidgetId = WidgetId(23);
pub const BODY_FLOATING_SPLIT_WIDGET_ID: WidgetId = WidgetId(24);
pub const LEFT_RIGHT_SPLIT_HANDLE_WIDGET_ID: WidgetId = WidgetId(25);
pub const CENTER_RIGHT_SPLIT_HANDLE_WIDGET_ID: WidgetId = WidgetId(26);
pub const BODY_CONSOLE_SPLIT_HANDLE_WIDGET_ID: WidgetId = WidgetId(27);

pub const OUTLINER_PANEL_WIDGET_ID: WidgetId = WidgetId(30);
pub const OUTLINER_TITLE_WIDGET_ID: WidgetId = WidgetId(31);
pub const OUTLINER_LIST_WIDGET_ID: WidgetId = WidgetId(32);
pub const OUTLINER_BODY_WIDGET_ID: WidgetId = WidgetId(33);
pub const OUTLINER_SCROLL_WIDGET_ID: WidgetId = WidgetId(34);

pub const ENTITY_TABLE_PANEL_WIDGET_ID: WidgetId = WidgetId(35);
pub const ENTITY_TABLE_TITLE_WIDGET_ID: WidgetId = WidgetId(36);
pub const ENTITY_TABLE_SEARCH_WIDGET_ID: WidgetId = WidgetId(37);
pub const ENTITY_TABLE_HEADER_ROW_WIDGET_ID: WidgetId = WidgetId(38);
pub const ENTITY_TABLE_LIST_WIDGET_ID: WidgetId = WidgetId(39);
pub const ENTITY_TABLE_SCROLL_WIDGET_ID: WidgetId = WidgetId(65);
pub const ENTITY_TABLE_BODY_WIDGET_ID: WidgetId = WidgetId(66);
pub const ENTITY_TABLE_HEADER_SCROLL_WIDGET_ID: WidgetId = WidgetId(67);
pub const ENTITY_TABLE_TABLE_SCROLL_WIDGET_ID: WidgetId = WidgetId(68);

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
pub const VIEWPORT_DETAILS_TOGGLE_WIDGET_ID: WidgetId = WidgetId(72);
pub const VIEWPORT_DETAILS_PANEL_WIDGET_ID: WidgetId = WidgetId(73);
pub const VIEWPORT_DETAILS_LABEL_WIDGET_ID: WidgetId = WidgetId(74);
pub const VIEWPORT_OPTIONS_BUTTON_WIDGET_ID: WidgetId = WidgetId(75);
pub const VIEWPORT_OPTIONS_POPUP_WIDGET_ID: WidgetId = WidgetId(76);
pub const VIEWPORT_STATISTICS_TOGGLE_WIDGET_ID: WidgetId = WidgetId(77);
pub const VIEWPORT_STATISTICS_LABEL_WIDGET_ID: WidgetId = WidgetId(78);

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
pub const CONSOLE_HSCROLL_WIDGET_ID: WidgetId = WidgetId(69);
pub const CONSOLE_LINE_WIDGET_ID_BASE: u64 = 20_000;

pub const OUTLINER_ROW_WIDGET_ID_BASE: u64 = 1_000;
pub const INSPECTOR_FIELD_WIDGET_ID_BASE: u64 = 10_000;
pub const INSPECTOR_FIELD_FOCUS_WIDGET_ID_BASE: u64 = 15_000;
pub const VIEWPORT_PRODUCT_BUTTON_WIDGET_ID_BASE: u64 = 30_000;
pub const ENTITY_TABLE_ROW_WIDGET_ID_BASE: u64 = 40_000;
pub const ENTITY_TABLE_SORT_BUTTON_WIDGET_ID_BASE: u64 = 41_000;
pub const TAB_STRIP_WIDGET_ID_BASE: u64 = 1_000_000;
pub const TAB_STACK_CONTAINER_WIDGET_ID_BASE: u64 = 1_050_000;
pub const TAB_STRIP_SCROLL_WIDGET_ID_BASE: u64 = 1_100_000;
pub const TAB_BUTTON_WIDGET_ID_BASE: u64 = 1_200_000;
pub const TAB_DROP_ZONE_WIDGET_ID_BASE: u64 = 1_400_000;
pub const TAB_CLOSE_BUTTON_WIDGET_ID_BASE: u64 = 1_500_000;
pub const TAB_CLOSE_OVERLAY_WIDGET_ID_BASE: u64 = 1_550_000;
pub const FLOATING_HOST_WIDGET_ID_BASE: u64 = 1_600_000;
pub const TAB_STACK_NEW_TAB_BUTTON_WIDGET_ID_BASE: u64 = 1_950_000;
pub const TAB_STACK_SPLIT_HORIZONTAL_BUTTON_WIDGET_ID_BASE: u64 = 2_000_000;
pub const TAB_STACK_SPLIT_VERTICAL_BUTTON_WIDGET_ID_BASE: u64 = 2_050_000;
pub const TAB_STACK_DUPLICATE_BUTTON_WIDGET_ID_BASE: u64 = 2_100_000;
pub const TAB_STACK_CLOSE_AREA_BUTTON_WIDGET_ID_BASE: u64 = 2_150_000;
pub const TAB_STACK_RESET_AREA_BUTTON_WIDGET_ID_BASE: u64 = 2_200_000;
pub const TAB_STACK_LOCK_TYPE_TOGGLE_WIDGET_ID_BASE: u64 = 2_250_000;
pub const TAB_STACK_SWITCH_SURFACE_BUTTON_WIDGET_ID_BASE: u64 = 2_260_000;
pub const TAB_STACK_ACTION_MENU_BUTTON_WIDGET_ID_BASE: u64 = 2_270_000;
pub const TAB_STACK_ACTION_MENU_POPUP_WIDGET_ID_BASE: u64 = 2_280_000;
pub const TAB_STACK_SURFACE_MENU_POPUP_WIDGET_ID_BASE: u64 = 2_285_000;
pub const TAB_STACK_SURFACE_MENU_ITEM_WIDGET_ID_BASE: u64 = 2_290_000;
pub const WORKSPACE_SPLIT_HOST_WIDGET_ID_BASE: u64 = 2_300_000;
pub const WORKSPACE_SPLIT_HANDLE_WIDGET_ID_BASE: u64 = 2_350_000;
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

pub fn inspector_field_focus_widget_id(index: usize) -> WidgetId {
    WidgetId(INSPECTOR_FIELD_FOCUS_WIDGET_ID_BASE + index as u64)
}

pub fn console_line_widget_id(index: usize) -> WidgetId {
    WidgetId(CONSOLE_LINE_WIDGET_ID_BASE + index as u64)
}

pub fn viewport_product_button_widget_id(index: usize) -> WidgetId {
    WidgetId(VIEWPORT_PRODUCT_BUTTON_WIDGET_ID_BASE + index as u64)
}

pub fn entity_table_row_widget_id(index: usize) -> WidgetId {
    WidgetId(ENTITY_TABLE_ROW_WIDGET_ID_BASE + index as u64)
}

pub fn entity_table_sort_button_widget_id(index: usize) -> WidgetId {
    WidgetId(ENTITY_TABLE_SORT_BUTTON_WIDGET_ID_BASE + index as u64)
}

pub fn toolbar_menu_item_widget_id(index: usize) -> WidgetId {
    WidgetId(TOOLBAR_MENU_ITEM_WIDGET_ID_BASE + index as u64)
}

pub fn toolbar_workspace_close_widget_id(profile_id: WorkspaceProfileId) -> WidgetId {
    WidgetId(TOOLBAR_WORKSPACE_CLOSE_WIDGET_ID_BASE + profile_id.raw())
}

pub fn tab_strip_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STRIP_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_container_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_CONTAINER_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_strip_scroll_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STRIP_SCROLL_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_button_widget_id(tab_stack_id: TabStackId, tab_index: usize) -> WidgetId {
    WidgetId(
        TAB_BUTTON_WIDGET_ID_BASE + tab_stack_id.raw() * STACK_WIDGET_STRIDE + tab_index as u64,
    )
}

pub fn tab_close_button_widget_id(tab_stack_id: TabStackId, tab_index: usize) -> WidgetId {
    WidgetId(
        TAB_CLOSE_BUTTON_WIDGET_ID_BASE
            + tab_stack_id.raw() * STACK_WIDGET_STRIDE
            + tab_index as u64,
    )
}

pub fn tab_close_overlay_widget_id(tab_stack_id: TabStackId, tab_index: usize) -> WidgetId {
    WidgetId(
        TAB_CLOSE_OVERLAY_WIDGET_ID_BASE
            + tab_stack_id.raw() * STACK_WIDGET_STRIDE
            + tab_index as u64,
    )
}

pub fn tab_drop_zone_widget_id(tab_stack_id: TabStackId, insert_index: usize) -> WidgetId {
    WidgetId(
        TAB_DROP_ZONE_WIDGET_ID_BASE
            + tab_stack_id.raw() * STACK_WIDGET_STRIDE
            + insert_index as u64,
    )
}

pub fn tab_stack_new_tab_button_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_NEW_TAB_BUTTON_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_split_horizontal_button_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_SPLIT_HORIZONTAL_BUTTON_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_split_vertical_button_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_SPLIT_VERTICAL_BUTTON_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_duplicate_button_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_DUPLICATE_BUTTON_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_close_area_button_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_CLOSE_AREA_BUTTON_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_reset_area_button_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_RESET_AREA_BUTTON_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_lock_type_toggle_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_LOCK_TYPE_TOGGLE_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_switch_surface_button_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_SWITCH_SURFACE_BUTTON_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_action_menu_button_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_ACTION_MENU_BUTTON_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_action_menu_popup_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_ACTION_MENU_POPUP_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_surface_menu_popup_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_SURFACE_MENU_POPUP_WIDGET_ID_BASE + tab_stack_id.raw())
}

pub fn tab_stack_surface_menu_item_widget_id(
    tab_stack_id: TabStackId,
    item_index: usize,
) -> WidgetId {
    WidgetId(
        TAB_STACK_SURFACE_MENU_ITEM_WIDGET_ID_BASE
            + tab_stack_id.raw() * STACK_WIDGET_STRIDE
            + item_index as u64,
    )
}

pub fn floating_host_widget_id(host_id: PanelHostId) -> WidgetId {
    WidgetId(FLOATING_HOST_WIDGET_ID_BASE + host_id.raw() * HOST_WIDGET_STRIDE)
}

pub fn workspace_split_host_widget_id(host_id: PanelHostId) -> WidgetId {
    match host_id.raw() {
        1 => BODY_CONSOLE_SPLIT_WIDGET_ID,
        2 => LEFT_RIGHT_SPLIT_WIDGET_ID,
        3 => CENTER_RIGHT_SPLIT_WIDGET_ID,
        _ => WidgetId(WORKSPACE_SPLIT_HOST_WIDGET_ID_BASE + host_id.raw() * HOST_WIDGET_STRIDE),
    }
}

pub fn workspace_split_handle_widget_id(host_id: PanelHostId) -> WidgetId {
    match host_id.raw() {
        1 => BODY_CONSOLE_SPLIT_HANDLE_WIDGET_ID,
        2 => LEFT_RIGHT_SPLIT_HANDLE_WIDGET_ID,
        3 => CENTER_RIGHT_SPLIT_HANDLE_WIDGET_ID,
        _ => WidgetId(WORKSPACE_SPLIT_HANDLE_WIDGET_ID_BASE + host_id.raw() * HOST_WIDGET_STRIDE),
    }
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
