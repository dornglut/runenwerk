//! Product-owned Render Lab automation proof.

use engine::automation::AutomationOwnerAdapter;
use engine::plugins::default_plugins;
use engine::prelude::{App, Update};

use crate::camera::{RenderLabCamera, update_render_lab_camera_system};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderLabAutomationTarget;

#[derive(Debug)]
pub enum RenderLabAutomationCommand {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderLabAutomationQuery {
    Camera,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderLabCameraObservation {
    pub yaw_radians: f64,
    pub pitch_radians: f64,
    pub distance: f64,
    pub pan: [f64; 2],
}

pub struct RenderLabAutomationAdapter<'a> {
    app: &'a mut App,
}

impl<'a> RenderLabAutomationAdapter<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Self { app }
    }
}

impl AutomationOwnerAdapter for RenderLabAutomationAdapter<'_> {
    type Target = RenderLabAutomationTarget;
    type Command = RenderLabAutomationCommand;
    type Query = RenderLabAutomationQuery;
    type Observation = RenderLabCameraObservation;
    type Error = &'static str;

    fn dispatch(
        &mut self,
        _target: &Self::Target,
        command: Self::Command,
    ) -> Result<(), Self::Error> {
        match command {}
    }

    fn query(
        &mut self,
        _target: &Self::Target,
        query: Self::Query,
    ) -> Result<Self::Observation, Self::Error> {
        match query {
            RenderLabAutomationQuery::Camera => {
                let camera = self
                    .app
                    .world()
                    .resource::<RenderLabCamera>()
                    .map_err(|_| "Render Lab automation camera resource is unavailable")?;
                Ok(RenderLabCameraObservation {
                    yaw_radians: camera.yaw_radians,
                    pitch_radians: camera.pitch_radians,
                    distance: camera.distance,
                    pan: camera.pan,
                })
            }
        }
    }
}

pub fn build_headless_automation_app() -> App {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.init_resource::<RenderLabCamera>();
    app.add_systems(Update, update_render_lab_camera_system);
    app
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::automation::{
        AppAutomationInputReplayExt, AppAutomationInputTraceExt, AutomationExecutionMode,
        AutomationInputReplayOutcome, AutomationInputReplaySourceMap,
        AutomationInputReplayStateAssumption, AutomationInputTracePlugin, AutomationSession,
        AutomationSessionId, AutomationStepResult, DigitalState, InputObservation, InputSourceId,
        PointerButton, PointerButtonInput, RelativeMotionUnit, ScrollDelta, ScrollDomain,
        ScrollInput, Vector2,
    };
    use engine::prelude::InputState;

    fn inject(app: &mut App, session: &mut AutomationSession, observation: InputObservation) {
        let input = app
            .world_mut()
            .resource_mut::<InputState>()
            .expect("headless automation fixture should install InputState");
        assert_eq!(
            session
                .inject_normalized(AutomationExecutionMode::NormalizedInput, input, observation,),
            AutomationStepResult::AdmittedOrDelivered
        );
    }

    fn camera(app: &mut App, session: &mut AutomationSession) -> RenderLabCameraObservation {
        let mut adapter = RenderLabAutomationAdapter::new(app);
        match session.query_owner(
            &mut adapter,
            &RenderLabAutomationTarget,
            RenderLabAutomationQuery::Camera,
        ) {
            AutomationStepResult::EffectConfirmed(observation) => observation,
            other => panic!("camera query should confirm product state, got {other:?}"),
        }
    }

    #[test]
    fn normalized_left_drag_orbits_headlessly() {
        let mut app = build_headless_automation_app();
        let mut session =
            AutomationSession::new(AutomationSessionId::new(1), InputSourceId::new(10_001));
        inject(
            &mut app,
            &mut session,
            InputObservation::PointerButton(PointerButtonInput {
                button: PointerButton::Left,
                state: DigitalState::Pressed,
            }),
        );
        inject(
            &mut app,
            &mut session,
            InputObservation::RelativeMotion {
                delta: Vector2::new(10.0, -5.0),
                unit: RelativeMotionUnit::BackendDeviceUnits,
            },
        );

        app = app
            .run_for_frames(1)
            .expect("Render Lab automation orbit frame should run");
        let observed = camera(&mut app, &mut session);
        assert_eq!(observed.yaw_radians, 0.1);
        assert_eq!(observed.pitch_radians, 0.05);
        assert_eq!(observed.pan, [0.0, 0.0]);
    }

    #[test]
    fn normalized_middle_drag_pans_headlessly() {
        let mut app = build_headless_automation_app();
        let mut session =
            AutomationSession::new(AutomationSessionId::new(2), InputSourceId::new(10_002));
        inject(
            &mut app,
            &mut session,
            InputObservation::PointerButton(PointerButtonInput {
                button: PointerButton::Middle,
                state: DigitalState::Pressed,
            }),
        );
        inject(
            &mut app,
            &mut session,
            InputObservation::RelativeMotion {
                delta: Vector2::new(10.0, -5.0),
                unit: RelativeMotionUnit::BackendDeviceUnits,
            },
        );

        app = app
            .run_for_frames(1)
            .expect("Render Lab automation pan frame should run");
        let observed = camera(&mut app, &mut session);
        assert_eq!(observed.yaw_radians, 0.0);
        assert_eq!(observed.pan, [0.1, 0.05]);
    }

    #[test]
    fn normalized_scroll_zooms_headlessly() {
        let mut app = build_headless_automation_app();
        let mut session =
            AutomationSession::new(AutomationSessionId::new(3), InputSourceId::new(10_003));
        inject(
            &mut app,
            &mut session,
            InputObservation::Scroll(ScrollInput {
                delta: ScrollDelta::vertical_only(1.0),
                domain: ScrollDomain::Unspecified,
                phase: None,
            }),
        );

        app = app
            .run_for_frames(1)
            .expect("Render Lab automation zoom frame should run");
        let observed = camera(&mut app, &mut session);
        assert!(observed.distance < 3.0);
        assert_eq!(observed.pan, [0.0, 0.0]);
    }

    #[test]
    fn recorded_normalized_trace_replays_to_the_same_camera_state() {
        let recorded_source = InputSourceId::new(10_100);
        let mut recording = build_headless_automation_app();
        recording.add_plugin(AutomationInputTracePlugin);
        recording
            .start_automation_input_trace()
            .expect("Render Lab trace should start");
        let mut recording_session =
            AutomationSession::new(AutomationSessionId::new(100), recorded_source);

        inject(
            &mut recording,
            &mut recording_session,
            InputObservation::PointerButton(PointerButtonInput {
                button: PointerButton::Left,
                state: DigitalState::Pressed,
            }),
        );
        inject(
            &mut recording,
            &mut recording_session,
            InputObservation::RelativeMotion {
                delta: Vector2::new(10.0, -5.0),
                unit: RelativeMotionUnit::BackendDeviceUnits,
            },
        );
        recording = recording
            .run_for_frames(1)
            .expect("recorded orbit frame should run");

        inject(
            &mut recording,
            &mut recording_session,
            InputObservation::PointerButton(PointerButtonInput {
                button: PointerButton::Left,
                state: DigitalState::Released,
            }),
        );
        recording = recording
            .run_for_frames(1)
            .expect("recorded orbit release frame should run");
        recording = recording
            .run_for_frames(1)
            .expect("recorded idle frame should run");

        inject(
            &mut recording,
            &mut recording_session,
            InputObservation::PointerButton(PointerButtonInput {
                button: PointerButton::Middle,
                state: DigitalState::Pressed,
            }),
        );
        inject(
            &mut recording,
            &mut recording_session,
            InputObservation::RelativeMotion {
                delta: Vector2::new(6.0, -4.0),
                unit: RelativeMotionUnit::BackendDeviceUnits,
            },
        );
        recording = recording
            .run_for_frames(1)
            .expect("recorded pan frame should run");

        inject(
            &mut recording,
            &mut recording_session,
            InputObservation::PointerButton(PointerButtonInput {
                button: PointerButton::Middle,
                state: DigitalState::Released,
            }),
        );
        recording = recording
            .run_for_frames(1)
            .expect("recorded pan release frame should run");

        inject(
            &mut recording,
            &mut recording_session,
            InputObservation::Scroll(ScrollInput {
                delta: ScrollDelta::vertical_only(1.0),
                domain: ScrollDomain::Unspecified,
                phase: None,
            }),
        );
        recording = recording
            .run_for_frames(1)
            .expect("recorded zoom frame should run");

        let recorded_camera = camera(&mut recording, &mut recording_session);
        let trace = recording
            .stop_automation_input_trace()
            .expect("Render Lab trace should stop");
        assert!(trace.trailing_groups().is_empty());
        assert_eq!(trace.frames().len(), 6);
        assert!(trace.frames()[2].groups().is_empty());

        let mut replay = build_headless_automation_app();
        let source_map =
            AutomationInputReplaySourceMap::new([(recorded_source, InputSourceId::new(20_100))]);
        let report = replay.replay_automation_input_trace(
            &trace,
            &source_map,
            AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
        );
        assert_eq!(report.outcome(), AutomationInputReplayOutcome::Completed);
        assert_eq!(report.completed_frames(), 6);

        let mut query_session =
            AutomationSession::new(AutomationSessionId::new(101), InputSourceId::new(30_100));
        let replayed_camera = camera(&mut replay, &mut query_session);
        assert_eq!(replayed_camera, recorded_camera);

        replay
            .teardown_automation_input_replay()
            .expect("Render Lab replay teardown should clean replay-owned input");
    }
}
