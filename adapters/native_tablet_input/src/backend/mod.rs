//! Native tablet backend adapters.

use crate::model::{
    NativeTabletBackendKind, NativeTabletDeviceControlResource, NativeTabletRuntimeResource,
};
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::window::Window;

pub mod macos_nsevent;
pub mod windows_pointer;
pub mod windows_wintab;

pub use macos_nsevent::*;
pub use windows_pointer::*;
pub use windows_wintab::*;

pub trait NativeTabletBackendAdapter {
    fn kind(&self) -> NativeTabletBackendKind;

    fn configure_event_loop(&mut self, _builder: &mut EventLoopBuilder<()>) {}

    fn attach(
        &mut self,
        _window: &Window,
        _runtime: &mut NativeTabletRuntimeResource,
        _control: &NativeTabletDeviceControlResource,
    ) {
    }

    fn window_event(
        &mut self,
        _window: &Window,
        _event: &WindowEvent,
        _runtime: &mut NativeTabletRuntimeResource,
        _control: &NativeTabletDeviceControlResource,
    ) {
    }

    fn device_event(
        &mut self,
        _event: &DeviceEvent,
        _runtime: &mut NativeTabletRuntimeResource,
        _control: &NativeTabletDeviceControlResource,
    ) {
    }

    fn frame(
        &mut self,
        _runtime: &mut NativeTabletRuntimeResource,
        _control: &NativeTabletDeviceControlResource,
    ) {
    }

    fn detach(&mut self, _runtime: &mut NativeTabletRuntimeResource) {}
}
