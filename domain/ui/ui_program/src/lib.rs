//! File: domain/ui/ui_program/src/lib.rs
//! Crate: ui_program

pub mod events;
pub mod graphs;
pub mod program;

pub use events::*;
pub use graphs::*;
pub use program::*;

#[cfg(test)]
mod tests {
    use super::*;
    use ui_schema::{UiSchemaRef, UiSchemaValue};

    #[test]
    fn route_contract_uses_stable_ids_and_schema_payloads() {
        let route = RouteId::new("editor.color.apply");
        let packet = UiEventPacket::new(
            route.clone(),
            RouteSchemaVersion::new(1),
            UiSchemaRef::new("ui.color.rgba", 1),
            UiSchemaValue::object([
                ("r", UiSchemaValue::number(0.1)),
                ("g", UiSchemaValue::number(0.2)),
                ("b", UiSchemaValue::number(0.3)),
                ("a", UiSchemaValue::number(1.0)),
            ]),
        )
        .with_capability(RouteCapability::new("editor.color.write"));

        assert_eq!(packet.route, route);
        assert_eq!(packet.schema_version.value(), 1);
        assert_eq!(packet.capabilities[0].as_str(), "editor.color.write");
        assert_eq!(
            packet.payload.value.get("a"),
            Some(&UiSchemaValue::number(1.0))
        );
    }

    #[test]
    fn architecture_contract_exposes_named_graph_families() {
        let program = UiProgram::new(
            UiProgramId::new("editor.inspector"),
            UiProgramVersion::new(1),
        );

        assert_eq!(program.id.as_str(), "editor.inspector");
        assert!(program.graphs.control.nodes.is_empty());
        assert!(program.graphs.layout.constraints.is_empty());
        assert!(program.graphs.state.requirements.is_empty());
        assert!(program.graphs.style.rules.is_empty());
        assert!(program.graphs.interaction.handlers.is_empty());
        assert!(program.graphs.binding.bindings.is_empty());
        assert!(program.graphs.visual.operators.is_empty());
        assert!(program.graphs.accessibility.nodes.is_empty());
        assert!(program.graphs.inspection.entries.is_empty());
    }
}
