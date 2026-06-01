//! File: domain/ui/ui_hosts/src/lib.rs
//! Crate: ui_hosts

use serde::{Deserialize, Serialize};
use ui_program::UiEventPacket;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostKind {
    Editor,
    Game,
    WorldSpace,
    Headless,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostCommand {
    pub host: HostKind,
    pub command_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainCommand {
    pub domain_id: String,
    pub command_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostRouteMapping {
    pub route_id: String,
    pub host_command: HostCommand,
    pub domain_command: Option<DomainCommand>,
}

pub trait UiHost {
    fn kind(&self) -> HostKind;
    fn map_event(&self, packet: &UiEventPacket) -> Option<HostRouteMapping>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticRouteHost {
    pub kind: HostKind,
    pub mappings: Vec<HostRouteMapping>,
}

impl UiHost for StaticRouteHost {
    fn kind(&self) -> HostKind {
        self.kind.clone()
    }

    fn map_event(&self, packet: &UiEventPacket) -> Option<HostRouteMapping> {
        self.mappings
            .iter()
            .find(|mapping| mapping.route_id == packet.route.as_str())
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_program::{RouteId, RouteSchemaVersion, UiEventPacket};
    use ui_schema::{UiSchemaRef, UiSchemaValue};

    #[test]
    fn host_contract_maps_ui_events_without_owning_ui_semantics() {
        let host = StaticRouteHost {
            kind: HostKind::Editor,
            mappings: vec![HostRouteMapping {
                route_id: "editor.open".to_owned(),
                host_command: HostCommand {
                    host: HostKind::Editor,
                    command_id: "open_panel".to_owned(),
                },
                domain_command: None,
            }],
        };
        let packet = UiEventPacket::new(
            RouteId::new("editor.open"),
            RouteSchemaVersion::new(1),
            UiSchemaRef::new("ui.empty", 1),
            UiSchemaValue::object([]),
        );

        let mapped = host.map_event(&packet).expect("route should map");

        assert_eq!(host.kind(), HostKind::Editor);
        assert_eq!(mapped.host_command.command_id, "open_panel");
    }
}
