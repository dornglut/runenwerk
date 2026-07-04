//! Demo-owned counter app proof semantics.

use crate::action::{
    AppAction, AppActionCapability, AppActionPayload, AppActionPayloadShape, AppActionSource,
};
use crate::effect::AppEffectPlan;
use crate::ids::{
    AppActionId, AppActionVersion, AppModelId, AppModelRevision, AppModelVersion, AppProgramId,
    AppProgramVersion,
};
use crate::model::{AppModelSnapshot, AppModelValue};
use crate::projection::{AppViewProjection, AppViewProjectionReport};
use crate::reducer::{AppReducerInput, AppReducerOutcome};
use crate::replay::AppReplayScenario;
use crate::report::{
    AppDiagnostic, NAMESPACE_MODEL_SCHEMA, NAMESPACE_PROJECTION, NAMESPACE_REDUCER,
    NAMESPACE_VERSION_COMPATIBILITY,
};
use crate::route_action::{RouteActionMap, RouteActionMapping, RouteActionRequest};

pub const COUNTER_PROGRAM_ID: &str = "runenwerk.proofs.counter_app";
pub const COUNTER_MODEL_ID: &str = "counter.model";
pub const COUNTER_INCREMENT_ROUTE: &str = "counter.increment";
pub const COUNTER_RESET_ROUTE: &str = "counter.reset";
pub const COUNTER_INCREMENT_ACTION_ID: &str = "counter.action.increment";
pub const COUNTER_RESET_ACTION_ID: &str = "counter.action.reset";
pub const COUNTER_CAPABILITY: &str = "counter.action.write";
pub const COUNTER_ROUTE_SCHEMA_VERSION: u32 = 1;
pub const COUNTER_WIN_THRESHOLD: i64 = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CounterModel {
    pub count: i64,
}

impl CounterModel {
    pub fn initial() -> Self {
        Self { count: 0 }
    }

    pub fn screen(&self) -> CounterScreen {
        if self.count >= COUNTER_WIN_THRESHOLD {
            CounterScreen::Win
        } else {
            CounterScreen::Counter
        }
    }

    pub fn to_snapshot(&self, revision: AppModelRevision) -> AppModelSnapshot {
        AppModelSnapshot::new(counter_model_id(), AppModelVersion::new(1), revision)
            .with_value("counter.count", AppModelValue::Integer(self.count))
            .with_value(
                "counter.screen",
                AppModelValue::String(self.screen().screen_id().to_owned()),
            )
    }

    pub fn from_snapshot(snapshot: &AppModelSnapshot) -> Result<Self, Vec<AppDiagnostic>> {
        let mut diagnostics = Vec::new();
        if snapshot.model_id != counter_model_id() {
            diagnostics.push(AppDiagnostic::new(
                NAMESPACE_MODEL_SCHEMA,
                "app.model.schema.unexpected_model_id",
                format!(
                    "counter reducer expected model {}, got {}",
                    COUNTER_MODEL_ID, snapshot.model_id
                ),
            ));
        }
        if snapshot.model_version != AppModelVersion::new(1) {
            diagnostics.push(AppDiagnostic::new(
                NAMESPACE_VERSION_COMPATIBILITY,
                "app.version.compatibility.model_version",
                format!(
                    "counter model version {} is not supported",
                    snapshot.model_version.value()
                ),
            ));
        }
        let Some(count) = snapshot.integer("counter.count") else {
            diagnostics.push(AppDiagnostic::new(
                NAMESPACE_MODEL_SCHEMA,
                "app.model.schema.counter_count_missing",
                "counter model snapshot is missing integer field counter.count",
            ));
            return Err(diagnostics);
        };
        if count < 0 {
            diagnostics.push(AppDiagnostic::new(
                NAMESPACE_MODEL_SCHEMA,
                "app.model.schema.counter_count_negative",
                "counter count must not be negative",
            ));
        }
        if diagnostics.is_empty() {
            Ok(Self { count })
        } else {
            Err(diagnostics)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterAction {
    Increment,
    Reset,
}

impl CounterAction {
    pub fn from_app_action(action: &AppAction) -> Result<Self, AppDiagnostic> {
        if action.action_version != AppActionVersion::new(1) {
            return Err(AppDiagnostic::new(
                NAMESPACE_VERSION_COMPATIBILITY,
                "app.version.compatibility.action_version",
                format!(
                    "counter action version {} is not supported",
                    action.action_version.value()
                ),
            ));
        }
        if action.payload != AppActionPayload::Unit {
            return Err(AppDiagnostic::new(
                NAMESPACE_REDUCER,
                "app.reducer.action_payload_not_unit",
                "counter reducer accepts only unit action payloads",
            ));
        }
        match action.action_id.as_str() {
            COUNTER_INCREMENT_ACTION_ID => Ok(Self::Increment),
            COUNTER_RESET_ACTION_ID => Ok(Self::Reset),
            other => Err(AppDiagnostic::new(
                NAMESPACE_REDUCER,
                "app.reducer.unknown_counter_action",
                format!("counter reducer does not support action {other}"),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterScreen {
    Counter,
    Win,
}

impl CounterScreen {
    pub fn screen_id(self) -> &'static str {
        match self {
            Self::Counter => "counter.screen.counter",
            Self::Win => "counter.screen.win",
        }
    }
}

pub fn counter_program_id() -> AppProgramId {
    AppProgramId::new(COUNTER_PROGRAM_ID)
}

pub fn counter_program_version() -> AppProgramVersion {
    AppProgramVersion::new(1)
}

pub fn counter_model_id() -> AppModelId {
    AppModelId::new(COUNTER_MODEL_ID)
}

pub fn counter_capability() -> AppActionCapability {
    AppActionCapability::new(COUNTER_CAPABILITY)
}

pub fn counter_initial_snapshot() -> AppModelSnapshot {
    CounterModel::initial().to_snapshot(AppModelRevision::initial())
}

pub fn counter_route_action_map() -> RouteActionMap {
    RouteActionMap::new(counter_program_id(), counter_program_version())
        .with_mapping(
            RouteActionMapping::new(
                COUNTER_INCREMENT_ROUTE,
                COUNTER_ROUTE_SCHEMA_VERSION,
                AppActionId::new(COUNTER_INCREMENT_ACTION_ID),
                AppActionVersion::new(1),
                AppActionPayloadShape::unit(),
            )
            .with_required_capability(counter_capability()),
        )
        .with_mapping(
            RouteActionMapping::new(
                COUNTER_RESET_ROUTE,
                COUNTER_ROUTE_SCHEMA_VERSION,
                AppActionId::new(COUNTER_RESET_ACTION_ID),
                AppActionVersion::new(1),
                AppActionPayloadShape::unit(),
            )
            .with_required_capability(counter_capability()),
        )
}

pub fn counter_increment_event() -> RouteActionRequest {
    RouteActionRequest::new(
        COUNTER_INCREMENT_ROUTE,
        COUNTER_ROUTE_SCHEMA_VERSION,
        AppActionPayload::Unit,
    )
    .with_capability(counter_capability())
    .with_source_control("control.counter.increment")
    .with_source_map("program.counter.interaction.increment")
}

pub fn counter_reset_event() -> RouteActionRequest {
    RouteActionRequest::new(
        COUNTER_RESET_ROUTE,
        COUNTER_ROUTE_SCHEMA_VERSION,
        AppActionPayload::Unit,
    )
    .with_capability(counter_capability())
    .with_source_control("control.counter.reset")
    .with_source_map("program.counter.interaction.reset")
}

pub fn counter_positive_scenario() -> AppReplayScenario {
    let mut scenario = AppReplayScenario::new(
        "counter.scenario.increment_to_win_then_reset",
        counter_program_id(),
        counter_initial_snapshot(),
    );
    for _ in 0..COUNTER_WIN_THRESHOLD {
        scenario = scenario.with_event(counter_increment_event());
    }
    scenario.with_event(counter_reset_event())
}

pub fn counter_projection(snapshot: &AppModelSnapshot) -> AppViewProjectionReport {
    if snapshot.has_error_diagnostics() {
        return AppViewProjectionReport::rejected(
            snapshot.revision,
            vec![AppDiagnostic::new(
                NAMESPACE_PROJECTION,
                "app.projection.source_model_has_diagnostics",
                "projection rejected model snapshot with diagnostics",
            )],
        );
    }

    let model = match CounterModel::from_snapshot(snapshot) {
        Ok(model) => model,
        Err(mut diagnostics) => {
            diagnostics.push(AppDiagnostic::new(
                NAMESPACE_PROJECTION,
                "app.projection.counter_model_invalid",
                "counter projection could not read a valid counter model",
            ));
            return AppViewProjectionReport::rejected(snapshot.revision, diagnostics);
        }
    };

    let screen = model.screen();
    let mut projection = AppViewProjection::new(
        counter_program_id(),
        counter_program_version(),
        snapshot.model_id.clone(),
        snapshot.model_version,
        snapshot.revision,
        screen.screen_id(),
    )
    .with_value("counter.count", AppModelValue::Integer(model.count))
    .with_value(
        "counter.win_threshold",
        AppModelValue::Integer(COUNTER_WIN_THRESHOLD),
    );

    projection = match screen {
        CounterScreen::Counter => projection.with_route(COUNTER_INCREMENT_ROUTE),
        CounterScreen::Win => projection.with_route(COUNTER_RESET_ROUTE),
    };

    AppViewProjectionReport::accepted(projection)
}

pub fn counter_reducer(input: AppReducerInput) -> AppReducerOutcome {
    let model = match CounterModel::from_snapshot(&input.before_model) {
        Ok(model) => model,
        Err(diagnostics) => {
            let summary = diagnostics
                .first()
                .map(|diagnostic| diagnostic.summary.clone())
                .unwrap_or_else(|| "counter model is invalid".to_owned());
            return AppReducerOutcome::rejected(
                input,
                AppDiagnostic::new(
                    NAMESPACE_REDUCER,
                    "app.reducer.invalid_counter_model",
                    summary,
                ),
            );
        }
    };

    let action = match CounterAction::from_app_action(&input.action) {
        Ok(action) => action,
        Err(diagnostic) => return AppReducerOutcome::rejected(input, diagnostic),
    };

    let next_count = match action {
        CounterAction::Increment => model.count + 1,
        CounterAction::Reset => 0,
    };
    let after_model =
        CounterModel { count: next_count }.to_snapshot(input.before_model.revision.next());

    AppReducerOutcome::accepted(
        input.before_model,
        input.action,
        after_model,
        AppEffectPlan::NoEffect,
    )
}

pub fn counter_action_for_test(action_id: &str) -> AppAction {
    AppAction::new(
        AppActionId::new(action_id),
        AppActionVersion::new(1),
        AppActionPayload::Unit,
        AppActionSource::local_headless(None, None, Vec::new()),
    )
}
