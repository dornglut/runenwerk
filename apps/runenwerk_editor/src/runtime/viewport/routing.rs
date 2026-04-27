//! File: apps/runenwerk_editor/src/runtime/viewport/routing.rs
//! Purpose: Shared viewport routing policy for structural bindings and bootstrap seams.

use editor_viewport::{ArtifactObservationFrame, ViewportId};

use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
};
use crate::shell::RunenwerkEditorShellState;

/// Resolve the viewport observation frame routed through structural bindings.
///
/// Bootstrap fallback is intentionally constrained:
/// it is only allowed before shell projection artifacts exist and only when
/// exactly one observed viewport frame exists.
pub fn resolve_structural_viewport_products<'a>(
    shell_state: &RunenwerkEditorShellState,
    viewport_observations: &'a ViewportArtifactObservationResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
) -> Option<&'a ArtifactObservationFrame> {
    let structural_viewport_id = resolve_structural_viewport_id(shell_state, tool_surface_bindings);
    let projection_artifacts_available = shell_state.last_projection_artifacts().is_some();
    let viewport_id = select_viewport_id_with_bootstrap_policy(
        structural_viewport_id,
        projection_artifacts_available,
        viewport_observations.viewport_ids(),
    )?;
    viewport_observations.frame_for(viewport_id)
}

pub fn resolve_structural_viewport_id(
    shell_state: &RunenwerkEditorShellState,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
) -> Option<ViewportId> {
    let structural_context = shell_state
        .last_projection_artifacts()
        .and_then(|artifacts| {
            artifacts
                .widget_structural_context_by_id
                .get(&editor_shell::VIEWPORT_SURFACE_EMBED_WIDGET_ID)
                .copied()
        })?;
    let binding = tool_surface_bindings.resolve_structural_context(structural_context)?;
    Some(binding.viewport_id)
}

fn bootstrap_single_viewport_id(
    mut observed_viewport_ids: impl Iterator<Item = ViewportId>,
) -> Option<ViewportId> {
    let viewport_id = observed_viewport_ids.next()?;
    if observed_viewport_ids.next().is_none() {
        Some(viewport_id)
    } else {
        None
    }
}

fn select_viewport_id_with_bootstrap_policy(
    structural_viewport_id: Option<ViewportId>,
    projection_artifacts_available: bool,
    observed_viewport_ids: impl Iterator<Item = ViewportId>,
) -> Option<ViewportId> {
    if let Some(viewport_id) = structural_viewport_id {
        return Some(viewport_id);
    }
    if projection_artifacts_available {
        return None;
    }
    bootstrap_single_viewport_id(observed_viewport_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_viewport::ViewportId;

    #[test]
    fn bootstrap_single_viewport_requires_exactly_one_observed_viewport() {
        assert_eq!(
            bootstrap_single_viewport_id([ViewportId(5)].into_iter()),
            Some(ViewportId(5)),
        );
        assert_eq!(bootstrap_single_viewport_id([].into_iter()), None);
        assert_eq!(
            bootstrap_single_viewport_id([ViewportId(5), ViewportId(6)].into_iter()),
            None,
        );
    }

    #[test]
    fn bootstrap_policy_only_applies_before_projection_artifacts_exist() {
        let bootstrap_ids = [ViewportId(11)];
        assert_eq!(
            select_viewport_id_with_bootstrap_policy(None, false, bootstrap_ids.into_iter()),
            Some(ViewportId(11)),
        );
        assert_eq!(
            select_viewport_id_with_bootstrap_policy(None, true, [ViewportId(11)].into_iter()),
            None,
        );
    }

    #[test]
    fn bootstrap_policy_prioritizes_structural_viewport_binding() {
        assert_eq!(
            select_viewport_id_with_bootstrap_policy(
                Some(ViewportId(22)),
                false,
                [ViewportId(11)].into_iter()
            ),
            Some(ViewportId(22)),
        );
        assert_eq!(
            select_viewport_id_with_bootstrap_policy(
                Some(ViewportId(22)),
                true,
                [ViewportId(11)].into_iter()
            ),
            Some(ViewportId(22)),
        );
    }
}
