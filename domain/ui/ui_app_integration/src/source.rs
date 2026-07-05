//! Minimal code-authored source helpers for the proof bridge.

use serde::{Deserialize, Serialize};
use ui_definition::{AuthoredId, UiNodeDefinition, UiRouteSlotRef, UiValueBinding};
use ui_program::RouteId;

use crate::ids::UiAppScreenId;
use crate::screen::{UiAppScreenDescriptor, UiAppScreenRoute};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAppSourceNodeRef {
    pub node_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAppSourceRouteRef {
    pub slot_id: String,
    pub route: RouteId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAppSourceBuildReport {
    pub screen_id: UiAppScreenId,
    pub root_node_id: String,
    pub nodes: Vec<UiAppSourceNodeRef>,
    pub routes: Vec<UiAppSourceRouteRef>,
    pub root: UiNodeDefinition,
}

impl UiAppSourceBuildReport {
    pub fn node_ids(&self) -> Vec<&str> {
        self.nodes.iter().map(|node| node.node_id.as_str()).collect()
    }
}

#[derive(Clone, Debug, Default)]
pub struct UiAppSourceBuilder;

impl UiAppSourceBuilder {
    pub fn counter_screen(count: u32) -> UiAppSourceBuildReport {
        let screen_id = UiAppScreenId::new("counter.screen");
        let route = RouteId::new("counter.increment");
        let root_id = AuthoredId::new("counter.root");
        let label_id = AuthoredId::new("counter.count_label");
        let button_id = AuthoredId::new("counter.increment_button");
        let route_slot_id = AuthoredId::new("counter.increment");
        let route_slot = UiRouteSlotRef { id: route_slot_id.clone() };

        let root = UiNodeDefinition::Column {
            id: root_id.clone(),
            children: vec![
                UiNodeDefinition::Label {
                    id: label_id.clone(),
                    label: UiValueBinding::static_text(format!("Clicked {count} / 5")),
                    availability: None,
                },
                UiNodeDefinition::Button {
                    id: button_id.clone(),
                    label: UiValueBinding::static_text("Click me"),
                    route: Some(route_slot.clone()),
                    availability: None,
                    selected: None,
                },
            ],
        };

        UiAppSourceBuildReport {
            screen_id,
            root_node_id: root_id.as_str().to_owned(),
            nodes: vec![
                UiAppSourceNodeRef { node_id: root_id.as_str().to_owned() },
                UiAppSourceNodeRef { node_id: label_id.as_str().to_owned() },
                UiAppSourceNodeRef { node_id: button_id.as_str().to_owned() },
            ],
            routes: vec![UiAppSourceRouteRef {
                slot_id: route_slot_id.as_str().to_owned(),
                route,
            }],
            root,
        }
    }

    pub fn win_screen() -> UiAppSourceBuildReport {
        let screen_id = UiAppScreenId::new("counter.win");
        let route = RouteId::new("counter.reset");
        let root_id = AuthoredId::new("counter.win_root");
        let label_id = AuthoredId::new("counter.win_label");
        let button_id = AuthoredId::new("counter.reset_button");
        let route_slot_id = AuthoredId::new("counter.reset");
        let route_slot = UiRouteSlotRef { id: route_slot_id.clone() };

        let root = UiNodeDefinition::Column {
            id: root_id.clone(),
            children: vec![
                UiNodeDefinition::Label {
                    id: label_id.clone(),
                    label: UiValueBinding::static_text("You win!"),
                    availability: None,
                },
                UiNodeDefinition::Button {
                    id: button_id.clone(),
                    label: UiValueBinding::static_text("Reset"),
                    route: Some(route_slot.clone()),
                    availability: None,
                    selected: None,
                },
            ],
        };

        UiAppSourceBuildReport {
            screen_id,
            root_node_id: root_id.as_str().to_owned(),
            nodes: vec![
                UiAppSourceNodeRef { node_id: root_id.as_str().to_owned() },
                UiAppSourceNodeRef { node_id: label_id.as_str().to_owned() },
                UiAppSourceNodeRef { node_id: button_id.as_str().to_owned() },
            ],
            routes: vec![UiAppSourceRouteRef {
                slot_id: route_slot_id.as_str().to_owned(),
                route,
            }],
            root,
        }
    }

    pub fn descriptor(report: &UiAppSourceBuildReport) -> UiAppScreenDescriptor {
        report.routes.iter().fold(
            UiAppScreenDescriptor::new(report.screen_id.clone(), report.root.clone()),
            |descriptor, route| {
                descriptor.with_route(UiAppScreenRoute {
                    slot: UiRouteSlotRef { id: AuthoredId::new(route.slot_id.clone()) },
                    route: route.route.clone(),
                })
            },
        )
    }
}
