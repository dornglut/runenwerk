//! Primary Render surface retirement, deliberately separate from both forced
//! secondary close and Native App lifetime selection.
//!
//! This operation is an internal Render owner prerequisite. It never calls the
//! Native event loop, selects a surviving primary, or changes App shutdown.

use super::super::renderer::Gfx;
use super::super::{
    RenderDynamicTextureTargetRequestRegistryResource, RenderGpuFlowStepStreams,
    render_integration_is_active,
};
use super::{RenderSurfaceId, RenderSurfaceLifecycleState, RenderSurfaceRegistryResource};
use crate::plugins::ui::retirement::{
    UiSurfaceRetirementBlocker, UiSurfaceRetirementError, UiSurfaceRetirementStatus,
    inspect_native_surface_ui_retirement,
};
use crate::runtime::NativeWindowId;
use runen_ecs::World;
use runen_gpu::GpuSurfaceConditionalDetachOutcome;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrimaryRenderRetirementDeferral {
    Ui(UiSurfaceRetirementBlocker),
    AcceptedPrimaryFrames,
    InFlightGpuSubmissions,
    PendingReadbacks,
    RetainedRenderSession,
    SemanticGpuProducer,
    ActiveSurfaceLease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrimaryRenderRetirementOutcome {
    NoRender,
    AlreadyRetired,
    Deferred(PrimaryRenderRetirementDeferral),
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum PrimaryRenderRetirementError {
    #[error("GPU context exists without selected Render integration")]
    InconsistentRenderActivation,
    #[error("selected Render integration has no surface registry")]
    MissingSurfaceRegistry,
    #[error("selected Render integration has no primary Render record")]
    MissingPrimaryRecord,
    #[error("primary Render association does not match the primary native window")]
    MismatchedPrimaryIdentity,
    #[error("primary Render surface is not attached")]
    PrimaryNotAttached,
    #[error("attached primary Render surface has no physical GPU context or surface")]
    MissingGpuSurface,
    #[error("selected Render integration has no dynamic target registry")]
    MissingTargetRegistry,
    #[error("primary still has registered dynamic texture target intent")]
    RegisteredPrimaryTargetIntent,
    #[error("RunenUI cannot certify the primary: {0}")]
    Ui(#[from] UiSurfaceRetirementError),
    #[error("conditional physical primary detach refused: {0}")]
    PhysicalDetach(String),
    #[error("primary Render record disappeared after physical retirement")]
    LostPrimaryRecord,
}

/// Read-only, synchronous Native-invoker preflight. The UI result is an owner
/// witness, never proof of completed G5 or an inactive G7 acquisition lease.
pub(crate) fn inspect_primary_render_retirement(
    world: &World,
) -> Result<PrimaryRenderRetirementOutcome, PrimaryRenderRetirementError> {
    let selected = render_integration_is_active(world);
    if !selected {
        return if world.has_resource::<Gfx>() {
            Err(PrimaryRenderRetirementError::InconsistentRenderActivation)
        } else {
            Ok(PrimaryRenderRetirementOutcome::NoRender)
        };
    }

    let registry = world
        .resource::<RenderSurfaceRegistryResource>()
        .map_err(|_| PrimaryRenderRetirementError::MissingSurfaceRegistry)?;
    let native = NativeWindowId::primary();
    let primary = RenderSurfaceId::primary();
    let gfx = world
        .resource::<Gfx>()
        .map_err(|_| PrimaryRenderRetirementError::MissingGpuSurface)?;

    if registry.surface_for_native_window(native).is_none()
        && registry.primary_surface_id().is_none()
        && registry
            .record(primary)
            .is_some_and(|record| {
                record.native_window_id == native
                    && record.lifecycle_state == RenderSurfaceLifecycleState::Retired
            })
        && !gfx.has_surface(primary)
    {
        return Ok(PrimaryRenderRetirementOutcome::AlreadyRetired);
    }
    if registry.primary_surface_id() != Some(primary)
        || registry.surface_for_native_window(native) != Some(primary)
    {
        return Err(PrimaryRenderRetirementError::MissingPrimaryRecord);
    }
    let record = registry
        .record(primary)
        .ok_or(PrimaryRenderRetirementError::MissingPrimaryRecord)?;
    if record.native_window_id != native {
        return Err(PrimaryRenderRetirementError::MismatchedPrimaryIdentity);
    }
    if record.lifecycle_state != RenderSurfaceLifecycleState::Attached {
        return Err(PrimaryRenderRetirementError::PrimaryNotAttached);
    }
    if !gfx.has_surface(primary) {
        return Err(PrimaryRenderRetirementError::MissingGpuSurface);
    }

    match inspect_native_surface_ui_retirement(world, native, primary)? {
        UiSurfaceRetirementStatus::NoUiCapability | UiSurfaceRetirementStatus::Cleared => {}
        UiSurfaceRetirementStatus::Blocked(blocker) => {
            return Ok(PrimaryRenderRetirementOutcome::Deferred(
                PrimaryRenderRetirementDeferral::Ui(blocker),
            ));
        }
    }

    let targets = world
        .resource::<RenderDynamicTextureTargetRequestRegistryResource>()
        .map_err(|_| PrimaryRenderRetirementError::MissingTargetRegistry)?;
    if !targets.snapshot_for_surface(primary).is_empty() {
        return Err(PrimaryRenderRetirementError::RegisteredPrimaryTargetIntent);
    }

    if gfx
        .pending_gpu_frame_submissions()
        .iter()
        .any(|receipt| receipt.surface() == primary.raw())
    {
        return Ok(PrimaryRenderRetirementOutcome::Deferred(
            PrimaryRenderRetirementDeferral::AcceptedPrimaryFrames,
        ));
    }
    if gfx.has_unsettled_retained_surface(primary) {
        return Ok(PrimaryRenderRetirementOutcome::Deferred(
            PrimaryRenderRetirementDeferral::RetainedRenderSession,
        ));
    }
    if world
        .resource::<RenderGpuFlowStepStreams>()
        .ok()
        .is_some_and(|streams| streams.has_unsettled_accepted_work())
    {
        return Ok(PrimaryRenderRetirementOutcome::Deferred(
            PrimaryRenderRetirementDeferral::SemanticGpuProducer,
        ));
    }
    // These RunenGPU counters are context-wide. Their precise surface origin
    // is not exposed; defer rather than falsely attribute readback completion.
    let execution = gfx.execution_stats();
    if execution.in_flight_submissions() != 0 {
        return Ok(PrimaryRenderRetirementOutcome::Deferred(
            PrimaryRenderRetirementDeferral::InFlightGpuSubmissions,
        ));
    }
    if execution.pending_readbacks() != 0 {
        return Ok(PrimaryRenderRetirementOutcome::Deferred(
            PrimaryRenderRetirementDeferral::PendingReadbacks,
        ));
    }
    Ok(PrimaryRenderRetirementOutcome::Retired)
}

/// The sole physical commit follows a fresh synchronous preflight. The G7
/// conditional detach protects the active image lease under RunenGPU's lock.
/// No opaque bool or forcibly abandoned lease may authorize logical retirement.
pub(crate) fn try_retire_primary_render_surface(
    world: &mut World,
) -> Result<PrimaryRenderRetirementOutcome, PrimaryRenderRetirementError> {
    let outcome = inspect_primary_render_retirement(world)?;
    if outcome != PrimaryRenderRetirementOutcome::Retired {
        return Ok(outcome);
    }
    let primary = RenderSurfaceId::primary();
    let detached = world
        .resource_mut::<Gfx>()
        .expect("fresh synchronous preflight retains Gfx")
        .detach_surface_if_unleased(primary)
        .map_err(|err| PrimaryRenderRetirementError::PhysicalDetach(err.to_string()))?;
    match detached {
        GpuSurfaceConditionalDetachOutcome::ActiveLease => {
            Ok(PrimaryRenderRetirementOutcome::Deferred(
                PrimaryRenderRetirementDeferral::ActiveSurfaceLease,
            ))
        }
        GpuSurfaceConditionalDetachOutcome::Detached => {
            // Only this exact primary mapping may retire. The surviving native
            // windows, RenderSurfaceIds, retained sessions and Gfx context stay.
            let retired = world
                .resource_mut::<RenderSurfaceRegistryResource>()
                .expect("fresh synchronous preflight retains Render registry")
                .retire_surface_for_native_window(NativeWindowId::primary());
            if retired != Some(primary) {
                return Err(PrimaryRenderRetirementError::LostPrimaryRecord);
            }
            Ok(PrimaryRenderRetirementOutcome::Retired)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::plugins::RenderPlugin;

    #[test]
    fn no_render_is_a_truthful_noop_and_does_not_create_registries() {
        let mut app = App::new();
        assert_eq!(
            try_retire_primary_render_surface(app.world_mut()),
            Ok(PrimaryRenderRetirementOutcome::NoRender)
        );
        assert!(!app.world().has_resource::<RenderSurfaceRegistryResource>());
        assert!(!app.world().has_resource::<Gfx>());
    }

    #[test]
    fn selected_render_without_the_owner_registry_refuses() {
        let mut app = App::new();
        app.world_mut()
            .insert_resource(crate::plugins::render::plugin::RenderIntegrationActivation);
        assert_eq!(
            try_retire_primary_render_surface(app.world_mut()),
            Err(PrimaryRenderRetirementError::MissingSurfaceRegistry)
        );
    }

    #[test]
    fn selected_render_cannot_claim_primary_without_real_physical_gpu() {
        let mut app = App::new();
        app.add_plugin(RenderPlugin);
        let primary = RenderSurfaceId::primary();
        let native = NativeWindowId::primary();
        app.world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()
            .unwrap()
            .confirm_surface_attachment(primary, native, (600, 400))
            .unwrap();
        assert_eq!(
            try_retire_primary_render_surface(app.world_mut()),
            Err(PrimaryRenderRetirementError::MissingGpuSurface)
        );
        let registry = app.world().resource::<RenderSurfaceRegistryResource>().unwrap();
        assert_eq!(registry.primary_surface_id(), Some(primary));
        assert_eq!(registry.surface_for_native_window(native), Some(primary));
        assert_eq!(
            registry.record(primary).unwrap().lifecycle_state,
            RenderSurfaceLifecycleState::Attached
        );
    }

    #[test]
    fn broken_logical_primary_is_not_a_successful_noop() {
        let mut app = App::new();
        app.add_plugin(RenderPlugin);
        assert_eq!(
            inspect_primary_render_retirement(app.world()),
            Err(PrimaryRenderRetirementError::MissingGpuSurface)
        );
        assert!(app.world().resource::<RenderSurfaceRegistryResource>().is_ok());
    }

    #[test]
    fn selected_render_with_missing_registered_owner_is_rejected() {
        let mut app = App::new();
        app.add_plugin(RenderPlugin);
        let _ = app
            .world_mut()
            .remove_resource::<RenderSurfaceRegistryResource>();
        assert_eq!(
            inspect_primary_render_retirement(app.world()),
            Err(PrimaryRenderRetirementError::MissingSurfaceRegistry)
        );
    }
}
