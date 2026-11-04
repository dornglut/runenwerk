use super::*;

#[test]
fn submission_completion_does_not_retire_independent_readback_interest() {
    assert!(gpu_execution_needs_progress(1, 1));
    assert!(gpu_execution_needs_progress(0, 1));
    assert!(!gpu_execution_needs_progress(0, 0));
}

#[test]
fn buffered_facts_are_delivered_once_and_retained_history_is_inert() {
    let mut renderer = Renderer::new();
    let publication = RunenUiPublicationId::try_from_raw(7).unwrap();
    renderer
        .pending_gpu_observation_output
        .runenui_accepted
        .push(publication);
    renderer
        .pending_gpu_observation_output
        .runenui_presented
        .push(publication);
    renderer
        .pending_gpu_observation_output
        .runenui_rejected
        .push((publication, "test failure"));
    renderer
        .pending_gpu_observation_output
        .timing_evidence
        .push(RenderPassTimingEvidence::gpu_sample(
            Some(12),
            Some(3),
            "flow",
            "pass",
            "compute",
            0.5,
        ));
    assert!(renderer.gpu_delivery_pending());
    renderer.publish_progressed_gpu_observations();
    assert!(renderer.gpu_delivery_pending());
    let (accepted, completed, rejected) = renderer.take_runenui_terminal_publications();
    assert_eq!(accepted, vec![publication]);
    assert_eq!(completed, vec![publication]);
    assert_eq!(rejected, vec![(publication, "test failure")]);
    let retained_history = renderer.last_gpu_pass_timing_evidence().to_vec();
    renderer.clear_published_gpu_pass_timing_evidence();
    assert_eq!(retained_history.len(), 1);
    renderer.publish_progressed_gpu_observations();
    assert!(!renderer.gpu_delivery_pending());
    let (accepted, completed, rejected) = renderer.take_runenui_terminal_publications();
    assert!(accepted.is_empty() && completed.is_empty() && rejected.is_empty());
}

/// Opt in on a quiet native desktop in a separate filtered test process.
/// Context shutdown here is controlled lifecycle testing, not driver loss.
#[test]
#[cfg(any(target_os = "windows", target_os = "linux"))]
fn native_prepared_readback_affinity_and_shutdown() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_GPU_PROGRESS_SMOKE").is_none() {
        return Ok(());
    }
    use runen_gpu::*;
    use std::time::Duration;
    use winit::event_loop::EventLoop;
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
    let window =
        Arc::new(event_loop.create_window(Window::default_attributes().with_visible(false))?);
    let mut gfx = Gfx::new(window)?;
    let affinity = gfx.gpu_context_affinity();
    gfx.progress_gpu_before_producer_update()?;
    assert_eq!(gfx.early_gpu_progress_affinity, Some(affinity));
    gfx.progress_gpu_without_frame()?;
    assert!(!consume_matching_early_gpu_progress(
        &mut gfx.early_gpu_progress_affinity,
        &affinity
    ));
    let mut ids = GpuWorkResourceIdAllocator::new();
    let buffer = ids.allocate_buffer_handle(GpuBufferDescriptor::ordinary_owned(
        "native.progress.buffer",
        GpuResourceLifetime::Transient,
        GpuReconstruction::SourceBacked,
        64,
        [GpuBufferUsage::CopySource],
        GpuBufferInitialization::Zeroed,
    )?)?;
    let label = GpuResourceLabel::new("native.progress.readback")?;
    let provenance = GpuResourceProvenance::new(label.clone(), None, None);
    let readback = GpuReadbackId::allocate()?;
    let mut work = GpuWorkFragmentBuilder::new(label.clone(), provenance.clone());
    work.declare_resource(buffer.clone().into())?;
    work.add_node(
        label.clone(),
        GpuWorkOperation::Readback(GpuReadbackOperation::new(
            GpuBufferRegion::whole(&buffer)?.into(),
            readback,
        )?),
        [],
        GpuCapabilityRequirements::new(),
        GpuExecutionPreference::TransferPreferred,
        provenance,
    )?;
    let graph = gfx
        .ctx
        .context()
        .prepare_work_graph(label, [work.finish()?])?;
    let prepared = pollster::block_on(gfx.ctx.context().prepare_submission(graph))?;
    assert_eq!(gfx.execution_stats().prepared_submissions(), 1);
    assert!(
        !gfx.native_gpu_progress_pending(),
        "prepared-only work cannot wake the Host"
    );
    let submission = gfx
        .ctx
        .context()
        .submit_prepared(prepared)
        .map_err(|e| anyhow::anyhow!("submission rejected: {e:?}"))?;
    assert!(gfx.native_gpu_progress_pending());
    assert_eq!(
        gfx.ctx.context().begin_shutdown(),
        GpuExecutionLifecycleState::ShuttingDown
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while gfx.native_gpu_progress_pending() {
        gfx.progress_gpu_without_frame()?;
        anyhow::ensure!(
            Instant::now() < deadline,
            "native readback/shutdown did not drain"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(matches!(
        submission.status(),
        GpuSubmissionStatus::Completed
    ));
    assert!(matches!(
        submission.readback(readback).unwrap().status(),
        GpuReadbackStatus::Ready(_)
    ));
    assert_eq!(
        gfx.ctx.context().execution_lifecycle_state(),
        GpuExecutionLifecycleState::Closed
    );
    println!(
        "native_gpu_progress=pass adapter={:?} stale_early_marker_cleared=true prepared_only_idle=true readback_terminal=true controlled_shutdown_drained=true",
        gfx.adapter_facts().diagnostic_name()
    );
    Ok(())
}
