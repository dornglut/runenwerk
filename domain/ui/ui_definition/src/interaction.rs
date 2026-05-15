//! Interaction V2 formation contracts.

use crate::UiNodeId;
use serde::{Deserialize, Serialize};
use ui_math::Axis;
use ui_tree::WidgetId;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiScrollBoundaryPolicyDefinition {
    #[default]
    ConsumeAtBoundary,
    PropagateAtBoundary,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiScrollOwnershipDefinition {
    #[serde(default)]
    pub boundary: UiScrollBoundaryPolicyDefinition,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiMenuDismissPolicyDefinition {
    #[default]
    OutsidePointerDown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiMenuFocusReturnDefinition {
    #[default]
    Anchor,
    Widget(UiNodeId),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiMenuStackScopeDefinition {
    pub id: String,
    pub anchor: UiNodeId,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub dismiss: UiMenuDismissPolicyDefinition,
    #[serde(default)]
    pub focus_return: UiMenuFocusReturnDefinition,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FormedInteractionModel {
    pub chrome_slots: Vec<FormedChromeSlot>,
    pub menu_scopes: Vec<FormedMenuStackScope>,
    pub menu_sizing: Vec<FormedMenuSizing>,
    pub scroll_owners: Vec<FormedScrollOwner>,
}

impl FormedInteractionModel {
    pub fn extend(&mut self, other: FormedInteractionModel) {
        self.chrome_slots.extend(other.chrome_slots);
        self.menu_scopes.extend(other.menu_scopes);
        self.menu_sizing.extend(other.menu_sizing);
        self.scroll_owners.extend(other.scroll_owners);
    }

    pub fn push_chrome_slot(&mut self, slot: FormedChromeSlot) {
        self.chrome_slots.push(slot);
    }

    pub fn push_menu_scope(&mut self, scope: FormedMenuStackScope) {
        self.menu_scopes.push(scope);
    }

    pub fn push_menu_sizing(&mut self, sizing: FormedMenuSizing) {
        self.menu_sizing.push(sizing);
    }

    pub fn push_scroll_owner(&mut self, owner: FormedScrollOwner) {
        self.scroll_owners.push(owner);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormedChromeSlot {
    pub host_widget_id: WidgetId,
    pub slot_widget_id: WidgetId,
    pub kind: UiChromeSlotKindDefinition,
    pub input_policy: UiChromeSlotInputPolicyDefinition,
    pub order: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiChromeSlotKindDefinition {
    CloseAffordance,
    ActiveIndicator,
    Label,
    CommandArea,
    DragRegion,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiChromeSlotInputPolicyDefinition {
    #[default]
    None,
    Activate,
    Command,
    Drag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormedMenuStackScope {
    pub scope_id: String,
    pub popup_widget_id: WidgetId,
    pub anchor_widget_id: WidgetId,
    pub parent_scope_id: Option<String>,
    pub dismiss: UiMenuDismissPolicyDefinition,
    pub focus_return: Option<WidgetId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormedMenuSizing {
    pub popup_widget_id: WidgetId,
    pub list_widget_id: WidgetId,
    pub item_width: UiMenuItemWidthDefinition,
    pub overflow: UiMenuOverflowDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormedScrollOwner {
    pub widget_id: WidgetId,
    pub axes: Vec<Axis>,
    pub boundary: UiScrollBoundaryPolicyDefinition,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiMenuItemWidthDefinition {
    #[default]
    FillToMenuWidth,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiMenuOverflowDefinition {
    #[default]
    ScrollWhenClamped,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiMenuSizingDefinition {
    #[serde(default)]
    pub item_width: UiMenuItemWidthDefinition,
    #[serde(default)]
    pub overflow: UiMenuOverflowDefinition,
}
