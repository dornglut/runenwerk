use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use engine::automation::{
    AppAutomationInputReplayExt, AppAutomationInputTraceExt, AutomationExecutionMode,
    AutomationInputReplayOutcome, AutomationInputReplaySourceMap,
    AutomationInputReplayStateAssumption, AutomationInputTracePlugin,
    AutomationInputTraceRecordingWitness, AutomationOwnerAdapter, AutomationSession,
    AutomationSessionId, AutomationStepResult, DigitalState, InputObservation, InputSourceId,
    MAX_ARTIFACT_BYTES, PointerButton, PointerButtonInput, RelativeMotionUnit, ScrollDelta,
    ScrollDomain, ScrollInput, Vector2, export_automation_input_trace_v1,
    import_automation_input_trace_v1,
};
use engine::prelude::InputState;
use runenwerk_render_lab::automation::{
    RenderLabAutomationAdapter, RenderLabAutomationQuery, RenderLabAutomationTarget,
    RenderLabCameraObservation, build_headless_automation_app,
};

static NEXT_PATH: AtomicU64 = AtomicU64::new(0);

fn inject(
    app: &mut engine::prelude::App,
    session: &mut AutomationSession,
    observation: InputObservation,
) {
    let input = app
        .world_mut()
        .resource_mut::<InputState>()
        .expect("headless automation fixture should install InputState");
    assert_eq!(
        session.inject_normalized(AutomationExecutionMode::NormalizedInput, input, observation),
        AutomationStepResult::AdmittedOrDelivered
    );
}

fn query_camera(app: &mut engine::prelude::App) -> RenderLabCameraObservation {
    let mut adapter = RenderLabAutomationAdapter::new(app);
    adapter
        .query(&RenderLabAutomationTarget, RenderLabAutomationQuery::Camera)
        .expect("Render Lab camera query should succeed")
}

fn persisted_trace_fixture() -> (String, RenderLabCameraObservation) {
    let recorded_source = InputSourceId::new(10_100);
    let mut app = build_headless_automation_app();
    app.add_plugin(AutomationInputTracePlugin);
    app.start_automation_input_trace()
        .expect("trace should start");
    let mut session = AutomationSession::new(AutomationSessionId::new(100), recorded_source);

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
    app = app.run_for_frames(1).expect("orbit frame should run");

    inject(
        &mut app,
        &mut session,
        InputObservation::PointerButton(PointerButtonInput {
            button: PointerButton::Left,
            state: DigitalState::Released,
        }),
    );
    app = app
        .run_for_frames(1)
        .expect("orbit release frame should run");
    app = app.run_for_frames(1).expect("idle frame should run");

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
            delta: Vector2::new(6.0, -4.0),
            unit: RelativeMotionUnit::BackendDeviceUnits,
        },
    );
    app = app.run_for_frames(1).expect("pan frame should run");

    inject(
        &mut app,
        &mut session,
        InputObservation::PointerButton(PointerButtonInput {
            button: PointerButton::Middle,
            state: DigitalState::Released,
        }),
    );
    app = app.run_for_frames(1).expect("pan release frame should run");

    inject(
        &mut app,
        &mut session,
        InputObservation::Scroll(ScrollInput {
            delta: ScrollDelta::vertical_only(1.0),
            domain: ScrollDomain::Unspecified,
            phase: None,
        }),
    );
    app = app.run_for_frames(1).expect("zoom frame should run");

    let recorded_camera = query_camera(&mut app);
    let trace = app
        .stop_automation_input_trace()
        .expect("trace should stop");
    assert_eq!(trace.frames().len(), 6);
    assert!(trace.frames()[2].groups().is_empty());
    assert!(trace.trailing_groups().is_empty());

    let encoded = export_automation_input_trace_v1(
        &trace,
        AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart,
        None,
    )
    .expect("trace should persist");

    (encoded, recorded_camera)
}

fn direct_replay_camera(encoded: &str) -> RenderLabCameraObservation {
    let imported = import_automation_input_trace_v1(encoded.as_bytes())
        .expect("persisted trace should import");
    let recorded_source = imported.trace().frames()[0].groups()[0].context.source;
    let source_map =
        AutomationInputReplaySourceMap::new([(recorded_source, InputSourceId::new(20_100))]);
    let mut app = build_headless_automation_app();
    let report = app.replay_automation_input_trace(
        imported.trace(),
        &source_map,
        AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    );
    assert_eq!(report.outcome(), AutomationInputReplayOutcome::Completed);
    assert_eq!(report.completed_frames(), 6);
    let camera = query_camera(&mut app);
    app.teardown_automation_input_replay()
        .expect("direct replay teardown should succeed");
    camera
}

fn temp_path(label: &str) -> PathBuf {
    let sequence = NEXT_PATH.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "runenwerk-render-lab-a9-{}-{sequence}-{label}.ron",
        std::process::id()
    ))
}

fn run_binary(path: &PathBuf) -> Output {
    Command::new(env!("CARGO_BIN_EXE_runenwerk-render-lab"))
        .arg("--replay-trace")
        .arg(path)
        .output()
        .expect("Render Lab binary should launch")
}

fn write_and_run(label: &str, bytes: &[u8]) -> Output {
    let path = temp_path(label);
    fs::write(&path, bytes).expect("test artifact should write");
    let output = run_binary(&path);
    let _ = fs::remove_file(path);
    output
}

#[test]
fn persisted_trace_replays_through_the_terminal_binary() {
    let (encoded, recorded_camera) = persisted_trace_fixture();
    let direct_camera = direct_replay_camera(&encoded);
    assert_eq!(direct_camera, recorded_camera);

    let output = write_and_run("success", encoded.as_bytes());
    assert!(
        output.status.success(),
        "binary failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("terminal output should be UTF-8");
    let expected = format!(
        "replay completed: frames=6 camera_yaw_radians={} camera_pitch_radians={} camera_distance={} camera_pan_x={} camera_pan_y={}",
        direct_camera.yaw_radians,
        direct_camera.pitch_radians,
        direct_camera.distance,
        direct_camera.pan[0],
        direct_camera.pan[1],
    );
    assert_eq!(stdout.trim(), expected);
}

#[test]
fn terminal_replay_fails_closed_for_invalid_artifacts() {
    let (encoded, _) = persisted_trace_fixture();

    let malformed = write_and_run("malformed", b"this is not RON");
    assert!(!malformed.status.success());

    let wrong_kind_text = encoded.replace(
        "runenwerk.automation.normalized-replay-trace",
        "runenwerk.automation.not-a-replay-trace",
    );
    assert_ne!(wrong_kind_text, encoded);
    let wrong_kind = write_and_run("wrong-kind", wrong_kind_text.as_bytes());
    assert!(!wrong_kind.status.success());

    let future_version_text = encoded.replace("schema_version: 1", "schema_version: 2");
    assert_ne!(future_version_text, encoded);
    let future_version = write_and_run("future-version", future_version_text.as_bytes());
    assert!(!future_version.status.success());

    let release_first_text = encoded.replacen("Pressed", "Released", 1);
    assert_ne!(release_first_text, encoded);
    let release_first = write_and_run("release-first", release_first_text.as_bytes());
    assert!(!release_first.status.success());

    let oversized = vec![b'x'; MAX_ARTIFACT_BYTES + 1];
    let oversized_result = write_and_run("oversized", &oversized);
    assert!(!oversized_result.status.success());

    let missing = run_binary(&temp_path("missing"));
    assert!(!missing.status.success());
}
