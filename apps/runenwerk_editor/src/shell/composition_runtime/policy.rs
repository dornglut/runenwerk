use ui_composition::{
    CompositionCapabilityPolicy, CompositionLifecyclePolicy, CompositionPolicyDecision,
    CompositionSnapshot, CompositionTargetPolicy, CompositionTransaction,
};

pub(crate) struct EditorCompositionPolicy;

impl CompositionLifecyclePolicy for EditorCompositionPolicy {
    fn evaluate(
        &self,
        _: CompositionSnapshot<'_>,
        _: &CompositionTransaction,
    ) -> CompositionPolicyDecision {
        CompositionPolicyDecision::Accepted
    }
}

impl CompositionCapabilityPolicy for EditorCompositionPolicy {
    fn evaluate(
        &self,
        _: CompositionSnapshot<'_>,
        _: &CompositionTransaction,
    ) -> CompositionPolicyDecision {
        CompositionPolicyDecision::Accepted
    }
}

impl CompositionTargetPolicy for EditorCompositionPolicy {
    fn evaluate(
        &self,
        _: CompositionSnapshot<'_>,
        _: &CompositionTransaction,
    ) -> CompositionPolicyDecision {
        CompositionPolicyDecision::Accepted
    }
}
