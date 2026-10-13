//! Engine-only common runtime operations over distinct concrete RunenUI UiApps.
//! App State, Action and HostProtocol remain typed; no second UI runtime exists.

use std::any::Any;

use runenui_core::UiApp;
use runenui_runtime::{
    AppRuntime, InputObservationError, InputPumpBatch, InputShutdownBatch, PumpBudget,
    SurfaceBuildContext, SurfacePublication,
};

/// Typed application observations without access to host clock or wake wiring.
/// The Engine retains those runtime capabilities for its selected native host.
///
/// Host wiring cannot be replaced through this observation:
///
/// ```compile_fail,E0599
/// use engine::plugins::ui::UiAppObservation;
/// use runenui_core::UiApp;
/// fn replace_wake<A: UiApp>(view: UiAppObservation<'_, A>) {
///     view.set_wake_transport(|| {});
/// }
/// ```
///
/// Deterministic time also stays with its host:
///
/// ```compile_fail,E0599
/// use engine::plugins::ui::UiAppObservation;
/// use runenui_core::UiApp;
/// fn advance_clock<A: UiApp>(view: UiAppObservation<'_, A>) {
///     view.advance_time(std::time::Duration::from_secs(1));
/// }
/// ```
pub struct UiAppObservation<'a, A: UiApp> {
    pub(super) state: &'a A::State,
    pub(super) pending_host_requests: Vec<runenui_runtime::HostRequestRef<'a, A::HostProtocol>>,
}

impl<A: UiApp> UiAppObservation<'_, A> {
    pub fn state(&self) -> &A::State {
        self.state
    }

    pub fn pending_host_requests(&self) -> &[runenui_runtime::HostRequestRef<'_, A::HostProtocol>] {
        &self.pending_host_requests
    }
}

/// The only erased work is truly common Engine host plumbing. Application
/// actions, state and host responses never cross this interface.
pub(super) trait TypedRuntimeOps {
    fn pump(&mut self, budget: PumpBudget) -> Result<InputPumpBatch, InputObservationError>;
    fn set_native_wake_transport(&self, transport: crate::runtime::NativeWakeTransport);
    fn pending_redraw(&mut self) -> bool;
    fn publish_surface(
        &mut self,
        context: &SurfaceBuildContext<'_>,
    ) -> Result<SurfacePublication, runenui_runtime::PublishSurfaceError>;
    fn shutdown(&mut self) -> Result<InputShutdownBatch, InputObservationError>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub(super) struct TypedRuntime<A: UiApp> {
    pub(super) runtime: AppRuntime<A>,
}

impl<A: UiApp + 'static> TypedRuntimeOps for TypedRuntime<A> {
    fn set_native_wake_transport(&self, transport: crate::runtime::NativeWakeTransport) {
        self.runtime.set_wake_transport(transport);
    }

    fn pump(&mut self, budget: PumpBudget) -> Result<InputPumpBatch, InputObservationError> {
        self.runtime.pump(budget)
    }

    fn pending_redraw(&mut self) -> bool {
        self.runtime.take_redraw_request().is_some()
    }

    fn publish_surface(
        &mut self,
        context: &SurfaceBuildContext<'_>,
    ) -> Result<SurfacePublication, runenui_runtime::PublishSurfaceError> {
        self.runtime.publish_surface(context)
    }

    fn shutdown(&mut self) -> Result<InputShutdownBatch, InputObservationError> {
        self.runtime.shutdown()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
