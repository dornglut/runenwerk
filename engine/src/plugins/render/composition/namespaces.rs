use crate::plugins::render::RenderFlow;
use crate::plugins::render::api::namespace_of;
use crate::plugins::render::composition::RenderFlowContribution;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContributionNamespaceValidationError {
    pub issues: Vec<String>,
}

impl std::fmt::Display for ContributionNamespaceValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.issues.join("; "))
    }
}

impl std::error::Error for ContributionNamespaceValidationError {}

pub fn validate_contribution_namespaces(
    base_flow: &RenderFlow,
    contributions: &[RenderFlowContribution],
) -> Result<(), ContributionNamespaceValidationError> {
    let mut issues = Vec::<String>::new();
    let mut namespaces = BTreeSet::<String>::new();

    for contribution in contributions {
        let namespace = contribution.namespace();
        if !is_valid_namespace(namespace) {
            issues.push(format!(
                "invalid contribution namespace '{}': only ASCII alphanumeric, '_' and '-' are allowed",
                namespace
            ));
        }
        if !namespaces.insert(namespace.to_string()) {
            issues.push(format!(
                "duplicate render flow contribution namespace '{}'",
                namespace
            ));
        }
    }

    let mut pass_owner = BTreeMap::<String, String>::new();
    let mut resource_owner = BTreeMap::<String, String>::new();

    for pass in &base_flow.graph().passes.passes {
        pass_owner.insert(
            pass.id.as_str().to_string(),
            format!("flow::{}", base_flow.id().as_str()),
        );
    }
    for resource in &base_flow.graph().resources.resources {
        resource_owner.insert(
            resource.id().as_str().to_string(),
            format!("flow::{}", base_flow.id().as_str()),
        );
    }

    for contribution in contributions {
        let owner = format!("contribution::{}", contribution.namespace());
        for pass in &contribution.flow().graph().passes.passes {
            if let Some(existing) = pass_owner.insert(pass.id.as_str().to_string(), owner.clone()) {
                issues.push(format!(
                    "duplicate pass id '{}' declared by {} and {}",
                    pass.id.as_str(),
                    existing,
                    owner
                ));
            }
        }
        for resource in &contribution.flow().graph().resources.resources {
            if let Some(existing) =
                resource_owner.insert(resource.id().as_str().to_string(), owner.clone())
            {
                issues.push(format!(
                    "duplicate resource id '{}' declared by {} and {}",
                    resource.id().as_str(),
                    existing,
                    owner
                ));
            }
        }
    }

    let known_passes = pass_owner.keys().cloned().collect::<BTreeSet<_>>();
    for contribution in contributions {
        for pass in &contribution.flow().graph().passes.passes {
            for dependency in &pass.depends_on {
                if !known_passes.contains(dependency.as_str()) {
                    issues.push(format!(
                        "invalid cross-plugin dependency: pass '{}' in contribution '{}' depends on unknown pass '{}'",
                        pass.id.as_str(),
                        contribution.namespace(),
                        dependency.as_str()
                    ));
                }
            }
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(ContributionNamespaceValidationError { issues })
    }
}

pub fn namespace_matches(id: &str, namespace: &str) -> bool {
    namespace_of(id).is_some_and(|value| value == namespace)
}

fn is_valid_namespace(namespace: &str) -> bool {
    !namespace.is_empty()
        && namespace
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}
