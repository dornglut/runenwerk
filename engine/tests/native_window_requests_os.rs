//! Real Linux/X11 Winit request-ingress and terminal-receipt proof.
#![cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]

#[cfg(target_os = "linux")]
mod linux {
    use anyhow::{Context, Result, ensure};
    use engine::app::App;
    use engine::plugins::default_plugins;
    use engine::plugins::render::Gfx;
    use engine::runtime::{
        NativeWindowHook, NativeWindowHookRegistryResource, NativeWindowId,
        NativeWindowLifecycleState, NativeWindowRequest, NativeWindowRequestOutcome,
        NativeWindowRequestReadiness, WindowStateRegistryResource,
    };
    use std::{
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
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
    if std::env::var_os("RUNENWERK_NATIVE_WINDOW_REQUEST_OS_SMOKE").is_some() {
        linux::run().expect("real Native window request and receipt proof must pass");
    }
}
