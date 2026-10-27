//! Native window hook extension points for platform adapters.

use crate::runtime::NativeHostEvent;
use anyhow::Result;
use runen_ecs::World;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::window::Window;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeWindowHookDiagnostic {
    pub hook_name: &'static str,
    pub stage: NativeWindowHookStage,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeWindowHookStage {
    Attach,
    WindowEvent,
    DeviceEvent,
    Frame,
    GpuProgress,
    Detach,
}

pub trait NativeWindowHook: 'static {
    fn name(&self) -> &'static str;

    fn configure_event_loop(&mut self, _builder: &mut EventLoopBuilder<NativeHostEvent>) {}

    fn attach(&mut self, _window: &Window, _world: &mut World) -> Result<()> {
        Ok(())
    }

    fn window_event(
        &mut self,
        _window: &Window,
        _event: &WindowEvent,
        _world: &mut World,
    ) -> Result<()> {
        Ok(())
    }

    fn device_event(&mut self, _event: &DeviceEvent, _world: &mut World) -> Result<()> {
        Ok(())
    }

    fn frame(&mut self, _window: &Window, _world: &mut World) -> Result<()> {
        Ok(())
    }

    /// Observe already-delivered GPU terminal facts on the native invoker.
    /// Return true only once for a live, compatible application wait whose
    /// next action requires an ordinary frame. Passive completion is not demand.
    /// This callback must not execute frame bookkeeping or poll the GPU.
    fn gpu_progress(&mut self, _world: &mut World) -> Result<bool> {
        Ok(false)
    }

    fn detach(&mut self, _world: &mut World) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct NativeWindowHookRegistryResource {
    hooks: Vec<Box<dyn NativeWindowHook>>,
    diagnostics: Vec<NativeWindowHookDiagnostic>,
}

impl runen_ecs::Resource for NativeWindowHookRegistryResource {}

impl NativeWindowHookRegistryResource {
    pub fn register_hook(&mut self, hook: impl NativeWindowHook) {
        self.hooks.push(Box::new(hook));
    }

    pub fn hook_count(&self) -> usize {
        self.hooks.len()
    }

    pub fn diagnostics(&self) -> &[NativeWindowHookDiagnostic] {
        &self.diagnostics
    }

    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    pub fn configure_event_loop(&mut self, builder: &mut EventLoopBuilder<NativeHostEvent>) {
        for hook in &mut self.hooks {
            hook.configure_event_loop(builder);
        }
    }

    pub fn attach_hooks(&mut self, window: &Window, world: &mut World) {
        for hook in &mut self.hooks {
            if let Err(err) = hook.attach(window, world) {
                self.diagnostics.push(NativeWindowHookDiagnostic {
                    hook_name: hook.name(),
                    stage: NativeWindowHookStage::Attach,
                    message: format!("{err:#}"),
                });
            }
        }
    }

    pub fn dispatch_window_event(
        &mut self,
        window: &Window,
        event: &WindowEvent,
        world: &mut World,
    ) {
        for hook in &mut self.hooks {
            if let Err(err) = hook.window_event(window, event, world) {
                self.diagnostics.push(NativeWindowHookDiagnostic {
                    hook_name: hook.name(),
                    stage: NativeWindowHookStage::WindowEvent,
                    message: format!("{err:#}"),
                });
            }
        }
    }

    pub fn dispatch_device_event(&mut self, event: &DeviceEvent, world: &mut World) {
        for hook in &mut self.hooks {
            if let Err(err) = hook.device_event(event, world) {
                self.diagnostics.push(NativeWindowHookDiagnostic {
                    hook_name: hook.name(),
                    stage: NativeWindowHookStage::DeviceEvent,
                    message: format!("{err:#}"),
                });
            }
        }
    }

    pub fn dispatch_frame(&mut self, window: &Window, world: &mut World) {
        for hook in &mut self.hooks {
            if let Err(err) = hook.frame(window, world) {
                self.diagnostics.push(NativeWindowHookDiagnostic {
                    hook_name: hook.name(),
                    stage: NativeWindowHookStage::Frame,
                    message: format!("{err:#}"),
                });
            }
        }
    }

    pub fn detach_hooks(&mut self, world: &mut World) {
        for hook in &mut self.hooks {
            if let Err(err) = hook.detach(world) {
                self.diagnostics.push(NativeWindowHookDiagnostic {
                    hook_name: hook.name(),
                    stage: NativeWindowHookStage::Detach,
                    message: format!("{err:#}"),
                });
            }
        }
    }

    pub(crate) fn dispatch_gpu_progress(&mut self, world: &mut World) -> Result<bool> {
        let mut requested = false;
        for hook in &mut self.hooks {
            match hook.gpu_progress(world) {
                Ok(continuation) => requested |= continuation,
                Err(error) => {
                    self.diagnostics.push(NativeWindowHookDiagnostic {
                        hook_name: hook.name(),
                        stage: NativeWindowHookStage::GpuProgress,
                        message: format!("{error:#}"),
                    });
                    return Err(error);
                }
            }
        }
        Ok(requested)
    }
}

pub(crate) fn with_native_window_hooks(
    world: &mut World,
    f: impl FnOnce(&mut NativeWindowHookRegistryResource, &mut World),
) {
    let Some(mut registry) = world.remove_resource::<NativeWindowHookRegistryResource>() else {
        return;
    };
    f(&mut registry, world);
    world.insert_resource(registry);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct WaitingConsumer(bool);
    impl NativeWindowHook for WaitingConsumer {
        fn name(&self) -> &'static str {
            "waiting consumer"
        }
        fn gpu_progress(&mut self, _: &mut World) -> Result<bool> {
            Ok(std::mem::take(&mut self.0))
        }
    }

    #[test]
    fn native_gpu_delivery_has_no_implicit_or_repeated_continuation() {
        let mut world = World::new();
        let mut hooks = NativeWindowHookRegistryResource::default();
        assert!(!hooks.dispatch_gpu_progress(&mut world).unwrap());
        hooks.register_hook(WaitingConsumer(false));
        assert!(!hooks.dispatch_gpu_progress(&mut world).unwrap());
        hooks.register_hook(WaitingConsumer(true));
        assert!(hooks.dispatch_gpu_progress(&mut world).unwrap());
        assert!(!hooks.dispatch_gpu_progress(&mut world).unwrap());
    }

    #[test]
    fn native_gpu_consumer_failure_cannot_return_a_successful_frame_request() {
        struct FailedConsumer;
        impl NativeWindowHook for FailedConsumer {
            fn name(&self) -> &'static str {
                "failed consumer"
            }
            fn gpu_progress(&mut self, _: &mut World) -> Result<bool> {
                anyhow::bail!("retired consumer wait")
            }
        }
        let mut world = World::new();
        let mut hooks = NativeWindowHookRegistryResource::default();
        hooks.register_hook(WaitingConsumer(true));
        hooks.register_hook(FailedConsumer);
        assert!(hooks.dispatch_gpu_progress(&mut world).is_err());
        assert_eq!(
            hooks.diagnostics()[0].stage,
            NativeWindowHookStage::GpuProgress
        );
    }
}
