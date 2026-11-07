//! Real Linux/X11 Winit request-ingress and terminal-receipt proof.
#![cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]

#[cfg(target_os = "linux")]
mod linux {
    use anyhow::{Context, Result, ensure};
    use engine::app::{App, AppNativeHostExt};
    use engine::plugins::{RenderPlugin, default_plugins};
    use engine::plugins::render::Gfx;
    use engine::plugins::render::host::{
        RenderSurfaceLifecycleState, RenderSurfaceRegistryResource,
    };
    use engine::runtime::{
        NativeWindowHook, NativeWindowHookRegistryResource, NativeWindowId,
        NativeWindowLifecycleState, NativeWindowRequest, NativeWindowRequestOutcome,
        NativeWindowRequestReadiness, WindowStateRegistryResource, FramePacingPolicyResource,
    };
    use std::{
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
            mpsc,
        },
        thread,
        time::{Duration, Instant},
    };
    use winit::window::Window;

    struct Hook {
        secondary_count: Arc<AtomicUsize>,
        unexpected_gpu: Arc<AtomicBool>,
    }

    impl NativeWindowHook for Hook {
        fn name(&self) -> &'static str {
            "native_window_request_os"
        }

        fn attach(&mut self, _window: &Window, world: &mut runen_ecs::World) -> Result<()> {
            if world.has_resource::<Gfx>() {
                self.unexpected_gpu.store(true, Ordering::SeqCst);
            }
            let secondary_count = world
                .resource::<WindowStateRegistryResource>()?
                .records()
                .filter(|record| {
                    record.native_window_id != NativeWindowId::primary()
                        && record.lifecycle_state == NativeWindowLifecycleState::Created
                })
                .count();
            self.secondary_count
                .fetch_max(secondary_count, Ordering::SeqCst);
            if secondary_count >= 2 {
                world
                    .resource_mut::<WindowStateRegistryResource>()?
                    .record_mut(NativeWindowId::primary())
                    .context("primary record missing after secondary attach")?
                    .request_close();
            }
            Ok(())
        }
    }

    /// Trigger a real Native close while a worker has an already-accepted
    /// external window request. Winit cannot drain the command while the
    /// original invoker executes this frame callback.
    struct CloseRaceHook {
        notified: bool,
        frame_entered: mpsc::Sender<()>,
        request_accepted: mpsc::Receiver<()>,
    }

    impl NativeWindowHook for CloseRaceHook {
        fn name(&self) -> &'static str {
            "native_request_vs_primary_close"
        }

        fn frame(&mut self, _window: &Window, world: &mut runen_ecs::World) -> Result<()> {
            if self.notified {
                return Ok(());
            }
            self.notified = true;
            let signalled = self.frame_entered.send(()).is_ok();
            let admitted = self
                .request_accepted
                .recv_timeout(Duration::from_secs(10))
                .is_ok();

            // Always make shutdown bounded, even on a failed handoff;
            // the worker's own result and missing marker then fail the test.
            world
                .resource_mut::<WindowStateRegistryResource>()?
                .record_mut(NativeWindowId::primary())
                .context("primary missing before native close/request race")?
                .request_close();
            ensure!(
                signalled && admitted,
                "external Native request was not accepted during the held frame"
            );
            Ok(())
        }
    }

    pub(super) fn close_race() -> Result<()> {
        let (frame_entered, enter_receiver) = mpsc::channel();
        let (accepted_sender, request_accepted) = mpsc::channel();
        let mut app = App::new();
        app.add_plugins(default_plugins());
        app.init_resource::<NativeWindowHookRegistryResource>();
        app.world_mut()
            .resource_mut::<NativeWindowHookRegistryResource>()?
            .register_hook(CloseRaceHook {
                notified: false,
                frame_entered,
                request_accepted,
            });

        let windows = app.native_window_requests()?;
        let worker = thread::spawn(move || -> Result<()> {
            enter_receiver
                .recv_timeout(Duration::from_secs(10))
                .context("no actual native frame callback reached the race barrier")?;
            ensure!(
                windows.readiness()? == NativeWindowRequestReadiness::Ready,
                "Native command channel not ready during active frame"
            );
            let ticket = windows.request(NativeWindowRequest::new(
                "Accepted request interrupted by primary close",
                (500, 340),
            ))?;
            accepted_sender
                .send(())
                .context("native frame callback dropped the acceptance barrier")?;
            let terminal = ticket.recv_timeout(Duration::from_secs(10))?;
            ensure!(
                terminal == NativeWindowRequestOutcome::Cancelled,
                "request accepted before final close was not terminally cancelled: {terminal:?}"
            );
            ensure!(
                windows.readiness()? == NativeWindowRequestReadiness::Closed,
                "Native request channel not closed after primary retired"
            );
            ensure!(
                matches!(
                    windows.request(NativeWindowRequest::new("too late", (480, 320))),
                    Err(engine::runtime::NativeWindowRequestError::Closed)
                ),
                "Native owner accepted a request after shutdown"
            );
            ensure!(
                ticket.try_recv() == Err(std::sync::mpsc::TryRecvError::Disconnected),
                "cancelled ticket unexpectedly emitted a second terminal result"
            );
            Ok(())
        });

        app.run()?;
        worker
            .join()
            .expect("Native request/close worker thread must not panic")?;
        println!("native_window_request_close_race=pass accepted=1 cancelled=1 physical_x11=true");
        Ok(())
    }

    /// Test-only fault injection: a real X11 primary and Render-owned Gfx
    /// must have existed. Withhold the Gfx resource for the secondary request,
    /// forcing Render's normal secondary-attach error + rollback behavior.
    /// No production fault switch or new Renderer interface is installed.
    struct RenderRollbackHook {
        injected: bool,
        verified: Arc<AtomicBool>,
        failure_reported: Arc<AtomicBool>,
        ready_to_request: mpsc::Sender<()>,
        accepted_request: mpsc::Receiver<()>,
    }

    impl NativeWindowHook for RenderRollbackHook {
        fn name(&self) -> &'static str {
            "native_real_window_render_rollback"
        }

        fn frame(&mut self, _window: &Window, world: &mut runen_ecs::World) -> Result<()> {
            if !self.injected {
                self.injected = true;
                let gfx = world
                    .remove_resource::<Gfx>()
                    .context("selected native Render must own real Gfx before fault injection")?;
                drop(gfx);
                self.ready_to_request
                    .send(())
                    .context("Native rollback proof worker went away")?;
                self.accepted_request
                    .recv_timeout(Duration::from_secs(12))
                    .context("Native rollback proof did not admit a secondary request")?;
                return Ok(());
            }

            if self.failure_reported.load(Ordering::SeqCst)
                && !self.verified.load(Ordering::SeqCst)
            {
                let failures = world
                    .resource::<WindowStateRegistryResource>()?
                    .records()
                    .filter(|record| {
                        record.native_window_id != NativeWindowId::primary()
                            && record.lifecycle_state
                                == NativeWindowLifecycleState::CreationFailed
                    })
                    .map(|record| (record.native_window_id, record.failure_reason.clone()))
                    .collect::<Vec<_>>();
                let mut verified = false;
                if failures.len() == 1 {
                    let (id, reason) = &failures[0];
                    if reason.as_deref().is_some_and(|message| {
                        message.contains("GPU surface attachment failed")
                            && message.contains("runtime gfx is unavailable")
                    }) {
                        let render = world.resource::<RenderSurfaceRegistryResource>()?;
                        let no_active_mapping = render.surface_for_native_window(*id).is_none();
                        let retired = render.records().any(|record| {
                            record.native_window_id == *id
                                && record.lifecycle_state
                                    == RenderSurfaceLifecycleState::Retired
                        });
                        verified = no_active_mapping && retired;
                    }
                }
                self.verified.store(verified, Ordering::SeqCst);
                // Always request a bounded exit; the external test assertion
                // below fails if the exact owner invariants were not observed.
                world
                    .resource_mut::<WindowStateRegistryResource>()?
                    .record_mut(NativeWindowId::primary())
                    .context("Native primary missing after injected secondary rollback")?
                    .request_close();
            }
            Ok(())
        }
    }

    pub(super) fn render_failure() -> Result<()> {
        let verified = Arc::new(AtomicBool::new(false));
        let failure_reported = Arc::new(AtomicBool::new(false));
        let (ready_to_request, ready_receiver) = mpsc::channel();
        let (accepted_sender, accepted_request) = mpsc::channel();
        let mut app = App::new();
        app.with_frame_pacing(FramePacingPolicyResource::continuous_capped(30));
        app.add_plugins(default_plugins());
        app.add_plugin(RenderPlugin);
        app.init_resource::<NativeWindowHookRegistryResource>();
        app.world_mut()
            .resource_mut::<NativeWindowHookRegistryResource>()?
            .register_hook(RenderRollbackHook {
                injected: false,
                verified: Arc::clone(&verified),
                failure_reported: Arc::clone(&failure_reported),
                ready_to_request,
                accepted_request,
            });

        let windows = app.native_window_requests()?;
        let worker = thread::spawn(move || -> Result<()> {
            ready_receiver
                .recv_timeout(Duration::from_secs(12))
                .context("real Render native primary never reached initial frame")?;
            ensure!(
                windows.readiness()? == NativeWindowRequestReadiness::Ready,
                "Native owner was not ready when Render rollback was injected"
            );
            let ticket = windows.request(NativeWindowRequest::new(
                "secondary with injected missing Gfx",
                (500, 360),
            ))?;
            accepted_sender
                .send(())
                .context("Native Render rollback hook exited before admission")?;
            let result = ticket.recv_timeout(Duration::from_secs(12))?;
            match result {
                NativeWindowRequestOutcome::CreationFailed { reason } => {
                    ensure!(
                        reason.contains("GPU surface attachment failed")
                            && reason.contains("runtime gfx is unavailable"),
                        "physical Winit secondary did not fail at the intended Render seam: {reason}"
                    );
                }
                other => anyhow::bail!("injected Render failure returned wrong terminal: {other:?}"),
            }
            ensure!(
                ticket.try_recv() == Err(std::sync::mpsc::TryRecvError::Disconnected),
                "failed Native request delivered multiple terminal outcomes"
            );
            failure_reported.store(true, Ordering::SeqCst);
            let deadline = Instant::now() + Duration::from_secs(12);
            while windows.readiness()? != NativeWindowRequestReadiness::Closed {
                ensure!(
                    Instant::now() < deadline,
                    "Native owner did not stop after injected Render rollback"
                );
                thread::sleep(Duration::from_millis(5));
            }
            Ok(())
        });

        let app_result = app.run();
        let worker_result = worker
            .join()
            .expect("Render rollback worker must not panic");
        app_result?;
        worker_result?;
        ensure!(
            verified.load(Ordering::SeqCst),
            "Native failure record or Render retired surface was inconsistent"
        );
        println!(
            "native_window_request_render_rollback=pass physical_x11=true native_failed=true render_retired=true"
        );
        Ok(())
    }

    pub(super) fn run() -> Result<()> {
        let secondary_count = Arc::new(AtomicUsize::new(0));
        let unexpected_gpu = Arc::new(AtomicBool::new(false));
        let mut app = App::new();
        app.add_plugins(default_plugins());
        app.init_resource::<NativeWindowHookRegistryResource>();
        app.world_mut()
            .resource_mut::<NativeWindowHookRegistryResource>()?
            .register_hook(Hook {
                secondary_count: Arc::clone(&secondary_count),
                unexpected_gpu: Arc::clone(&unexpected_gpu),
            });

        let windows = app.native_window_requests()?;
        ensure!(
            windows.readiness()? == NativeWindowRequestReadiness::NotReady,
            "Native handle became ready before Winit startup"
        );
        let worker = thread::spawn(move || -> Result<NativeWindowId> {
            let deadline = Instant::now() + Duration::from_secs(20);
            while windows.readiness()? == NativeWindowRequestReadiness::NotReady {
                ensure!(
                    Instant::now() < deadline,
                    "Native Host never published request readiness"
                );
                thread::sleep(Duration::from_millis(5));
            }
            ensure!(
                windows.readiness()? == NativeWindowRequestReadiness::Ready,
                "Native Host closed before request readiness"
            );

            let ticket = windows.request(NativeWindowRequest::new(
                "Native external request proof",
                (480, 320),
            ))?;
            let result = ticket.recv_timeout(Duration::from_secs(20))?;
            let id = match result {
                NativeWindowRequestOutcome::Created { native_window_id } => native_window_id,
                other => anyhow::bail!("Native failed accepted creation ticket: {other:?}"),
            };
            ensure!(
                id != NativeWindowId::primary(),
                "secondary reused primary NativeWindowId"
            );

            // The Native owner must realize an accepted request even if the
            // caller drops its individual result receiver. The second
            // physical attach closes the initial primary through its hook.
            let dropped = windows.request(NativeWindowRequest::new(
                "Native dropped-receiver proof",
                (400, 300),
            ))?;
            drop(dropped);

            // The Created receipt may precede primary-close settlement; do
            // not assume the owner has already closed when Winit sends it.
            let shutdown_deadline = Instant::now() + Duration::from_secs(20);
            while windows.readiness()? != NativeWindowRequestReadiness::Closed {
                ensure!(
                    Instant::now() < shutdown_deadline,
                    "Native owner did not close the request ingress on exit"
                );
                thread::sleep(Duration::from_millis(5));
            }
            Ok(id)
        });

        app.run()?;
        let secondary_id = worker
            .join()
            .expect("Native window request worker must not panic")?;
        ensure!(
            secondary_count.load(Ordering::SeqCst) >= 2,
            "two physical secondary windows, including the dropped receiver, did not reach Native attach hooks"
        );
        ensure!(
            !unexpected_gpu.load(Ordering::SeqCst),
            "Native window request materialized GPU without RenderPlugin"
        );
        println!(
            "native_window_requests_os=pass secondary={} physical_x11=true dropped_receiver_realized=true",
            secondary_id.raw()
        );
        Ok(())
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    if let Some(mode) = std::env::var_os("RUNENWERK_NATIVE_WINDOW_REQUEST_OS_SMOKE") {
        match mode.to_str() {
            Some("1") => linux::run().expect("real Native window request proof must pass"),
            Some("close-race") => {
                linux::close_race().expect("real Native close/request race proof must pass")
            }
            Some("render-failure") => {
                linux::render_failure().expect("real Render rollback test must pass")
            }
            _ => panic!("unexpected native window-request proof mode"),
        }
    }
}
