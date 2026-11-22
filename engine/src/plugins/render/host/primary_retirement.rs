//! Primary Render surface retirement, deliberately separate from both forced
//! secondary close and Native App lifetime selection.
//!
//! This operation is an internal Render owner prerequisite. It never calls the
//! Native event loop, selects a surviving primary, or changes App shutdown.

use super::super::renderer::Gfx;
use super::super::{
    PreparedRenderFrameResource, RenderDynamicTextureTargetRequestRegistryResource,
    RenderGpuFlowStepStreams, RunenUiPaintSubmissionRegistryResource,
    SurfaceFrameSubmissionRegistryResource, render_integration_is_active,
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
    PreparedPrimaryFrame,
    UndeliveredGpuObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrimaryRenderRetirementOutcome {
    NoRender,
    AlreadyRetired,
    Deferred(PrimaryRenderRetirementDeferral),
    ReadyToRetire,
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
    #[error("selected Render integration has no prepared frame resource")]
    MissingPreparedFrameRegistry,
    #[error("selected Render integration has no surface frame submission registry")]
    MissingFrameSubmissionRegistry,
    #[error("primary still has registered dynamic texture target intent")]
    RegisteredPrimaryTargetIntent,
    #[error("primary still has registered surface frame submission intent")]
    RegisteredPrimaryFrameIntent,
    #[error("selected Render integration has no RunenUI paint registry")]
    MissingPaintRegistry,
    #[error("primary still has unwithdrawn RunenUI Render paint")]
    ResidualRunenUiPaint,
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

    let prepared = world
        .resource::<PreparedRenderFrameResource>()
        .map_err(|_| PrimaryRenderRetirementError::MissingPreparedFrameRegistry)?;
    if prepared
        .frames()
        .any(|frame| frame.surface.render_surface_id == primary)
    {
        return Ok(PrimaryRenderRetirementOutcome::Deferred(
            PrimaryRenderRetirementDeferral::PreparedPrimaryFrame,
        ));
    }

    let surface_submissions = world
        .resource::<SurfaceFrameSubmissionRegistryResource>()
        .map_err(|_| PrimaryRenderRetirementError::MissingFrameSubmissionRegistry)?;
    if !surface_submissions
        .ordered_submissions_for_surface(primary)
        .is_empty()
    {
        return Err(PrimaryRenderRetirementError::RegisteredPrimaryFrameIntent);
    }

    // RunenUI paint is Render-owned even when UiPlugin is not installed.
    // Do not infer a paint-free surface solely from absent UI capability.
    let paint = world
        .resource::<RunenUiPaintSubmissionRegistryResource>()
        .map_err(|_| PrimaryRenderRetirementError::MissingPaintRegistry)?;
    if !paint.ordered_submissions_for_surface(primary).is_empty() {
        return Err(PrimaryRenderRetirementError::ResidualRunenUiPaint);
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
    if gfx.native_gpu_progress_pending() {
        return Ok(PrimaryRenderRetirementOutcome::Deferred(
            PrimaryRenderRetirementDeferral::UndeliveredGpuObservation,
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
    Ok(PrimaryRenderRetirementOutcome::ReadyToRetire)
}

/// The sole physical commit follows a fresh synchronous preflight. The G7
/// conditional detach protects the active image lease under RunenGPU's lock.
/// No opaque bool or forcibly abandoned lease may authorize logical retirement.
pub(crate) fn try_retire_primary_render_surface(
    world: &mut World,
) -> Result<PrimaryRenderRetirementOutcome, PrimaryRenderRetirementError> {
    let outcome = inspect_primary_render_retirement(world)?;
    if outcome != PrimaryRenderRetirementOutcome::ReadyToRetire {
        return Ok(outcome);
    }
    let primary = RenderSurfaceId::primary();
    let detached = world
        .resource_mut::<Gfx>()
        .expect("fresh synchronous preflight retains Gfx")
        .detach_surface_if_unleased(primary)
        .map_err(|err| PrimaryRenderRetirementError::PhysicalDetach(err.to_string()))?;
    finish_primary_conditional_detach(world, detached)
}

/// Render registry effects of the exact conditional G7 result. A busy lease
/// cannot change logical identities, even on repeated close attempts.
fn finish_primary_conditional_detach(
    world: &mut World,
    detached: GpuSurfaceConditionalDetachOutcome,
) -> Result<PrimaryRenderRetirementOutcome, PrimaryRenderRetirementError> {
    match detached {
        GpuSurfaceConditionalDetachOutcome::ActiveLease => {
            Ok(PrimaryRenderRetirementOutcome::Deferred(
                PrimaryRenderRetirementDeferral::ActiveSurfaceLease,
            ))
        }
        GpuSurfaceConditionalDetachOutcome::Detached => {
            // Only this exact primary mapping may retire. The surviving native
            // windows, RenderSurfaceIds, retained sessions and Gfx context stay.
            commit_primary_registry_retirement(world)?;
            Ok(PrimaryRenderRetirementOutcome::Retired)
        }
    }
}

/// Logical commit after G7 has already returned exact Detached. Never
/// invoked on a refused or actively leased physical surface.
fn commit_primary_registry_retirement(
    world: &mut World,
) -> Result<(), PrimaryRenderRetirementError> {
    let retired = world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .expect("fresh synchronous preflight retains Render registry")
        .retire_surface_for_native_window(NativeWindowId::primary());
    if retired != Some(RenderSurfaceId::primary()) {
        return Err(PrimaryRenderRetirementError::LostPrimaryRecord);
    }
    Ok(())
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
    fn post_g7_logical_commit_preserves_secondary_registry_and_identity() {
        // Pure Render registry transaction. Actual G7 lease safety is
        // separately verified by RunenGPU and by Native Vulkan qualification.
        let mut app = App::new();
        app.init_resource::<RenderSurfaceRegistryResource>();
        let native_primary = NativeWindowId::primary();
        let render_primary = RenderSurfaceId::primary();
        let native_secondary = NativeWindowId::try_from_raw(2).unwrap();
        app.world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()
            .unwrap()
            .confirm_surface_attachment(render_primary, native_primary, (640, 480))
            .unwrap();
        let secondary = app
            .world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()
            .unwrap()
            .reserve_surface_for_native_window(native_secondary, (480, 320));
        app.world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()
            .unwrap()
            .confirm_surface_attachment(secondary, native_secondary, (480, 320))
            .unwrap();
        assert_eq!(
            finish_primary_conditional_detach(
                app.world_mut(),
                GpuSurfaceConditionalDetachOutcome::ActiveLease,
            ),
            Ok(PrimaryRenderRetirementOutcome::Deferred(
                PrimaryRenderRetirementDeferral::ActiveSurfaceLease,
            ))
        );
        {
            let registry = app.world().resource::<RenderSurfaceRegistryResource>().unwrap();
            assert_eq!(registry.surface_for_native_window(native_primary), Some(render_primary));
            assert_eq!(registry.surface_for_native_window(native_secondary), Some(secondary));
            assert_eq!(
                registry.record(render_primary).unwrap().lifecycle_state,
                RenderSurfaceLifecycleState::Attached
            );
        }
        assert_eq!(
            finish_primary_conditional_detach(
                app.world_mut(),
                GpuSurfaceConditionalDetachOutcome::Detached,
            ),
            Ok(PrimaryRenderRetirementOutcome::Retired)
        );
        let registry = app.world().resource::<RenderSurfaceRegistryResource>().unwrap();
        assert_eq!(registry.surface_for_native_window(native_primary), None);
        assert_eq!(registry.primary_surface_id(), None);
        assert_eq!(registry.surface_for_native_window(native_secondary), Some(secondary));
        assert_eq!(
            registry.record(secondary).unwrap().lifecycle_state,
            RenderSurfaceLifecycleState::Attached
        );
        assert_eq!(
            registry.record(render_primary).unwrap().lifecycle_state,
            RenderSurfaceLifecycleState::Retired
        );
        assert_eq!(
            commit_primary_registry_retirement(app.world_mut()),
            Err(PrimaryRenderRetirementError::LostPrimaryRecord)
        );
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
