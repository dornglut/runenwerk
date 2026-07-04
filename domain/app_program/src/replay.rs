//! Deterministic app-program replay traces.

use crate::AppProgramId;
use crate::effect::AppEffectPlan;
use crate::model::AppModelSnapshot;
use crate::projection::AppViewProjectionReport;
use crate::reducer::{AppReducerInput, AppReducerOutcome};
use crate::report::{
    AppDiagnostic, AppProgramReport, CounterEffectPlanReport, CounterReducerTraceReport,
    CounterViewProjectionReport, NAMESPACE_EFFECT_PLAN, NAMESPACE_REPLAY, NAMESPACE_REPORT_BUDGET,
};
use crate::route_action::{RouteActionMap, RouteActionRequest, RouteActionResolution};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppReplayScenario {
    pub scenario_id: String,
    pub program_id: AppProgramId,
    pub initial_model: AppModelSnapshot,
    pub events: Vec<RouteActionRequest>,
}

impl AppReplayScenario {
    pub fn new(
        scenario_id: impl Into<String>,
        program_id: AppProgramId,
        initial_model: AppModelSnapshot,
    ) -> Self {
        Self {
            scenario_id: scenario_id.into(),
            program_id,
            initial_model,
            events: Vec::new(),
        }
    }

    pub fn with_event(mut self, event: RouteActionRequest) -> Self {
        self.events.push(event);
        self
    }

    pub fn run<R, P>(&self, route_map: &RouteActionMap, reducer: R, projection: P) -> AppReplayTrace
    where
        R: Fn(AppReducerInput) -> AppReducerOutcome,
        P: Fn(&AppModelSnapshot) -> AppViewProjectionReport,
    {
        let mut current_model = self.initial_model.clone();
        let mut steps = Vec::new();
        let mut trace_diagnostics = Vec::new();

        for (step_index, event) in self.events.iter().cloned().enumerate() {
            let model_before = current_model.clone();
            let projection_before = projection(&model_before);
            let route_resolution = route_map.resolve(&event);
            let mut step_diagnostics = Vec::new();

            if projection_before
                .diagnostics
                .iter()
                .any(AppDiagnostic::is_error)
            {
                step_diagnostics.push(AppDiagnostic::new(
                    NAMESPACE_REPLAY,
                    "app.replay.projection_rejected_step",
                    format!("step {step_index} stopped before reducer because projection failed"),
                ));
            }

            let mut reducer_outcome = None;
            let mut effect_plan = AppEffectPlan::NoEffect;
            let mut model_after = model_before.clone();

            if projection_before.passed() {
                if let Some(action) = route_resolution.action.clone() {
                    let input = AppReducerInput::new(model_before.clone(), action);
                    let outcome = reducer(input);
                    effect_plan = outcome.effect_plan.clone();
                    if outcome.accepted {
                        model_after = outcome.after_model.clone();
                        current_model = model_after.clone();
                    } else {
                        step_diagnostics.push(AppDiagnostic::new(
                            NAMESPACE_REPLAY,
                            "app.replay.reducer_rejected_step",
                            format!("step {step_index} preserved model because reducer rejected"),
                        ));
                    }
                    reducer_outcome = Some(outcome);
                } else {
                    step_diagnostics.push(AppDiagnostic::new(
                        NAMESPACE_REPLAY,
                        "app.replay.route_rejected_step",
                        format!(
                            "step {step_index} preserved model because route resolution was {}",
                            route_resolution.status.as_str()
                        ),
                    ));
                }
            }

            let projection_after = if projection_before.passed() {
                projection(&model_after)
            } else {
                projection_before.clone()
            };

            if projection_after
                .diagnostics
                .iter()
                .any(AppDiagnostic::is_error)
            {
                step_diagnostics.push(AppDiagnostic::new(
                    NAMESPACE_REPLAY,
                    "app.replay.after_projection_failed",
                    format!("step {step_index} after-model projection produced diagnostics"),
                ));
            }

            steps.push(AppReplayStep {
                step_index,
                model_before,
                projection_before,
                route_event: event,
                route_resolution,
                reducer_outcome,
                effect_plan,
                model_after,
                projection_after,
                diagnostics: step_diagnostics,
            });
        }

        if steps.iter().any(|step| step.has_error()) {
            trace_diagnostics.push(AppDiagnostic::new(
                NAMESPACE_REPLAY,
                "app.replay.failed",
                "one or more replay steps failed closed",
            ));
        }

        AppReplayTrace {
            scenario_id: self.scenario_id.clone(),
            program_id: self.program_id.clone(),
            initial_model: self.initial_model.clone(),
            steps,
            final_model: current_model,
            diagnostics: trace_diagnostics,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppReplayStep {
    pub step_index: usize,
    pub model_before: AppModelSnapshot,
    pub projection_before: AppViewProjectionReport,
    pub route_event: RouteActionRequest,
    pub route_resolution: RouteActionResolution,
    pub reducer_outcome: Option<AppReducerOutcome>,
    pub effect_plan: AppEffectPlan,
    pub model_after: AppModelSnapshot,
    pub projection_after: AppViewProjectionReport,
    pub diagnostics: Vec<AppDiagnostic>,
}

impl AppReplayStep {
    pub fn has_error(&self) -> bool {
        self.diagnostics.iter().any(AppDiagnostic::is_error)
            || self
                .projection_before
                .diagnostics
                .iter()
                .any(AppDiagnostic::is_error)
            || self
                .projection_after
                .diagnostics
                .iter()
                .any(AppDiagnostic::is_error)
            || self
                .route_resolution
                .diagnostics
                .iter()
                .any(AppDiagnostic::is_error)
            || self
                .reducer_outcome
                .as_ref()
                .map(|outcome| outcome.diagnostics.iter().any(AppDiagnostic::is_error))
                .unwrap_or(false)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppReplayTrace {
    pub scenario_id: String,
    pub program_id: AppProgramId,
    pub initial_model: AppModelSnapshot,
    pub steps: Vec<AppReplayStep>,
    pub final_model: AppModelSnapshot,
    pub diagnostics: Vec<AppDiagnostic>,
}

impl AppReplayTrace {
    pub fn passed(&self) -> bool {
        !self.diagnostics.iter().any(AppDiagnostic::is_error)
            && self.steps.iter().all(|step| !step.has_error())
    }

    pub fn to_report(&self) -> AppProgramReport {
        let mut route_reports = Vec::new();
        let mut reducer_reports = Vec::new();
        let mut projection_reports = Vec::new();
        let mut effect_reports = Vec::new();

        for step in &self.steps {
            route_reports.push(step.route_resolution.report());

            if let Some(outcome) = &step.reducer_outcome {
                reducer_reports.push(CounterReducerTraceReport {
                    step_index: step.step_index,
                    accepted: outcome.accepted,
                    action_id: Some(outcome.action.action_id.as_str().to_owned()),
                    before_revision: outcome.before_model.revision.value(),
                    after_revision: outcome.after_model.revision.value(),
                    count_before: outcome.before_model.integer("counter.count"),
                    count_after: outcome.after_model.integer("counter.count"),
                    diagnostics: outcome.diagnostics.clone(),
                });
            } else {
                reducer_reports.push(CounterReducerTraceReport {
                    step_index: step.step_index,
                    accepted: false,
                    action_id: None,
                    before_revision: step.model_before.revision.value(),
                    after_revision: step.model_after.revision.value(),
                    count_before: step.model_before.integer("counter.count"),
                    count_after: step.model_after.integer("counter.count"),
                    diagnostics: vec![AppDiagnostic::info(
                        NAMESPACE_REPLAY,
                        "app.replay.reducer_not_run",
                        "reducer was not run for this failed-closed step",
                    )],
                });
            }

            projection_reports.push(CounterViewProjectionReport {
                step_index: step.step_index,
                model_revision: step.projection_after.source_model_revision.value(),
                screen_id: step
                    .projection_after
                    .projection
                    .as_ref()
                    .map(|projection| projection.screen_id.clone()),
                route_ids: step
                    .projection_after
                    .projection
                    .as_ref()
                    .map(|projection| projection.route_ids.clone())
                    .unwrap_or_default(),
                diagnostics: step.projection_after.diagnostics.clone(),
            });

            effect_reports.push(CounterEffectPlanReport {
                step_index: step.step_index,
                plan_kind: step.effect_plan.kind().to_owned(),
                effect_count: step.effect_plan.effect_count(),
                diagnostics: vec![AppDiagnostic::info(
                    NAMESPACE_EFFECT_PLAN,
                    "app.effect.plan.no_effect",
                    "counter proof does not request host or domain effects",
                )],
            });
        }

        let mut diagnostics = self.diagnostics.clone();
        diagnostics.push(AppDiagnostic::info(
            NAMESPACE_REPORT_BUDGET,
            "app.report.budget.within_limit",
            "app-program report uses bounded payload summaries",
        ));

        AppProgramReport {
            scenario_id: self.scenario_id.clone(),
            program_id: self.program_id.as_str().to_owned(),
            passed: self.passed(),
            initial_revision: self.initial_model.revision.value(),
            final_revision: self.final_model.revision.value(),
            route_reports,
            reducer_reports,
            projection_reports,
            effect_reports,
            diagnostics,
        }
    }
}
