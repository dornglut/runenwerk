//! Proof orchestration for the ECS-backed counter UI integration proof.

use ui_controls::{ControlPackageRegistry, runenwerk_control_package};
use ui_program::{RouteCapability, RouteId, RouteSchemaVersion, UiEventPacket};
use ui_program_lowering::form_ui_program_report_from_node_with_registry_snapshot;
use ui_schema::{UiSchemaRef, UiSchemaValue};

use crate::bridge::{UiAppRouteBinding, UiAppRouteBridge};
use crate::host::CounterHost;
use crate::ids::{UiAppActionId, UiAppProofId, UiAppRouteBindingId};
use crate::report::{
    UiAppActionReport, UiAppFormationReportSummary, UiAppIntegrationReport,
    UiAppIntegrationStepReport, UiAppProofDiagnostic, UiAppRuntimeReportSummary, UiAppSourceReport,
};
use crate::source::{UiAppSourceBuildReport, UiAppSourceBuilder};

#[derive(Clone, Debug)]
pub struct UiAppIntegrationProof {
    proof_id: UiAppProofId,
}

impl UiAppIntegrationProof {
    pub fn builder(proof_id: UiAppProofId) -> UiAppIntegrationProofBuilder {
        UiAppIntegrationProofBuilder { proof_id }
    }
}

#[derive(Clone, Debug)]
pub struct UiAppIntegrationProofBuilder {
    proof_id: UiAppProofId,
}

impl UiAppIntegrationProofBuilder {
    pub fn build(self) -> UiAppIntegrationProof {
        UiAppIntegrationProof {
            proof_id: self.proof_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiAppIntegrationProofRun {
    pub report: UiAppIntegrationReport,
}

impl UiAppIntegrationProof {
    pub fn run_counter_story(&self) -> UiAppIntegrationProofRun {
        let mut host = CounterHost::new(0);
        let initial = host.snapshot();
        let mut steps = Vec::new();
        let mut diagnostics = Vec::new();
        let bridge = counter_route_bridge();

        for index in 0..5 {
            let count = host.count();
            let packet = counter_packet("counter.increment", "counter.action.increment");
            let source = UiAppSourceBuilder::counter_screen(count);
            let step = run_step(
                format!("increment.{index}"),
                &mut host,
                source,
                &bridge,
                packet,
            );
            if !step.passed() {
                diagnostics.extend(step.diagnostics.clone());
            }
            steps.push(step);
        }

        let reset = run_step(
            "reset".to_string(),
            &mut host,
            UiAppSourceBuilder::win_screen(),
            &bridge,
            counter_packet("counter.reset", "counter.action.reset"),
        );
        if !reset.passed() {
            diagnostics.extend(reset.diagnostics.clone());
        }
        steps.push(reset);

        let report = UiAppIntegrationReport {
            proof_id: self.proof_id.clone(),
            initial,
            final_snapshot: host.snapshot(),
            steps,
            diagnostics,
        };

        UiAppIntegrationProofRun { report }
    }
}

fn run_step(
    step: String,
    host: &mut CounterHost,
    source: UiAppSourceBuildReport,
    bridge: &UiAppRouteBridge,
    packet: UiEventPacket,
) -> UiAppIntegrationStepReport {
    let before = host.snapshot();
    let registry = ControlPackageRegistry::new()
        .with_package(runenwerk_control_package())
        .expect("runenwerk control package must register for proof");
    let snapshot = registry.snapshot();
    let formation_report = form_ui_program_report_from_node_with_registry_snapshot(
        format!("{}.program", source.screen_id.as_str()),
        format!("{}.source", source.screen_id.as_str()),
        &source.root,
        &snapshot,
    );

    let mut diagnostics = Vec::new();
    if !formation_report.passed() {
        diagnostics.push(UiAppProofDiagnostic::FormationFailed {
            screen: source.screen_id.as_str().to_owned(),
            diagnostic_count: formation_report.diagnostics.len(),
        });
    }

    let resolution = bridge.resolve(&packet);
    let action = resolution.action.clone().map(|resolved| UiAppActionReport {
        action_id: resolved.action_id.clone(),
        route: resolved.route.as_str().to_owned(),
        resolved: true,
    });

    for diagnostic in &resolution.diagnostics {
        diagnostics.push(UiAppProofDiagnostic::RouteRejected {
            diagnostic: diagnostic.clone(),
        });
    }

    let mutation = resolution
        .action
        .as_ref()
        .and_then(|resolved| host.apply_resolved_action(&resolved.action_id));

    if resolution.is_resolved() && mutation.is_none() {
        if let Some(action) = action.as_ref() {
            diagnostics.push(UiAppProofDiagnostic::MutationMissing {
                action: action.action_id.as_str().to_owned(),
            });
        }
    }

    let after = host.snapshot();
    UiAppIntegrationStepReport {
        step,
        before,
        source: UiAppSourceReport { source: source.clone() },
        formation: UiAppFormationReportSummary {
            screen_id: source.screen_id.clone(),
            passed: formation_report.passed(),
            diagnostics: formation_report.diagnostics.len(),
            source_map_entries: formation_report.program.source_map.len(),
        },
        runtime: UiAppRuntimeReportSummary {
            screen_id: source.screen_id.clone(),
            output_contains_text: text_facts(&source),
            route_ids: source
                .routes
                .iter()
                .map(|route| route.route.as_str().to_owned())
                .collect(),
        },
        action,
        mutation,
        after,
        diagnostics,
    }
}

fn text_facts(source: &UiAppSourceBuildReport) -> Vec<String> {
    source
        .nodes
        .iter()
        .map(|node| node.node_id.clone())
        .collect()
}

pub fn counter_payload_schema() -> UiSchemaRef {
    UiSchemaRef::new("counter.action.payload", 1)
}

pub fn counter_packet(route: &str, capability: &str) -> UiEventPacket {
    UiEventPacket::new(
        RouteId::new(route),
        RouteSchemaVersion::new(1),
        counter_payload_schema(),
        UiSchemaValue::null(),
    )
    .with_capability(RouteCapability::new(capability))
}

pub fn counter_route_bridge() -> UiAppRouteBridge {
    UiAppRouteBridge::new()
        .with_binding(UiAppRouteBinding::new(
            UiAppRouteBindingId::new("counter.increment.binding"),
            RouteId::new("counter.increment"),
            RouteSchemaVersion::new(1),
            counter_payload_schema(),
            UiAppActionId::new("counter.increment"),
            RouteCapability::new("counter.action.increment"),
        ))
        .with_binding(UiAppRouteBinding::new(
            UiAppRouteBindingId::new("counter.reset.binding"),
            RouteId::new("counter.reset"),
            RouteSchemaVersion::new(1),
            counter_payload_schema(),
            UiAppActionId::new("counter.reset"),
            RouteCapability::new("counter.action.reset"),
        ))
}
