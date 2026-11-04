use super::tests::runner_with_frame_pacing;
use super::*;

#[test]
fn no_gfx_progress_turn_never_runs_an_ordinary_lifecycle_or_requests_a_frame() {
    let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
    fn forbidden() {
        panic!("GPU-only turn entered an ordinary schedule");
    }
    runner
        .state
        .scheduler
        .add_systems(crate::runtime::Update, forbidden)
        .unwrap();
    runner
        .state
        .scheduler
        .add_systems(crate::runtime::RenderPrepare, forbidden)
        .unwrap();
    runner
        .state
        .scheduler
        .add_systems(crate::runtime::RenderSubmit, forbidden)
        .unwrap();
    runner
        .state
        .scheduler
        .add_systems(crate::runtime::FrameEnd, forbidden)
        .unwrap();
    for _ in 0..32 {
        assert!(!runner.progress_gpu_if_due(Instant::now()).unwrap());
        assert!(!runner.frame_requested);
        assert!(runner.gpu_progress_deadline.is_none());
    }
}

/// Real affinities, deterministic scheduling time. This is not physical loss.
#[test]
#[cfg(any(target_os = "windows", target_os = "linux"))]
fn native_deadline_retirement_and_context_replacement() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_GPU_DEADLINE_SMOKE").is_none() {
        return Ok(());
    }
    use crate::plugins::render::Gfx;
    use std::sync::Arc;
    let mut builder = EventLoop::builder();
    #[cfg(target_os = "windows")]
    {
        use winit::platform::windows::EventLoopBuilderExtWindows;
        builder.with_any_thread(true);
    }
    #[cfg(target_os = "linux")]
    {
        use winit::platform::x11::EventLoopBuilderExtX11;
        builder.with_any_thread(true).with_x11();
    }
    let event_loop = builder.build()?;
    #[allow(deprecated)]
    let first = Gfx::new(Arc::new(
        event_loop.create_window(Window::default_attributes().with_visible(false))?,
    ))?;
    #[allow(deprecated)]
    let second = Gfx::new(Arc::new(
        event_loop.create_window(Window::default_attributes().with_visible(false))?,
    ))?;
    let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
    let now = Instant::now();
    let first_affinity = first.gpu_context_affinity();
    let second_affinity = second.gpu_context_affinity();
    assert_ne!(first_affinity, second_affinity);
    runner.refresh_gpu_progress_deadline(now, Some(first_affinity));
    let deadline = now + NATIVE_GPU_PROGRESS_INTERVAL;
    for step in 0..128 {
        runner
            .refresh_gpu_progress_deadline(now + Duration::from_micros(step), Some(first_affinity));
        assert_eq!(
            runner.gpu_progress_deadline,
            Some((first_affinity, deadline)),
            "native callbacks must not postpone pending progress"
        );
    }
    runner.refresh_gpu_progress_deadline(deadline, Some(second_affinity));
    assert_eq!(
        runner.gpu_progress_deadline,
        Some((second_affinity, deadline + NATIVE_GPU_PROGRESS_INTERVAL))
    );
    runner.refresh_gpu_progress_deadline(deadline, None);
    assert!(runner.gpu_progress_deadline.is_none());
    runner.refresh_gpu_progress_deadline(deadline, Some(second_affinity));
    runner.shutdown_native()?;
    assert!(runner.gpu_progress_deadline.is_none());
    assert!(!runner.frame_requested);
    println!(
        "native_gpu_deadline=pass preserved_across_callbacks=true context_replacement_reset=true drained_and_shutdown_retired=true"
    );
    Ok(())
}
