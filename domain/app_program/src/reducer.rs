//! Pure reducer input and outcome contracts.

use crate::action::AppAction;
use crate::effect::AppEffectPlan;
use crate::model::AppModelSnapshot;
use crate::report::AppDiagnostic;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppReducerInput {
    pub before_model: AppModelSnapshot,
    pub action: AppAction,
}

impl AppReducerInput {
    pub fn new(before_model: AppModelSnapshot, action: AppAction) -> Self {
        Self {
            before_model,
            action,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppReducerOutcome {
    pub before_model: AppModelSnapshot,
    pub action: AppAction,
    pub accepted: bool,
    pub after_model: AppModelSnapshot,
    pub effect_plan: AppEffectPlan,
    pub diagnostics: Vec<AppDiagnostic>,
}

impl AppReducerOutcome {
    pub fn accepted(
        before_model: AppModelSnapshot,
        action: AppAction,
        after_model: AppModelSnapshot,
        effect_plan: AppEffectPlan,
    ) -> Self {
        Self {
            before_model,
            action,
            accepted: true,
            after_model,
            effect_plan,
            diagnostics: Vec::new(),
        }
    }

    pub fn rejected(input: AppReducerInput, diagnostic: AppDiagnostic) -> Self {
        Self {
            after_model: input.before_model.clone(),
            before_model: input.before_model,
            action: input.action,
            accepted: false,
            effect_plan: AppEffectPlan::NoEffect,
            diagnostics: vec![diagnostic],
        }
    }
}
