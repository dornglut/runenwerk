//! File: domain/editor/editor_shell/src/workspace/projection_ratification.rs
//! Purpose: Ratify editor-shell projection route artifacts using foundation reports.

use std::collections::BTreeMap;

use foundation_ratification::{RatificationIssue, RatificationReport, Ratifier};

use crate::{
    FLOATING_DROP_ZONE_WIDGET_ID, PanelInstanceId, ProjectedPanelSlot, ProjectedTabButtonRoute,
    ProjectedTabDropRoute, ProjectedTabDropTarget, ProjectedTabStackSlot,
    StructuralWidgetRoutingContext, TabStackId, ToolSurfaceInstanceId, WidgetId,
    WorkspaceProjectionArtifact, projected_host_tab_stacks,
};

pub type EditorShellProjectionRatificationReport = RatificationReport<
    EditorShellProjectionRatificationCode,
    EditorShellProjectionRatificationSubject,
>;

/// editor_shell-owned projection/route ratification issue codes.
///
/// Foundation owns the report shape. editor_shell owns the meaning of these
/// route/projection failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EditorShellProjectionRatificationCode {
    MissingTabButtonRoute,
    TabButtonRouteMismatch,
    MissingTabDropRoute,
    TabDropRouteMismatch,
    MissingFloatingDropRoute,
    FloatingDropRouteMismatch,
    StaleWidgetRoutingContext,
    WidgetRoutingContextSurfaceMismatch,
}

/// editor_shell-owned projection/route ratification subjects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EditorShellProjectionRatificationSubject {
    Widget(WidgetId),
    Panel(PanelInstanceId),
    TabStack(TabStackId),
}

/// Concrete candidate for Phase 3 ratification.
///
/// The candidate observes an already-produced projection artifact. It does not
/// mutate workspace state and does not rebuild projection output.
#[derive(Debug, Clone, Copy)]
pub struct WorkspaceProjectionRouteCandidate<'a> {
    artifact: &'a WorkspaceProjectionArtifact,
}

impl<'a> WorkspaceProjectionRouteCandidate<'a> {
    pub const fn new(artifact: &'a WorkspaceProjectionArtifact) -> Self {
        Self { artifact }
    }

    pub const fn artifact(self) -> &'a WorkspaceProjectionArtifact {
        self.artifact
    }
}

/// Ratifies the internal route consistency of editor-shell projection output.
#[derive(Debug, Clone, Copy, Default)]
pub struct EditorShellProjectionRouteRatifier;

impl<'a> Ratifier<WorkspaceProjectionRouteCandidate<'a>> for EditorShellProjectionRouteRatifier {
    type Code = EditorShellProjectionRatificationCode;
    type Subject = EditorShellProjectionRatificationSubject;

    fn ratify(
        &self,
        candidate: &WorkspaceProjectionRouteCandidate<'a>,
    ) -> EditorShellProjectionRatificationReport {
        ratify_workspace_projection_routes(*candidate)
    }
}

pub fn ratify_workspace_projection_routes(
    candidate: WorkspaceProjectionRouteCandidate<'_>,
) -> EditorShellProjectionRatificationReport {
    let artifact = candidate.artifact();
    let mut report = RatificationReport::accepted();
    let active_panels = active_panels_by_identity(artifact);

    for stack in all_projected_stacks(artifact) {
        ratify_stack_tab_routes(stack, artifact, &mut report);
        ratify_stack_drop_routes(stack, artifact, &mut report);
    }

    ratify_floating_drop_route(artifact, &mut report);
    ratify_widget_contexts(artifact, &active_panels, &mut report);

    report
}

fn all_projected_stacks(artifact: &WorkspaceProjectionArtifact) -> Vec<&ProjectedTabStackSlot> {
    let mut stacks = projected_host_tab_stacks(&artifact.root_host);
    stacks.extend(artifact.floating_hosts.iter().map(|host| &host.tab_stack));
    stacks
}

fn active_panels_by_identity(
    artifact: &WorkspaceProjectionArtifact,
) -> BTreeMap<(PanelInstanceId, TabStackId), Option<ToolSurfaceInstanceId>> {
    let mut active_panels = BTreeMap::new();

    for stack in all_projected_stacks(artifact) {
        if let Some(panel) = &stack.active_panel {
            active_panels.insert(active_panel_key(panel), panel.active_tool_surface);
        }
    }

    active_panels
}

fn active_panel_key(panel: &ProjectedPanelSlot) -> (PanelInstanceId, TabStackId) {
    (panel.panel_instance_id, panel.tab_stack_id)
}

fn ratify_stack_tab_routes(
    stack: &ProjectedTabStackSlot,
    artifact: &WorkspaceProjectionArtifact,
    report: &mut EditorShellProjectionRatificationReport,
) {
    for tab in &stack.tabs {
        let expected = ProjectedTabButtonRoute {
            panel_instance_id: tab.panel.panel_instance_id,
            tab_stack_id: stack.tab_stack_id,
        };

        match artifact.tab_button_route_by_widget_id.get(&tab.widget_id) {
            Some(route) if *route == expected => {}
            Some(_) => report.push(RatificationIssue::error(
                EditorShellProjectionRatificationCode::TabButtonRouteMismatch,
                EditorShellProjectionRatificationSubject::Widget(tab.widget_id),
                format!(
                    "tab button widget {} has a route that does not match panel {} in tab stack {}",
                    tab.widget_id.0,
                    tab.panel.panel_instance_id.raw(),
                    stack.tab_stack_id.raw(),
                ),
            )),
            None => report.push(RatificationIssue::error(
                EditorShellProjectionRatificationCode::MissingTabButtonRoute,
                EditorShellProjectionRatificationSubject::Widget(tab.widget_id),
                format!(
                    "tab button widget {} has no projected tab button route",
                    tab.widget_id.0,
                ),
            )),
        }
    }
}

fn ratify_stack_drop_routes(
    stack: &ProjectedTabStackSlot,
    artifact: &WorkspaceProjectionArtifact,
    report: &mut EditorShellProjectionRatificationReport,
) {
    for drop_slot in &stack.drop_slots {
        let expected = ProjectedTabDropRoute {
            target: ProjectedTabDropTarget::TabStack {
                tab_stack_id: stack.tab_stack_id,
                insert_index: drop_slot.insert_index,
            },
        };

        match artifact.tab_drop_route_by_widget_id.get(&drop_slot.widget_id) {
            Some(route) if *route == expected => {}
            Some(_) => report.push(RatificationIssue::error(
                EditorShellProjectionRatificationCode::TabDropRouteMismatch,
                EditorShellProjectionRatificationSubject::Widget(drop_slot.widget_id),
                format!(
                    "tab drop widget {} has a route that does not match tab stack {} at insert index {}",
                    drop_slot.widget_id.0,
                    stack.tab_stack_id.raw(),
                    drop_slot.insert_index,
                ),
            )),
            None => report.push(RatificationIssue::error(
                EditorShellProjectionRatificationCode::MissingTabDropRoute,
                EditorShellProjectionRatificationSubject::Widget(drop_slot.widget_id),
                format!(
                    "tab drop widget {} has no projected tab drop route",
                    drop_slot.widget_id.0,
                ),
            )),
        }
    }
}

fn ratify_floating_drop_route(
    artifact: &WorkspaceProjectionArtifact,
    report: &mut EditorShellProjectionRatificationReport,
) {
    let expected = ProjectedTabDropRoute {
        target: ProjectedTabDropTarget::NewFloatingHost,
    };

    match artifact
        .tab_drop_route_by_widget_id
        .get(&FLOATING_DROP_ZONE_WIDGET_ID)
    {
        Some(route) if *route == expected => {}
        Some(_) => report.push(RatificationIssue::error(
            EditorShellProjectionRatificationCode::FloatingDropRouteMismatch,
            EditorShellProjectionRatificationSubject::Widget(FLOATING_DROP_ZONE_WIDGET_ID),
            "floating drop-zone widget route does not target a new floating host",
        )),
        None => report.push(RatificationIssue::error(
            EditorShellProjectionRatificationCode::MissingFloatingDropRoute,
            EditorShellProjectionRatificationSubject::Widget(FLOATING_DROP_ZONE_WIDGET_ID),
            "floating drop-zone widget has no projected drop route",
        )),
    }
}

fn ratify_widget_contexts(
    artifact: &WorkspaceProjectionArtifact,
    active_panels: &BTreeMap<(PanelInstanceId, TabStackId), Option<ToolSurfaceInstanceId>>,
    report: &mut EditorShellProjectionRatificationReport,
) {
    for (widget_id, context) in &artifact.widget_context_by_id {
        ratify_widget_context(*widget_id, *context, active_panels, report);
    }
}

fn ratify_widget_context(
    widget_id: WidgetId,
    context: StructuralWidgetRoutingContext,
    active_panels: &BTreeMap<(PanelInstanceId, TabStackId), Option<ToolSurfaceInstanceId>>,
    report: &mut EditorShellProjectionRatificationReport,
) {
    let key = (context.panel_instance_id, context.tab_stack_id);

    let Some(expected_surface) = active_panels.get(&key) else {
        report.push(RatificationIssue::error(
            EditorShellProjectionRatificationCode::StaleWidgetRoutingContext,
            EditorShellProjectionRatificationSubject::Widget(widget_id),
            format!(
                "widget {} routes to non-active or missing panel {} in tab stack {}",
                widget_id.0,
                context.panel_instance_id.raw(),
                context.tab_stack_id.raw(),
            ),
        ));
        return;
    };

    if *expected_surface != context.active_tool_surface {
        report.push(RatificationIssue::error(
            EditorShellProjectionRatificationCode::WidgetRoutingContextSurfaceMismatch,
            EditorShellProjectionRatificationSubject::Widget(widget_id),
            format!(
                "widget {} routes to panel {} with stale active tool surface context",
                widget_id.0,
                context.panel_instance_id.raw(),
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        WorkspaceId, WorkspaceIdentityAllocator, WorkspaceState, project_workspace_for_shell,
    };

    fn projected_workspace() -> WorkspaceProjectionArtifact {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace = WorkspaceState::bootstrap_current_layout(
            WorkspaceId::try_from_raw(1).unwrap(),
            &mut allocator,
        );

        project_workspace_for_shell(&workspace).expect("workspace projection should succeed")
    }

    fn first_tab_stack_pair(
        artifact: &WorkspaceProjectionArtifact,
    ) -> (&ProjectedTabStackSlot, &ProjectedTabStackSlot) {
        let stacks = all_projected_stacks(artifact);
        let first = stacks
            .first()
            .expect("workspace should project a tab stack");
        let second = stacks
            .iter()
            .find(|stack| stack.tab_stack_id != first.tab_stack_id)
            .expect("workspace should project a second tab stack");
        (first, second)
    }

    #[test]
    fn projection_route_ratifier_accepts_clean_projection_artifact() {
        let artifact = projected_workspace();
        let ratifier = EditorShellProjectionRouteRatifier;

        let report = ratifier.ratify(&WorkspaceProjectionRouteCandidate::new(&artifact));

        assert!(report.is_accepted());
        assert!(report.is_clean());
        assert!(report.is_empty());
    }

    #[test]
    fn projection_route_ratifier_rejects_missing_tab_button_route() {
        let mut artifact = projected_workspace();
        let (stack, _) = first_tab_stack_pair(&artifact);
        let widget_id = stack.tabs[0].widget_id;
        artifact.tab_button_route_by_widget_id.remove(&widget_id);

        let report =
            ratify_workspace_projection_routes(WorkspaceProjectionRouteCandidate::new(&artifact));

        assert!(report.is_rejected());
        assert_eq!(report.len(), 1);
        assert_eq!(
            report.issues()[0].code(),
            &EditorShellProjectionRatificationCode::MissingTabButtonRoute
        );
        assert_eq!(
            report.issues()[0].subject(),
            &EditorShellProjectionRatificationSubject::Widget(widget_id)
        );
    }

    #[test]
    fn projection_route_ratifier_rejects_mismatched_tab_button_route() {
        let mut artifact = projected_workspace();
        let (stack, other_stack) = first_tab_stack_pair(&artifact);
        let widget_id = stack.tabs[0].widget_id;
        let panel_instance_id = stack.tabs[0].panel.panel_instance_id;
        let mismatched_tab_stack_id = other_stack.tab_stack_id;
        artifact.tab_button_route_by_widget_id.insert(
            widget_id,
            ProjectedTabButtonRoute {
                panel_instance_id,
                tab_stack_id: mismatched_tab_stack_id,
            },
        );

        let report =
            ratify_workspace_projection_routes(WorkspaceProjectionRouteCandidate::new(&artifact));

        assert!(report.is_rejected());
        assert_eq!(
            report.issues()[0].code(),
            &EditorShellProjectionRatificationCode::TabButtonRouteMismatch
        );
    }

    #[test]
    fn projection_route_ratifier_rejects_missing_floating_drop_route() {
        let mut artifact = projected_workspace();
        artifact
            .tab_drop_route_by_widget_id
            .remove(&FLOATING_DROP_ZONE_WIDGET_ID);

        let report =
            ratify_workspace_projection_routes(WorkspaceProjectionRouteCandidate::new(&artifact));

        assert!(report.is_rejected());
        assert_eq!(
            report.issues()[0].code(),
            &EditorShellProjectionRatificationCode::MissingFloatingDropRoute
        );
    }

    #[test]
    fn projection_route_ratifier_rejects_stale_widget_context() {
        let mut artifact = projected_workspace();
        let (stack, _) = first_tab_stack_pair(&artifact);
        let tab_stack_id = stack.tab_stack_id;
        let widget_id = WidgetId(999_999);
        artifact.widget_context_by_id.insert(
            widget_id,
            StructuralWidgetRoutingContext {
                panel_instance_id: PanelInstanceId::try_from_raw(999).unwrap(),
                active_tool_surface: None,
                tab_stack_id,
            },
        );

        let report =
            ratify_workspace_projection_routes(WorkspaceProjectionRouteCandidate::new(&artifact));

        assert!(report.is_rejected());
        assert_eq!(
            report.issues()[0].code(),
            &EditorShellProjectionRatificationCode::StaleWidgetRoutingContext
        );
        assert_eq!(
            report.issues()[0].subject(),
            &EditorShellProjectionRatificationSubject::Widget(widget_id)
        );
    }

    #[test]
    fn projection_route_ratifier_rejects_stale_widget_surface_context() {
        let mut artifact = projected_workspace();
        let (widget_id, mut context) = artifact
            .widget_context_by_id
            .iter()
            .next()
            .map(|(widget_id, context)| (*widget_id, *context))
            .expect("projection should include widget routing contexts");

        context.active_tool_surface = None;
        artifact.widget_context_by_id.insert(widget_id, context);

        let report =
            ratify_workspace_projection_routes(WorkspaceProjectionRouteCandidate::new(&artifact));

        assert!(report.is_rejected());
        assert_eq!(
            report.issues()[0].code(),
            &EditorShellProjectionRatificationCode::WidgetRoutingContextSurfaceMismatch
        );
    }
}
