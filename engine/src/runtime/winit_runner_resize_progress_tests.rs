use super::tests::runner_with_frame_pacing;
use super::*;
use crate::runtime::schedules::{FrameEnd, RenderSubmit};
use crate::runtime::{Res, ResMut};
use std::time::Duration;

#[derive(Debug, Default, runen_ecs::Resource)]
struct ResizeObservations {
    frame_ends: usize,
    widths: Vec<u32>,
    window_ids: Vec<NativeWindowId>,
}

fn observe_frame_end(
    journal: Res<PlatformWindowEventQueueResource>,
    mut observations: ResMut<ResizeObservations>,
) {
    observations.frame_ends += 1;
    for event in journal.events() {
        observations.window_ids.push(event.native_window_id);
        if let PlatformEvent::Resized { width, .. } = event.event {
            observations.widths.push(width);
        }
    }
}

fn runner_for_progress(policy: FramePacingPolicyResource) -> WinitRunner {
    let mut runner = runner_with_frame_pacing(policy);
    runner.state.world.insert_resource(ResizeObservations::default());
    runner
        .state
        .scheduler
        .add_systems(FrameEnd, observe_frame_end)
        .expect("install canonical FrameEnd observer");
    runner
}

fn queue_resize(runner: &mut WinitRunner, width: u32) {
    runner
        .apply_event(PlatformEvent::Resized { width, height: 720 })
        .expect("bounded native callback admission");
    runner.frame_requested = true;
}

fn journal_len(runner: &WinitRunner) -> usize {
    runner
        .state
        .world
        .resource::<PlatformWindowEventQueueResource>()
        .unwrap()
        .events()
        .len()
}

#[test]
fn blocked_about_to_wait_can_process_more_than_4096_original_resize_occurrences() {
    // No about_to_wait is invoked: every attempted frame is reached via the
    // same native Host operation offered by Windows' resize callback.
    for policy in [
        FramePacingPolicyResource::on_demand(),
        FramePacingPolicyResource::continuous_capped(30),
        FramePacingPolicyResource::continuous_capped(60),
        FramePacingPolicyResource::continuous_uncapped(),
    ] {
        let mut runner = runner_for_progress(policy);
        let start = Instant::now();
        let mut frame_starts = Vec::new();
        const EVENT_COUNT: u32 = 4_608;
        for index in 0..EVENT_COUNT {
            queue_resize(&mut runner, index + 1);
            if (index + 1) % 48 == 0 {
                let now = start + Duration::from_millis(u64::from(index) + 1);
                if runner
                    .run_requested_frame(now, true, |_| Ok(()))
                    .expect("canonical resize frame")
                {
                    frame_starts.push(now);
                    assert_eq!(journal_len(&runner), 0, "successful FrameEnd retires the journal");
                }
            }
        }
        let observed = runner.state.world.resource::<ResizeObservations>().unwrap();
        assert_eq!(observed.widths, (1..=EVENT_COUNT).collect::<Vec<_>>());
        assert_eq!(observed.frame_ends, frame_starts.len());
        assert!(observed.frame_ends > 0);
        assert!(journal_len(&runner) == 0);
        assert!(!runner.frame_requested);
        if let Some(interval) = policy.target_frame_interval() {
            assert!(frame_starts.windows(2).all(|w| w[1].duration_since(w[0]) >= interval));
        }
    }
}

#[test]
fn capped_frame_eligibility_is_shared_by_normal_and_resize_opportunities() {
    for fps in [30, 60] {
        let policy = FramePacingPolicyResource::continuous_capped(fps);
        let interval = policy.target_frame_interval().unwrap();
        let start = Instant::now();
        let mut runner = runner_for_progress(policy);
        runner.last_primary_redraw_at = Some(start);
        queue_resize(&mut runner, 1600);

        assert!(!runner.run_requested_frame(start + interval / 2, true, |_| Ok(())).unwrap());
        assert_eq!(journal_len(&runner), 1);
        assert!(runner.frame_requested);
        assert!(runner.run_requested_frame(start + interval, true, |_| Ok(())).unwrap());
        assert_eq!(journal_len(&runner), 0);
        assert!(!runner.run_requested_frame(start + interval, true, |_| Ok(())).unwrap());
        assert_eq!(
            runner.state.world.resource::<ResizeObservations>().unwrap().frame_ends,
            1
        );
    }
}

#[test]
fn on_demand_idle_and_unrealized_primary_do_not_run_frames() {
    let mut runner = runner_for_progress(FramePacingPolicyResource::on_demand());
    let now = Instant::now();
    assert!(!runner.run_requested_frame(now, true, |_| Ok(())).unwrap());
    assert!(!runner.run_requested_frame(now, false, |_| Ok(())).unwrap());
    queue_resize(&mut runner, 1680);
    assert!(!runner.run_requested_frame(now, false, |_| Ok(())).unwrap());
    assert_eq!(journal_len(&runner), 1);
    assert!(runner.run_requested_frame(now, true, |_| Ok(())).unwrap());
    assert!(!runner.run_requested_frame(now, true, |_| Ok(())).unwrap());
    assert_eq!(runner.state.world.resource::<ResizeObservations>().unwrap().frame_ends, 1);
}

#[test]
fn nested_frame_request_is_not_run_reentrantly_or_discarded() {
    let mut runner = runner_for_progress(FramePacingPolicyResource::on_demand());
    let now = Instant::now();
    queue_resize(&mut runner, 1400);
    assert!(
        runner
            .run_requested_frame(now, true, |runner| {
                assert!(runner.frame_in_progress);
                runner.frame_requested = true;
                assert!(!runner.run_requested_frame(now, true, |_| Ok(()))?);
                assert!(runner.frame_requested);
                Ok(())
            })
            .unwrap()
    );
    assert!(!runner.frame_in_progress);
    assert!(runner.frame_requested);
    assert!(runner.run_requested_frame(now, true, |_| Ok(())).unwrap());
    assert_eq!(runner.state.world.resource::<ResizeObservations>().unwrap().frame_ends, 2);
}

#[test]
fn frame_end_failure_never_retires_the_native_journal() {
    fn failing_frame_end() -> Result<()> {
        anyhow::bail!("injected FrameEnd failure")
    }
    let mut runner = runner_for_progress(FramePacingPolicyResource::on_demand());
    runner.state.scheduler.add_systems(FrameEnd, failing_frame_end).unwrap();
    queue_resize(&mut runner, 1700);
    let error = runner
        .run_requested_frame(Instant::now(), true, |_| Ok(()))
        .expect_err("FrameEnd failure must propagate");
    assert!(format!("{error:#}").contains("injected FrameEnd failure"));
    assert_eq!(journal_len(&runner), 1);
    assert!(!runner.frame_in_progress);
}

#[test]
fn focus_continuity_and_multiwindow_occurrences_remain_ordered_through_frame_end() {
    let mut runner = runner_for_progress(FramePacingPolicyResource::on_demand());
    runner.state.world.insert_resource(InputState::new());
    let primary = NativeWindowId::primary();
    let secondary = runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .unwrap()
        .request_window("second", (640, 480))
        .native_window_id;
    runner.input_adapter.window_context(primary, winit::event::DeviceId::dummy()).unwrap();
    runner.apply_window_focus_event(primary, false).unwrap();
    runner
        .apply_event_for_native_window(
            secondary,
            PlatformEvent::Resized { width: 1920, height: 1080 },
        )
        .unwrap();
    {
        let journal = runner.state.world.resource::<PlatformWindowEventQueueResource>().unwrap();
        assert_eq!(journal.events().len(), 3);
        assert!(matches!(journal.events()[0].event, PlatformEvent::Focused { focused: false }));
        assert!(matches!(journal.events()[1].event, PlatformEvent::InputContinuityLost { .. }));
        assert_eq!(journal.events()[2].native_window_id, secondary);
    }
    runner.frame_requested = true;
    assert!(runner.run_requested_frame(Instant::now(), true, |_| Ok(())).unwrap());
    assert_eq!(journal_len(&runner), 0);
    let observed = runner.state.world.resource::<ResizeObservations>().unwrap();
    assert_eq!(observed.window_ids, vec![primary, primary, secondary]);
    assert_eq!(observed.widths, vec![1920]);
}

#[test]
fn slow_render_schedule_runs_only_once_per_admitted_frame() {
    fn slow_submit() {
        std::thread::sleep(Duration::from_millis(2));
    }
    let mut runner = runner_for_progress(FramePacingPolicyResource::continuous_capped(60));
    runner.state.scheduler.add_systems(RenderSubmit, slow_submit).unwrap();
    queue_resize(&mut runner, 1350);
    let now = Instant::now();
    assert!(runner.run_requested_frame(now, true, |_| Ok(())).unwrap());
    assert!(!runner.run_requested_frame(now, true, |_| Ok(())).unwrap());
    assert_eq!(runner.state.world.resource::<ResizeObservations>().unwrap().frame_ends, 1);
    assert_eq!(journal_len(&runner), 0);
}
