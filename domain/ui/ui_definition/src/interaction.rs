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
    pub menu_scopes: Vec<FormedMenuStackScope>,
    pub scroll_owners: Vec<FormedScrollOwner>,
}

impl FormedInteractionModel {
    pub fn extend(&mut self, other: FormedInteractionModel) {
        self.menu_scopes.extend(other.menu_scopes);
        self.scroll_owners.extend(other.scroll_owners);
    }

    pub fn push_menu_scope(&mut self, scope: FormedMenuStackScope) {
        self.menu_scopes.push(scope);
    }

    pub fn push_scroll_owner(&mut self, owner: FormedScrollOwner) {
        self.scroll_owners.push(owner);
    }
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
pub struct FormedScrollOwner {
    pub widget_id: WidgetId,
    pub axes: Vec<Axis>,
    pub boundary: UiScrollBoundaryPolicyDefinition,
}
