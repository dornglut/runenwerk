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

    /// Observes terminal facts already delivered by Render/UI on the native
    /// invoker. Return true only once for an actual application-owned wait
    /// satisfied by exact successful completion in its still-live window,
    /// slot/publication and GPU generation. Retire cancelled or stale waits.
    ///
    /// Passive completion returns false. Do not poll the GPU, perform the next
    /// application action, or run a frame here. The Host admits a requested
    /// ordinary continuation through its existing canonical frame gate.
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
        let mut continuation = false;
        for hook in &mut self.hooks {
            match hook.gpu_progress(world) {
                Ok(requested) => continuation |= requested,
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
        Ok(continuation)
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

    struct Consumer(Option<bool>);
    impl NativeWindowHook for Consumer {
        fn name(&self) -> &'static str {
            "waiting application"
        }
        fn gpu_progress(&mut self, _: &mut World) -> Result<bool> {
            match self.0.take() {
                Some(false) => anyhow::bail!("cancelled wait"),
                ready => Ok(ready == Some(true)),
            }
        }
    }

    #[test]
    fn terminal_delivery_has_no_passive_or_repeated_application_demand() {
        let mut world = World::new();
        let mut hooks = NativeWindowHookRegistryResource::default();
        assert!(!hooks.dispatch_gpu_progress(&mut world).unwrap());
        hooks.register_hook(Consumer(None));
        assert!(!hooks.dispatch_gpu_progress(&mut world).unwrap());
        hooks.register_hook(Consumer(Some(true)));
        assert!(hooks.dispatch_gpu_progress(&mut world).unwrap());
        assert!(!hooks.dispatch_gpu_progress(&mut world).unwrap());
    }

    #[test]
    fn failed_consumer_does_not_return_an_earlier_successful_request() {
        let mut world = World::new();
        let mut hooks = NativeWindowHookRegistryResource::default();
        hooks.register_hook(Consumer(Some(true)));
        hooks.register_hook(Consumer(Some(false)));
        assert!(hooks.dispatch_gpu_progress(&mut world).is_err());
        assert_eq!(
            hooks.diagnostics()[0].stage,
            NativeWindowHookStage::GpuProgress
        );
    }
}
