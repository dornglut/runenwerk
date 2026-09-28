//! Product-owned Editor automation proof.

use editor_shell::{
    ShellCommand, SurfaceSessionMutation, ViewportSessionMutation, ViewportToolKind,
};
use engine::automation::AutomationOwnerAdapter;
use serde::{Deserialize, Serialize};
use ui_composition::MountedUnitId;

use crate::runtime::resources::EditorHostResource;
use crate::shell::dispatch_shell_command;

pub const EDITOR_AUTOMATION_SCENARIO_PRODUCT_ID: &str = "runenwerk.editor.automation";
pub const EDITOR_AUTOMATION_SCENARIO_PRODUCT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorAutomationScenarioTargetV1 {
    UniqueSceneViewport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorAutomationViewportToolV1 {
    Select,
    Translate,
    Rotate,
    Scale,
}

impl From<EditorAutomationViewportToolV1> for ViewportToolKind {
    fn from(value: EditorAutomationViewportToolV1) -> Self {
        match value {
            EditorAutomationViewportToolV1::Select => Self::Select,
            EditorAutomationViewportToolV1::Translate => Self::Translate,
            EditorAutomationViewportToolV1::Rotate => Self::Rotate,
            EditorAutomationViewportToolV1::Scale => Self::Scale,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EditorAutomationScenarioStepV1 {
    ActivateViewportTool {
        target: EditorAutomationScenarioTargetV1,
        tool: EditorAutomationViewportToolV1,
    },
    AssertViewportTool {
        target: EditorAutomationScenarioTargetV1,
        tool: EditorAutomationViewportToolV1,
    },
}

pub fn validate_editor_automation_scenario_step_v1(
    _step: &EditorAutomationScenarioStepV1,
) -> Result<(), String> {
    Ok(())
}

pub fn resolve_editor_automation_scenario_target_v1(
    host: &EditorHostResource,
    target: EditorAutomationScenarioTargetV1,
) -> Result<EditorAutomationTarget, String> {
    match target {
        EditorAutomationScenarioTargetV1::UniqueSceneViewport => {
            resolve_unique_scene_viewport_target(host)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorAutomationTarget {
    pub mounted_unit_id: MountedUnitId,
}

pub fn resolve_unique_scene_viewport_target(
    host: &EditorHostResource,
) -> Result<EditorAutomationTarget, String> {
    let mut viewports = host
        .shell_state
        .composition_runtime()
        .extension()
        .mounted_units()
        .iter()
        .filter(|record| {
            record.stable_content_key == crate::shell::tool_suites::SCENE_VIEWPORT_SURFACE_KEY
        });

    let viewport = viewports
        .next()
        .ok_or_else(|| "Editor automation found no mounted scene viewport".to_owned())?;
    if viewports.next().is_some() {
        return Err(
            "Editor automation scene viewport target is ambiguous: multiple viewports are mounted"
                .to_owned(),
        );
    }

    Ok(EditorAutomationTarget {
        mounted_unit_id: viewport.mounted_unit_id,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorAutomationCommand {
    ActivateViewportTool(ViewportToolKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorAutomationQuery {
    ViewportTool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorAutomationObservation {
    ViewportTool(ViewportToolKind),
}

pub struct EditorAutomationAdapter<'a> {
    host: &'a mut EditorHostResource,
}

impl<'a> EditorAutomationAdapter<'a> {
    pub fn new(host: &'a mut EditorHostResource) -> Self {
        Self { host }
    }

    fn resolve_viewport_target(
        &self,
        target: &EditorAutomationTarget,
    ) -> Result<editor_shell::StructuralCommandTarget, String> {
        let mounted = self
            .host
            .shell_state
            .composition_runtime()
            .extension()
            .mounted_units()
            .iter()
            .find(|record| record.mounted_unit_id == target.mounted_unit_id)
            .ok_or_else(|| "Editor automation target is missing or stale".to_owned())?;
        if mounted.stable_content_key != crate::shell::tool_suites::SCENE_VIEWPORT_SURFACE_KEY {
            return Err("Editor automation target is not a viewport surface".to_owned());
        }
        self.host
            .shell_state
            .structural_command_target_for_mounted_unit(target.mounted_unit_id)
            .ok_or_else(|| "Editor automation target has no current structural route".to_owned())
    }
}

impl AutomationOwnerAdapter for EditorAutomationAdapter<'_> {
    type Target = EditorAutomationTarget;
    type Command = EditorAutomationCommand;
    type Query = EditorAutomationQuery;
    type Observation = EditorAutomationObservation;
    type Error = String;

    fn dispatch(
        &mut self,
        target: &Self::Target,
        command: Self::Command,
    ) -> Result<(), Self::Error> {
        let structural_target = self.resolve_viewport_target(target)?;
        let projection_epoch = self.host.shell_state.current_projection_epoch();
        let mutation = match command {
            EditorAutomationCommand::ActivateViewportTool(tool) => {
                SurfaceSessionMutation::Viewport(ViewportSessionMutation::ActivateTool { tool })
            }
        };

        dispatch_shell_command(
            &mut self.host.app,
            Some(&mut self.host.shell_state),
            ShellCommand::ApplySurfaceSessionMutation {
                target: structural_target,
                mutation,
                projection_epoch,
            },
            None,
            None,
            None,
            Some(projection_epoch),
        )
        .map_err(|error| error.to_string())
    }

    fn query(
        &mut self,
        target: &Self::Target,
        query: Self::Query,
    ) -> Result<Self::Observation, Self::Error> {
        self.resolve_viewport_target(target)?;
        match query {
            EditorAutomationQuery::ViewportTool => Ok(EditorAutomationObservation::ViewportTool(
                self.host
                    .app
                    .surface_sessions()
                    .viewport_tool(target.mounted_unit_id),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::automation::{
        AutomationExecutionMode, AutomationSession, AutomationSessionId, AutomationStepKind,
        AutomationStepOutcome, AutomationStepRecord, AutomationStepResult, InputSourceId,
    };

    #[test]
    fn product_semantic_session_mutates_and_queries_one_mounted_viewport() {
        let mut app = crate::runtime::build_headless_app()
            .expect("headless Editor automation fixture should build");
        let target = {
            let host = app
                .world()
                .resource::<EditorHostResource>()
                .expect("headless Editor should install EditorHostResource");
            resolve_unique_scene_viewport_target(host)
                .expect("headless Full Editor should contain exactly one scene viewport")
        };
        let mut session =
            AutomationSession::new(AutomationSessionId::new(20), InputSourceId::new(20_001));

        let observed = {
            let host = app
                .world_mut()
                .resource_mut::<EditorHostResource>()
                .expect("headless Editor should install EditorHostResource");
            let mut adapter = EditorAutomationAdapter::new(host);
            assert_eq!(
                session.dispatch_product(
                    AutomationExecutionMode::ProductSemantic,
                    &mut adapter,
                    &target,
                    EditorAutomationCommand::ActivateViewportTool(ViewportToolKind::Rotate),
                ),
                AutomationStepResult::Dispatched
            );
            match session.query_owner(&mut adapter, &target, EditorAutomationQuery::ViewportTool) {
                AutomationStepResult::EffectConfirmed(observation) => observation,
                other => panic!("Editor viewport query should confirm state, got {other:?}"),
            }
        };

        assert_eq!(
            session.assert_observation(&observed, |observation| {
                *observation == EditorAutomationObservation::ViewportTool(ViewportToolKind::Rotate)
            }),
            AutomationStepResult::AssertionPassed
        );
        assert_eq!(
            session.finish(&mut app),
            AutomationStepResult::EffectConfirmed(())
        );
        assert_eq!(
            session
                .history()
                .iter()
                .map(AutomationStepRecord::kind)
                .collect::<Vec<_>>(),
            vec![
                AutomationStepKind::ProductDispatch,
                AutomationStepKind::OwnerQuery,
                AutomationStepKind::OwnerAssertion,
                AutomationStepKind::Finish,
            ]
        );
        assert_eq!(
            session
                .history()
                .iter()
                .map(AutomationStepRecord::outcome)
                .collect::<Vec<_>>(),
            vec![
                AutomationStepOutcome::Dispatched,
                AutomationStepOutcome::EffectConfirmed,
                AutomationStepOutcome::AssertionPassed,
                AutomationStepOutcome::EffectConfirmed,
            ]
        );
    }

    #[test]
    fn stale_mounted_unit_fails_closed_without_redirecting() {
        let mut app = crate::runtime::build_headless_app()
            .expect("headless Editor automation fixture should build");
        let mut session =
            AutomationSession::new(AutomationSessionId::new(21), InputSourceId::new(20_002));
        let target = EditorAutomationTarget {
            mounted_unit_id: MountedUnitId::try_from_raw(999_999)
                .expect("test mounted-unit id should be valid"),
        };

        let host = app
            .world_mut()
            .resource_mut::<EditorHostResource>()
            .expect("headless Editor should install EditorHostResource");
        let mut adapter = EditorAutomationAdapter::new(host);

        assert!(matches!(
            session.dispatch_product(
                AutomationExecutionMode::ProductSemantic,
                &mut adapter,
                &target,
                EditorAutomationCommand::ActivateViewportTool(ViewportToolKind::Scale),
            ),
            AutomationStepResult::InfrastructureFailure(_)
        ));
        assert_eq!(
            session.history()[0].outcome(),
            AutomationStepOutcome::InfrastructureFailure
        );
    }

    #[test]
    fn native_mode_does_not_fall_back_to_editor_product_semantics() {
        let mut app = crate::runtime::build_headless_app()
            .expect("headless Editor automation fixture should build");
        let target = {
            let host = app
                .world()
                .resource::<EditorHostResource>()
                .expect("headless Editor should install EditorHostResource");
            resolve_unique_scene_viewport_target(host)
                .expect("headless Full Editor should contain exactly one scene viewport")
        };
        let mut session =
            AutomationSession::new(AutomationSessionId::new(22), InputSourceId::new(20_003));
        let host = app
            .world_mut()
            .resource_mut::<EditorHostResource>()
            .expect("headless Editor should install EditorHostResource");
        let mut adapter = EditorAutomationAdapter::new(host);

        assert_eq!(
            session.dispatch_product(
                AutomationExecutionMode::NativeOs,
                &mut adapter,
                &target,
                EditorAutomationCommand::ActivateViewportTool(ViewportToolKind::Translate),
            ),
            AutomationStepResult::Unsupported
        );
        assert_eq!(
            session.query_owner(&mut adapter, &target, EditorAutomationQuery::ViewportTool,),
            AutomationStepResult::EffectConfirmed(EditorAutomationObservation::ViewportTool(
                ViewportToolKind::Select
            ))
        );
        assert_eq!(
            session.history()[0].outcome(),
            AutomationStepOutcome::Unsupported
        );
        assert_eq!(
            session.history()[1].outcome(),
            AutomationStepOutcome::EffectConfirmed
        );
    }
}
