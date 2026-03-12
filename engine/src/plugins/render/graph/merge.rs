use crate::plugins::render::RenderFlowValidationError;
use crate::plugins::render::api::RenderFlow;
use crate::plugins::render::composition::{
    RenderFlowContribution, validate_contribution_namespaces,
};

pub fn merge_flow_with_contributions(
    base_flow: &RenderFlow,
    contributions: &[RenderFlowContribution],
) -> Result<RenderFlow, RenderFlowValidationError> {
    let mut issues = Vec::<String>::new();

    if let Err(err) = validate_contribution_namespaces(base_flow, contributions) {
        issues.extend(err.issues);
    }

    let mut merged = base_flow.clone();
    for contribution in contributions {
        merged = merged.merge(contribution.flow().clone());
    }

    if let Err(err) = merged.validate() {
        issues.extend(err.issues);
    }

    if issues.is_empty() {
        Ok(merged)
    } else {
        Err(RenderFlowValidationError { issues })
    }
}
