//! Engine-only common runtime operations over distinct concrete RunenUI UiApps.
//! App State, Action and HostProtocol remain typed; no second UI runtime exists.

use std::any::Any;

use runenui_core::UiApp;
use runenui_runtime::{
    AppRuntime, InputObservationError, InputPumpBatch, InputShutdownBatch, PumpBudget,
    SurfaceBuildContext, SurfacePublication,
};

/// The only erased work is truly common Engine host plumbing. Application
/// actions, state and host responses never cross this interface.
pub(super) trait TypedRuntimeOps {
    fn pump(&mut self, budget: PumpBudget) -> Result<InputPumpBatch, InputObservationError>;
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
