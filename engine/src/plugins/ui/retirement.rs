//! Exact-target, no-frame RunenUI withdrawal for future Native primary retirement.
//!
//! This does NOT retire a native window, settle M13, complete a GPU submission,
//! or certify Render G5/G7 physical detachment. Those semantic owners remain
//! separate. The caller first explicitly retires the original input recipient
//! and unpresents/unmounts the UI, then asks this owner to withdraw only the
//! exact target's remaining renderer paint and producer lease.

use runen_ecs::World;

use crate::plugins::render::host::{RenderSurfaceId, RenderSurfaceRegistryResource};
use crate::plugins::render::{
    ManagedRenderProducerIds, RunenUiPaintSubmissionRegistryResource, render_integration_is_active,
};
use crate::runtime::NativeWindowId;

use super::native_input::UiNativeInputRetirementState;
use super::publish::retire_withdrawn_managed_producer;
use super::{
    UiNativeInputRecipientsResource, UiPluginStateResource,
    UiRuntimePresentationAssociationsResource, UiRuntimeSlotsResource,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiSurfaceRetirementBlocker {
    LiveBinding,
    UnsettledPublication,
    LiveNativeInput,
    UnsettledNativeInput,
    PendingWithdrawal,
    RendererPaint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiSurfaceRetirementStatus {
    NoUiCapability,
    Cleared,
    Blocked(UiSurfaceRetirementBlocker),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub(crate) enum UiSurfaceRetirementError {
    #[error("UI retirement owner is incomplete: {0}")]
    IncompleteUiOwner(&'static str),
    #[error("Render paint withdrawal owner is unavailable: {0}")]
    IncompleteRenderOwner(&'static str),
    #[error("native window and Render surface have different actual owners")]
    MismatchedTarget,
}

/// Inspects exactly one native/render target without changing owner state.
/// A cleared UI witness is never a Render in-flight or physical GPU certificate.
pub(crate) fn inspect_native_surface_ui_retirement(
    world: &World,
    native: NativeWindowId,
    surface: RenderSurfaceId,
) -> Result<UiSurfaceRetirementStatus, UiSurfaceRetirementError> {
    let ui = world.resource::<UiPluginStateResource>().ok();
    if !ui.is_some_and(UiPluginStateResource::is_installed) {
        if ui.is_some()
            || world.has_resource::<UiRuntimePresentationAssociationsResource>()
            || world.has_resource::<UiNativeInputRecipientsResource>()
            || world.has_resource::<UiRuntimeSlotsResource>()
        {
            return Err(UiSurfaceRetirementError::IncompleteUiOwner(
                "UiPlugin is absent but UI integration resources exist",
            ));
        }
        return Ok(UiSurfaceRetirementStatus::NoUiCapability);
    }
    let associations = world
        .resource::<UiRuntimePresentationAssociationsResource>()
        .map_err(|_| UiSurfaceRetirementError::IncompleteUiOwner("presentation associations"))?;
    let recipients = world
        .resource::<UiNativeInputRecipientsResource>()
        .map_err(|_| UiSurfaceRetirementError::IncompleteUiOwner("native input recipients"))?;
    world
        .resource::<UiRuntimeSlotsResource>()
        .map_err(|_| UiSurfaceRetirementError::IncompleteUiOwner("typed UI runtime slots"))?;

    // Correlate only existing Render-owned facts; do not manufacture a target
    // or claim attachment when no Render capability is selected.
    if let Ok(surfaces) = world.resource::<RenderSurfaceRegistryResource>() {
        if surfaces
            .record(surface)
            .is_some_and(|record| record.native_window_id != native)
            || surfaces
                .surface_for_native_window(native)
                .is_some_and(|actual| actual != surface)
        {
            return Err(UiSurfaceRetirementError::MismatchedTarget);
        }
    }
    if render_integration_is_active(world) {
        world
            .resource::<RenderSurfaceRegistryResource>()
            .map_err(|_| UiSurfaceRetirementError::IncompleteRenderOwner("surface registry"))?;
        world
            .resource::<RunenUiPaintSubmissionRegistryResource>()
            .map_err(|_| UiSurfaceRetirementError::IncompleteRenderOwner("paint registry"))?;
        world
            .resource::<ManagedRenderProducerIds>()
            .map_err(|_| {
                UiSurfaceRetirementError::IncompleteRenderOwner("managed producer leases")
            })?;
    }

    if associations.has_live_surface_binding(surface) {
        return Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::LiveBinding,
        ));
    }
    if associations.has_unsettled_surface_publication(surface) {
        return Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::UnsettledPublication,
        ));
    }
    match recipients.native_ui_retirement_state(native) {
        UiNativeInputRetirementState::Live => {
            return Ok(UiSurfaceRetirementStatus::Blocked(
                UiSurfaceRetirementBlocker::LiveNativeInput,
            ));
        }
        UiNativeInputRetirementState::Unsettled => {
            return Ok(UiSurfaceRetirementStatus::Blocked(
                UiSurfaceRetirementBlocker::UnsettledNativeInput,
            ));
        }
        UiNativeInputRetirementState::NoUiRecipient | UiNativeInputRetirementState::Settled => {}
    }
    if associations.has_withdrawal_for_surface(surface) {
        return Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::PendingWithdrawal,
        ));
    }
    // Includes global UI paint applying to this surface. An untracked
    // residual submission is not permission for physical retirement.
    if world
        .resource::<RunenUiPaintSubmissionRegistryResource>()
        .ok()
        .is_some_and(|paints| !paints.ordered_submissions_for_surface(surface).is_empty())
    {
        return Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::RendererPaint,
        ));
    }
    Ok(UiSurfaceRetirementStatus::Cleared)
}

/// Removes only exact, already-unbound target paint without a synthetic frame.
///
/// This operation deliberately does not unbind a live UI, drop an accepted
/// generation, fabricate RunenUI input finality, or force GPU detachment.
/// Refusal leaves the complete outstanding withdrawal queue unchanged.
pub(crate) fn settle_native_surface_ui_withdrawals_without_frame(
    world: &mut World,
    native: NativeWindowId,
    surface: RenderSurfaceId,
) -> Result<UiSurfaceRetirementStatus, UiSurfaceRetirementError> {
    let status = inspect_native_surface_ui_retirement(world, native, surface)?;
    if status
        != UiSurfaceRetirementStatus::Blocked(UiSurfaceRetirementBlocker::PendingWithdrawal)
    {
        return Ok(status);
    }
    // All fallible resource preflight is before consuming any obligation.
    world
        .resource::<RunenUiPaintSubmissionRegistryResource>()
        .map_err(|_| UiSurfaceRetirementError::IncompleteRenderOwner("paint registry"))?;
    world
        .resource::<ManagedRenderProducerIds>()
        .map_err(|_| {
                UiSurfaceRetirementError::IncompleteRenderOwner("managed producer leases")
            })?;

    let withdrawals = world
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .expect("preflight retained the UI presentation owner")
        .take_withdrawals_for_surface(surface);
    for (producer, target) in withdrawals {
        world
            .resource_mut::<RunenUiPaintSubmissionRegistryResource>()
            .expect("preflight retained Render paint owner")
            .remove_for_surface(&producer, target);
        retire_withdrawn_managed_producer(world, producer);
    }
    inspect_native_surface_ui_retirement(world, native, surface)
}

#[cfg(test)]
mod tests;
