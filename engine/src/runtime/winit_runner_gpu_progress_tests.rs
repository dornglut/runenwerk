//! Native scheduling proof and process-isolated, attached lifecycle proof.

use super::tests::runner_with_frame_pacing;
use super::*;

#[test]
fn readback_only_interest_survives_runner_turns_and_retires_after_delivery() -> Result<()> {
    use runen_gpu::{
        GpuCapabilityProfile, GpuContext, GpuContextDescriptor, GpuContextRequestErrorCategory,
    };
    // Real affinity; the execution snapshots and delayed delivery are controlled
    // inputs. This is a deterministic Host test, not physical readback overlap.
    let context = match pollster::block_on(GpuContext::request(GpuContextDescriptor::new(
        GpuCapabilityProfile::ComputeBaseline.requirements(),
    ))) {
        Ok(context) => context,
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable => {
            eprintln!("readback-only runner affinity unavailable: no adapter");
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    #[derive(Default)]
    struct Pending {
        submissions: usize,
        readbacks: usize,
        delivered: usize,
    }
    impl runen_ecs::Resource for Pending {}
    let affinity = context.affinity();
    let snapshot = |world: &runen_ecs::World| {
        let state = world.resource::<Pending>().unwrap();
        Some((affinity, state.submissions != 0 || state.readbacks != 0))
    };
    let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
    runner.state.world.insert_resource(Pending {
        submissions: 1,
        readbacks: 1,
        delivered: 0,
    });
    let now = Instant::now();
    runner.gpu_progress_deadline = retain_gpu_progress_deadline(None, Some(affinity), now)?;
    runner.run_gpu_progress_turn_with(snapshot, |world, frame| {
        assert_eq!(frame, 0);
        world.resource_mut::<Pending>()?.submissions = 0;
        Ok(())
    })?;
    let readback_deadline = runner.gpu_progress_deadline.unwrap();
    let pending = runner.state.world.resource::<Pending>()?;
    assert_eq!(
        (pending.submissions, pending.readbacks, pending.delivered),
        (0, 1, 0)
    );
    // Repeated unrelated scheduling preserves even an already-due deadline.
    for _ in 0..3 {
        runner.gpu_progress_deadline = retain_gpu_progress_deadline(
            runner.gpu_progress_deadline,
            runner.gpu_progress_interest_from(snapshot(&runner.state.world)),
            readback_deadline.at + Duration::from_secs(1),
        )?;
        assert_eq!(
            runner.gpu_progress_deadline.unwrap().at,
            readback_deadline.at
        );
    }
    runner.run_gpu_progress_turn_with(snapshot, |world, _| {
        let pending = world.resource_mut::<Pending>()?;
        assert_eq!((pending.submissions, pending.readbacks), (0, 1));
        pending.readbacks = 0;
        pending.delivered += 1;
        Ok(())
    })?;
    assert!(runner.gpu_progress_deadline.is_none());
    runner.run_gpu_progress_turn_with(snapshot, |_, _| panic!("drained work delivered twice"))?;
    assert_eq!(runner.state.world.resource::<Pending>()?.delivered, 1);
    assert_eq!(runner.ordinary_frames, 0);
    assert!(!runner.frame_requested);
    runner.shutdown_native()?;
    assert!(runner.gpu_progress_deadline.is_none());
    println!(
        "readback_only_runner=pass simulated_snapshots=true retained_due_deadline=true terminal_delivery_once=true ordinary_frames=0"
    );
    Ok(())
}

#[cfg(target_os = "windows")]
mod attached {
    use super::*;
    use crate::app::{App, AppNativeHostExt};
    use crate::plugins::render::host::RenderSurfaceId;
    use crate::plugins::render::{Gfx, RenderFlow, RenderFrameProducerId};
    use crate::plugins::ui::{
        AppUiExt, UiRuntimeFontConfiguration, UiRuntimePresentationBinding, UiTypedScreenId,
    };
    use crate::plugins::{RenderPlugin, UiPlugin, default_plugins};
    use crate::prelude::AppRenderExt;
    use crate::runtime::{NativeWindowHook, NativeWindowHookRegistryResource};
    use runen_gpu::{
        GpuReadbackStatus, GpuSubmission, GpuSubmissionFailureKind, GpuSubmissionStatus,
    };
    use std::{cell::RefCell, rc::Rc};
    use winit::platform::windows::EventLoopBuilderExtWindows;

    #[derive(Default)]
    struct Evidence {
        submission: Option<GpuSubmission>,
        detached: bool,
        callbacks: usize,
        close_requested: bool,
    }

    struct TextApp;
    impl runenui_core::UiApp for TextApp {
        type State = ();
        type Action = ();
        type HostProtocol = runenui_core::NoHostProtocol;
        fn root(_: &()) -> impl runenui_core::View<()> {
            runenui_core::text("attached pending GPU close")
        }
        fn update(
            _: &mut (),
            _: (),
        ) -> impl runenui_core::IntoUpdateOutput<(), Self::HostProtocol> {
        }
    }

    struct CloseBeforeObservation {
        evidence: Rc<RefCell<Evidence>>,
        slot: UiRuntimeSlotId,
        destroy_context: bool,
    }
    impl NativeWindowHook for CloseBeforeObservation {
        fn name(&self) -> &'static str {
            "attached pending GPU teardown"
        }
        fn attach(&mut self, window: &Window, _: &mut runen_ecs::World) -> Result<()> {
            // A genuinely attached native surface, without cursor contamination.
            window.set_visible(false);
            Ok(())
        }
        fn frame(&mut self, _: &Window, world: &mut runen_ecs::World) -> Result<()> {
            world
                .resource_mut::<Gfx>()?
                .set_gpu_frame_terminal_delivery(true);
            Ok(())
        }
        fn gpu_progress(&mut self, world: &mut runen_ecs::World) -> Result<bool> {
            let mut evidence = self.evidence.borrow_mut();
            evidence.callbacks += 1;
            anyhow::ensure!(!evidence.close_requested, "callback after close intent");
            let gfx = world.resource::<Gfx>()?;
            let pending = gfx.pending_gpu_frame_submissions();
            anyhow::ensure!(
                pending.len() == 1,
                "expected exact first accepted submission"
            );
            let submission = pending[0].submission().clone();
            anyhow::ensure!(
                matches!(submission.status(), GpuSubmissionStatus::Accepted),
                "submission observed terminal before controlled close"
            );
            anyhow::ensure!(
                gfx.execution_stats().in_flight_submissions() == 1,
                "accepted GPU obligation missing"
            );
            anyhow::ensure!(
                world
                    .resource::<UiRuntimePresentationAssociationsResource>()?
                    .displayed_for_native_recipient(self.slot, NativeWindowId::primary())
                    .is_none(),
                "accepted-only work was displayed"
            );
            println!(
                "attached_teardown_pending submission={} affinity={:?} readbacks={} mode={}",
                submission.id(),
                submission.affinity(),
                gfx.execution_stats().pending_readbacks(),
                if self.destroy_context {
                    "context-drop"
                } else {
                    "native-close"
                }
            );
            evidence.submission = Some(submission);
            if self.destroy_context {
                drop(
                    world
                        .remove_resource::<Gfx>()
                        .context("missing controlled context owner")?,
                );
            }
            let primary = world
                .resource_mut::<WindowStateRegistryResource>()?
                .record_mut(NativeWindowId::primary())
                .context("missing attached primary")?;
            primary.request_close();
            primary.approve_close();
            evidence.close_requested = true;
            Ok(false)
        }
        fn detach(&mut self, world: &mut runen_ecs::World) -> Result<()> {
            anyhow::ensure!(
                world.resource::<UiRuntimeSlotsResource>()?.is_empty(),
                "native slots survived teardown"
            );
            anyhow::ensure!(
                world.resource::<NativeWakeTransport>()?.notify().is_err(),
                "late wake accepted after close"
            );
            anyhow::ensure!(
                world
                    .resource::<UiRuntimePresentationAssociationsResource>()?
                    .displayed_for_native_recipient(self.slot, NativeWindowId::primary())
                    .is_none(),
                "closed publication acquired displayed input"
            );
            self.evidence.borrow_mut().detached = true;
            Ok(())
        }
    }

    /// Run in its own process with RUNENWERK_NATIVE_PENDING_TEARDOWN=close or
    /// context-drop. Context destruction is controlled, not physical device loss.
    #[test]
    fn attached_on_demand_close_preserves_exact_pending_terminal_outcomes() -> Result<()> {
        let Ok(mode) = std::env::var("RUNENWERK_NATIVE_PENDING_TEARDOWN") else {
            return Ok(());
        };
        anyhow::ensure!(
            matches!(mode.as_str(), "close" | "context-drop"),
            "unknown teardown mode"
        );
        let mut app = App::new();
        app.with_frame_pacing(FramePacingPolicyResource::on_demand());
        app.add_plugins(default_plugins());
        app.add_plugin(UiPlugin);
        app.add_plugin(RenderPlugin);
        let fonts = UiRuntimeFontConfiguration::new(vec![
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../assets/fonts/JetBrainsMono-Regular.ttf"
            ))
            .to_vec(),
        ])
        .with_generic_mapping(
            runenui_core::GenericFontFamily::SansSerif,
            vec![runenui_core::FontFamilyName::new("JetBrains Mono")?],
        );
        let slot = app
            .ui()
            .mount_app::<TextApp>(UiTypedScreenId::new("native.pending.teardown"), (), &fonts)
            .slot_id()
            .context("text app failed to mount")?;
        app.ui().bind_native_primary_input(slot)?;
        app.world_mut()
            .resource_mut::<UiRuntimePresentationAssociationsResource>()?
            .bind(UiRuntimePresentationBinding::new(
                slot,
                RenderFrameProducerId::try_from_raw(993)?,
                RenderSurfaceId::primary(),
                runenui_core::StyleEnvironment::default(),
            ))?;
        let flow = RenderFlow::new("native.pending.teardown")
            .with_surface_color()?
            .fullscreen_pass("clear")
            .main_surface_only()
            .clear_color([0.1, 0.1, 0.1, 1.0])
            .write_surface_color()?
            .finish()
            .builtin_ui_composite_pass("ui")?
            .main_surface_only()
            .finish()
            .present_pass("present")?
            .main_surface_only()
            .surface_color()?
            .finish()
            .validate()?;
        app.add_render_flow(flow);
        let evidence = Rc::new(RefCell::new(Evidence::default()));
        app.init_resource::<NativeWindowHookRegistryResource>();
        app.world_mut()
            .resource_mut::<NativeWindowHookRegistryResource>()?
            .register_hook(CloseBeforeObservation {
                evidence: evidence.clone(),
                slot,
                destroy_context: mode == "context-drop",
            });
        let mut state = app.into_windowed_state();
        install_native_window_provider_resources(&mut state.world);
        let event_loop = EventLoop::<NativeHostEvent>::with_user_event()
            .with_any_thread(true)
            .build()?;
        let transport = NativeWakeTransport::new(event_loop.create_proxy());
        state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()?
            .attach_native_transport(transport.clone());
        state.world.insert_resource(transport.clone());
        event_loop.set_control_flow(ControlFlow::Wait);
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state = state;
        event_loop.run_app(&mut runner)?;
        if let Some(error) = runner.fatal_error.take() {
            return Err(error);
        }
        runner.shutdown_native()?;
        assert!(runner.shutdown_done);
        assert!(runner.gpu_progress_deadline.is_none());
        assert!(runner.gpu_progress_interest().is_none());
        assert!(!runner.frame_requested);
        assert_eq!(runner.ordinary_frames, 1);
        assert!(transport.notify().is_err());
        let submission = evidence
            .borrow()
            .submission
            .clone()
            .context("no accepted GPU work captured")?;
        // The production runner destructor owns hook detachment and final
        // World/context destruction, after shutdown has closed native slots.
        drop(runner);
        let evidence = evidence.borrow();
        assert!(evidence.detached && evidence.close_requested);
        assert_eq!(evidence.callbacks, 1);
        // Close cancels observation opportunities; it does not manufacture GPU
        // completion. Dropping the remaining context owner must terminalize the
        // retained public handles even though the native loop no longer polls.
        assert!(
            matches!(submission.status(), GpuSubmissionStatus::Failed(ref failure) if failure.kind() == GpuSubmissionFailureKind::ContextDropped)
        );
        for readback in submission.readbacks() {
            assert!(
                matches!(readback.status(), GpuReadbackStatus::Failed(ref failure) if failure.kind() == GpuSubmissionFailureKind::ContextDropped)
            );
        }
        let terminal = format!("{:?}", submission.status());
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(terminal, format!("{:?}", submission.status()));
        println!(
            "attached_teardown=pass mode={mode} submission={} terminal=ContextDropped readbacks={} frames=1 continuations=0 deadline_retired=true wake_closed=true scanout=unobserved physical_device_loss=unproven",
            submission.id(),
            submission.readbacks().len()
        );
        Ok(())
    }
}
