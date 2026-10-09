//! RenderPrepare publication bridge for explicitly bound RunenUI runtime slots.
//!
//! This system runs headlessly without Render/native resources installed. Actual
//! host presentation maps are read only from the attached RenderSurface and its
//! corresponding native window record, never from primary-window convenience facts.

use anyhow::{Result, anyhow};
use runenui_runtime::{LogicalSize, RasterScale, SurfaceBuildContext};

use crate::plugins::render::host::{RenderSurfaceLifecycleState, RenderSurfaceRegistryResource};
use crate::plugins::render::{
    Gfx, RunenUiPaintSubmission, RunenUiPaintSubmissionRegistryResource,
    SurfaceFrameSubmissionRegistryResource,
};
use crate::runtime::{NativeWindowLifecycleState, WindowStateRegistryResource, WorldMut};

use super::{
    UiRuntimeNativeMapping, UiRuntimePresentationAssociationsResource,
    UiRuntimePresentationBinding, UiRuntimeSlotsResource,
};

/// Applies only renderer-owned terminal GPU/F2/Present correlation IDs after
/// the RenderSubmit owner has completed. A submitted/accepted frame never enters
/// this API; all original source fields are recovered from UiPlugin associations.
///
/// # Errors
///
/// Returns a resource access failure when an installed RunenUI association
/// resource unexpectedly becomes unavailable.
pub fn apply_runenui_terminal_presentations_system(mut world: WorldMut) -> Result<()> {
    let (accepted, completed, rejected) = match world.resource_mut::<Gfx>() {
        Ok(gfx) => gfx.take_runenui_terminal_publications(),
        Err(_) => return Ok(()),
    };
    if accepted.is_empty() && completed.is_empty() && rejected.is_empty() {
        return Ok(());
    }
    let associations = world.resource_mut::<UiRuntimePresentationAssociationsResource>()?;
    for publication_id in accepted {
        // Input remains pending/in-flight; GPU submission acceptance alone is
        // never a displayed-input receipt.
        let _ = associations.associate_accepted_submission(publication_id);
    }
    for publication_id in completed {
        // Stale completion never promotes another publication generation.
        let _ = associations.promote_completed_publication(publication_id);
    }
    for (publication_id, reason) in rejected {
        if associations.reject_terminal_publication(publication_id) {
            tracing::warn!(
                publication_id = publication_id.raw(),
                reason,
                "RunenUI GPU publication retired without displayed-input promotion"
            );
        }
    }
    Ok(())
}

/// Publishes changed bound slot generations with exact native mapping, then
/// installs only their RunenUI renderer-facing paint in the typed Render intake.
/// Runtime publication owns redraw acknowledgment; GPU completion remains later.
///
/// # Errors
///
/// Fails closed for invalid attached scale/extent, conflicting producer families,
/// rejected publication or lost association, without manufacturing a fake frame.
pub fn publish_runenui_bound_surfaces_system(mut world: WorldMut) -> Result<()> {
    let bindings = match world.resource::<UiRuntimePresentationAssociationsResource>() {
        Ok(associations) => associations.bindings().cloned().collect::<Vec<_>>(),
        Err(_) => return Ok(()),
    };
    if world
        .resource::<RunenUiPaintSubmissionRegistryResource>()
        .is_err()
    {
        // Headless U3 clients do not need any Render or native resource.
        // The association owner retains withdrawals until Render is available.
        return Ok(());
    }

    // Remove paint owned by unbound or replaced slots *even if no bindings remain*.
    // Without this outbox, an unbind leaves an indefinitely live renderer submission.
    let withdrawals = world
        .resource_mut::<UiRuntimePresentationAssociationsResource>()?
        .take_withdrawals();
    for (producer, surface) in withdrawals {
        world
            .resource_mut::<RunenUiPaintSubmissionRegistryResource>()?
            .remove_for_surface(&producer, surface);
    }

    for binding in bindings {
        let slot = binding.slot_id;
        let mounted = world
            .resource::<UiRuntimeSlotsResource>()
            .ok()
            .is_some_and(|slots| slots.contains(slot));
        if !mounted {
            withdraw_binding(&mut world, &binding);
            continue;
        }

        let mapping = current_attached_mapping(&world, &binding)?;
        let Some(mapping) = mapping else {
            withdraw_paint_and_mapping(&mut world, &binding);
            continue;
        };

        let epoch = world
            .resource::<UiRuntimeSlotsResource>()?
            .ingress_epoch(slot)
            .ok_or_else(|| anyhow!("RunenUI slot disappeared during publication"))?;
        let (mapping_changed, needs_host_publication) = {
            let association = world.resource_mut::<UiRuntimePresentationAssociationsResource>()?;
            let changed = association.update_mapping(slot, mapping);
            (changed, association.needs_publication(slot, mapping, epoch))
        };
        if mapping_changed {
            // Never reuse paint from an old physical extent/scale while a new
            // source publication is still pending or has failed.
            world
                .resource_mut::<RunenUiPaintSubmissionRegistryResource>()?
                .remove_for_surface(&binding.producer_id, binding.render_surface_id);
        }
        let runtime_redraw_pending = if needs_host_publication {
            false
        } else {
            world
                .resource_mut::<UiRuntimeSlotsResource>()?
                .has_pending_redraw(slot)
                .map_err(|error| anyhow!("RunenUI redraw query failed: {error:?}"))?
        };
        if !needs_host_publication && !runtime_redraw_pending {
            continue;
        }

        // Fail before advancing the RunenUI publication if the same Engine
        // producer has an active predecessor frame submission on this surface.
        let collision = world
            .resource::<SurfaceFrameSubmissionRegistryResource>()
            .ok()
            .is_some_and(|legacy| {
                legacy
                    .get_for_surface(&binding.producer_id, binding.render_surface_id)
                    .is_some()
                    || legacy
                        .get(&binding.producer_id)
                        .is_some_and(|submission| submission.render_surface_id.is_none())
            });
        if collision {
            return Err(anyhow!(
                "UI producer {:?} has both legacy and RunenUI submissions on surface {:?}",
                binding.producer_id,
                binding.render_surface_id
            ));
        }

        let context = publication_context(&binding, mapping)?;
        let publication_attempt = world
            .resource_mut::<UiRuntimeSlotsResource>()?
            .publish_surface_for_slot(slot, &context);
        let publication = match publication_attempt {
            Ok(publication) => publication,
            Err(error) => {
                // No old RunenUI paint may cross a failed fresh publication.
                // Redraw remains runtime-owned and pending until a later success.
                withdraw_paint_and_mapping(&mut world, &binding);
                return Err(anyhow!("RunenUI bound publication rejected: {error:?}"));
            }
        };
        let paint = publication.paint_publication().clone();

        // The same producer/surface may not silently overwrite a distinct
        // already-owned RunenUI source publication. Retained same-source
        // replacement is valid; foreign source identity is not.
        let foreign_source = world
            .resource::<RunenUiPaintSubmissionRegistryResource>()?
            .get_for_surface(&binding.producer_id, binding.render_surface_id)
            .is_some_and(|previous| previous.publication.surface_id() != paint.surface_id());
        if foreign_source {
            // Publication has succeeded, and RunenUI has already acknowledged
            // the matching redraw. Retire the foreign paint and reset the host
            // association so a subsequent attempt cannot silently reuse it.
            withdraw_paint_and_mapping(&mut world, &binding);
            return Err(anyhow!(
                "RunenUI producer {:?} already belongs to a different source surface",
                binding.producer_id
            ));
        }
        let stage_result = world
            .resource_mut::<UiRuntimePresentationAssociationsResource>()?
            .stage(binding.clone(), mapping, epoch, publication);
        let publication_id = match stage_result {
            Ok(id) => id,
            Err(error) => {
                withdraw_paint_and_mapping(&mut world, &binding);
                return Err(anyhow!(
                    "RunenUI publication lost its presentation binding: {error}"
                ));
            }
        };

        world
            .resource_mut::<RunenUiPaintSubmissionRegistryResource>()?
            .replace(
                RunenUiPaintSubmission::new(binding.producer_id, publication_id, paint)
                    .with_render_surface(binding.render_surface_id)
                    .with_route(binding.route)
                    .with_order(binding.order),
            );
    }
    Ok(())
}

fn current_attached_mapping(
    world: &WorldMut,
    binding: &UiRuntimePresentationBinding,
) -> Result<Option<UiRuntimeNativeMapping>> {
    let Ok(surfaces) = world.resource::<RenderSurfaceRegistryResource>() else {
        return Ok(None);
    };
    let Some(surface) = surfaces.record(binding.render_surface_id) else {
        return Ok(None);
    };
    if surface.lifecycle_state != RenderSurfaceLifecycleState::Attached {
        return Ok(None);
    }
    let Ok(windows) = world.resource::<WindowStateRegistryResource>() else {
        return Ok(None);
    };
    let Some(window) = windows.record(surface.native_window_id) else {
        return Ok(None);
    };
    if window.lifecycle_state != NativeWindowLifecycleState::Created
        || window.size_px != surface.target_size_px
    {
        return Ok(None);
    }
    Ok(Some(UiRuntimeNativeMapping::new(
        surface.native_window_id,
        surface.render_surface_id,
        surface.target_size_px,
        window.scale_factor,
    )?))
}

fn publication_context<'a>(
    binding: &'a UiRuntimePresentationBinding,
    mapping: UiRuntimeNativeMapping,
) -> Result<SurfaceBuildContext<'a>> {
    let native_scale = mapping.native_scale_factor();
    let (width, height) = mapping.target_size_px();
    // RunenUI's accepted raster scale and logical lengths are f32 while F2's
    // target admission uses an exact f64 product and ceil. Derive the logical
    // extent from the *accepted* raster, then admit only values whose product
    // resolves back to the exact host-owned physical extent.
    let raster_scale = RasterScale::new(native_scale as f32)?;
    let logical_width = exact_logical_axis_for_pixels(width, raster_scale.get())?;
    let logical_height = exact_logical_axis_for_pixels(height, raster_scale.get())?;
    let size = LogicalSize::try_new(logical_width, logical_height)?;
    Ok(SurfaceBuildContext::tight(&binding.style, size).with_raster_scale(raster_scale))
}

/// Produces an f32 logical coordinate whose *exact* F2 logical-to-physical
/// product rounds to this host-owned extent, or fails without changing it.
/// An upward-rounded f32 quotient would otherwise generate a phantom pixel
/// on fractional-DPI targets and make F2 target admission fail.
fn exact_logical_axis_for_pixels(physical: u32, raster: f32) -> Result<f32> {
    if physical == 0 || !raster.is_finite() || raster <= 0.0 {
        return Err(anyhow!("RunenUI physical/raster axis is invalid"));
    }
    let wanted = f64::from(physical);
    let scale = f64::from(raster);
    let estimate = (wanted / scale) as f32;
    if !estimate.is_finite() || estimate <= 0.0 {
        return Err(anyhow!("RunenUI logical target axis is not representable"));
    }
    let extent = |logical: f32| (f64::from(logical) * scale).ceil();
    if extent(estimate) == wanted {
        return Ok(estimate);
    }
    // Positive, finite IEEE-754 f32 values have strictly ordered bit patterns.
    // If nearest f32 rounded above the exact quotient, one predecessor is
    // enough to test the adjacent lower representable logical coordinate.
    let lower = f32::from_bits(estimate.to_bits() - 1);
    if lower > 0.0 && extent(lower) == wanted {
        return Ok(lower);
    }
    Err(anyhow!(
        "RunenUI logical target axis cannot preserve the exact native extent at this raster scale"
    ))
}

fn withdraw_paint_and_mapping(world: &mut WorldMut, binding: &UiRuntimePresentationBinding) {
    if let Ok(registry) = world.resource_mut::<RunenUiPaintSubmissionRegistryResource>() {
        registry.remove_for_surface(&binding.producer_id, binding.render_surface_id);
    }
    if let Ok(associations) = world.resource_mut::<UiRuntimePresentationAssociationsResource>() {
        associations.invalidate_mapping(binding.slot_id);
    }
}

fn withdraw_binding(world: &mut WorldMut, binding: &UiRuntimePresentationBinding) {
    // Unbind enqueues exactly the previous renderer-owned paint identity. Drain
    // that obligation immediately rather than removing once now and again on
    // the next frame, which could retire a newly rebound owner's publication.
    let withdrawals = if let Ok(associations) =
        world.resource_mut::<UiRuntimePresentationAssociationsResource>()
    {
        associations.unbind(binding.slot_id);
        associations.take_withdrawals()
    } else {
        Vec::new()
    };
    if let Ok(registry) = world.resource_mut::<RunenUiPaintSubmissionRegistryResource>() {
        for (producer, surface) in withdrawals {
            registry.remove_for_surface(&producer, surface);
        }
    }
}

#[cfg(test)]
mod exact_raster_extent_tests {
    use super::exact_logical_axis_for_pixels;

    #[test]
    fn native_fractional_dpi_preserves_exact_f2_target_pixels() {
        for raster in [1.0_f32, 1.25, 1.5, 1.75, 2.0, 2.25, 2.75, 3.0] {
            for physical in [
                1_u32, 2, 3, 7, 21, 320, 321, 399, 640, 1279, 1280, 2049, 4096,
            ] {
                let logical = exact_logical_axis_for_pixels(physical, raster)
                    .expect("ordinary native target axis must remain exactly representable");
                assert_eq!(
                    (f64::from(logical) * f64::from(raster)).ceil(),
                    f64::from(physical),
                    "native={physical}, scale={raster}, logical={logical}"
                );
            }
        }
    }

    #[test]
    fn invalid_or_unrepresentable_axes_fail_closed() {
        assert!(exact_logical_axis_for_pixels(0, 1.0).is_err());
        assert!(exact_logical_axis_for_pixels(1, 0.0).is_err());
        assert!(exact_logical_axis_for_pixels(1, f32::INFINITY).is_err());
        assert!(exact_logical_axis_for_pixels(1, f32::NAN).is_err());
    }
}
