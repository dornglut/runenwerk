//! Authored UI node definitions.

use crate::{
    availability::UiAvailabilityBinding,
    embed::UiEmbedSlotRef,
    identity::{UiNodeId, UiTemplateId},
    slot::{UiCollectionSlotRef, UiMenuSlotRef, UiRouteSlotRef, UiSelectionSlotRef},
    value::UiValueBinding,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiAxisDefinition {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UiNodeDefinition {
    Panel {
        id: UiNodeId,
        children: Vec<UiNodeDefinition>,
        availability: Option<UiAvailabilityBinding>,
    },
    Row {
        id: UiNodeId,
        children: Vec<UiNodeDefinition>,
    },
    Column {
        id: UiNodeId,
        children: Vec<UiNodeDefinition>,
    },
    Stack {
        id: UiNodeId,
        axis: UiAxisDefinition,
        children: Vec<UiNodeDefinition>,
    },
    Scroll {
        id: UiNodeId,
        axis: UiAxisDefinition,
        children: Vec<UiNodeDefinition>,
    },
    Split {
        id: UiNodeId,
        axis: UiAxisDefinition,
        ratio: f32,
        children: Vec<UiNodeDefinition>,
    },
    Spacer {
        id: UiNodeId,
    },
    Separator {
        id: UiNodeId,
    },
    Label {
        id: UiNodeId,
        label: UiValueBinding,
    },
    Button {
        id: UiNodeId,
        label: UiValueBinding,
        route: Option<UiRouteSlotRef>,
        availability: Option<UiAvailabilityBinding>,
        selected: Option<UiValueBinding>,
    },
    Toggle {
        id: UiNodeId,
        label: UiValueBinding,
        checked: UiValueBinding,
        route: Option<UiRouteSlotRef>,
        availability: Option<UiAvailabilityBinding>,
    },
    TextInput {
        id: UiNodeId,
        value: UiValueBinding,
        placeholder: Option<String>,
        route: Option<UiRouteSlotRef>,
        availability: Option<UiAvailabilityBinding>,
    },
    NumericInput {
        id: UiNodeId,
        value: UiValueBinding,
        route: Option<UiRouteSlotRef>,
        availability: Option<UiAvailabilityBinding>,
    },
    Select {
        id: UiNodeId,
        items: UiCollectionSlotRef,
        selected: Option<UiSelectionSlotRef>,
        route: Option<UiRouteSlotRef>,
        availability: Option<UiAvailabilityBinding>,
    },
    Tabs {
        id: UiNodeId,
        items: UiCollectionSlotRef,
        selected: Option<UiSelectionSlotRef>,
        route: Option<UiRouteSlotRef>,
    },
    Table {
        id: UiNodeId,
        rows: UiCollectionSlotRef,
        route: Option<UiRouteSlotRef>,
    },
    Tree {
        id: UiNodeId,
        rows: UiCollectionSlotRef,
        route: Option<UiRouteSlotRef>,
    },
    Repeat {
        id: UiNodeId,
        items: UiCollectionSlotRef,
        template: UiTemplateId,
        #[serde(default)]
        axis: Option<UiAxisDefinition>,
    },
    TemplateRef {
        id: UiNodeId,
        template: UiTemplateId,
    },
    MenuSlot {
        id: UiNodeId,
        menu: UiMenuSlotRef,
    },
    EmbedSlot {
        id: UiNodeId,
        slot: UiEmbedSlotRef,
    },
}

impl UiNodeDefinition {
    pub fn id(&self) -> &UiNodeId {
        match self {
            Self::Panel { id, .. }
            | Self::Row { id, .. }
            | Self::Column { id, .. }
            | Self::Stack { id, .. }
            | Self::Scroll { id, .. }
            | Self::Split { id, .. }
            | Self::Spacer { id }
            | Self::Separator { id }
            | Self::Label { id, .. }
            | Self::Button { id, .. }
            | Self::Toggle { id, .. }
            | Self::TextInput { id, .. }
            | Self::NumericInput { id, .. }
            | Self::Select { id, .. }
            | Self::Tabs { id, .. }
            | Self::Table { id, .. }
            | Self::Tree { id, .. }
            | Self::Repeat { id, .. }
            | Self::TemplateRef { id, .. }
            | Self::MenuSlot { id, .. }
            | Self::EmbedSlot { id, .. } => id,
        }
    }

    pub fn children(&self) -> &[UiNodeDefinition] {
        match self {
            Self::Panel { children, .. }
            | Self::Row { children, .. }
            | Self::Column { children, .. }
            | Self::Stack { children, .. }
            | Self::Scroll { children, .. }
            | Self::Split { children, .. } => children,
            _ => &[],
        }
    }
}
