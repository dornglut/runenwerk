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

        // Resume previously accepted, capacity-paused RunenUI work on the
        // existing invoker-thread runtime before any paint/input publication.
        // A bounded pause is not a failed action and not terminal quiescence.
        let settlement = world
            .resource_mut::<UiRuntimeSlotsResource>()?
            .settle_for_publication(slot);
        match settlement {
            Ok(()) => {}
            Err(super::UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce) => {
                // No partially settled snapshot may be published. Preserve
                // only already-published compatible visual paint while
                // synchronously denying every old displayed-input generation.
                // A blank compositor frame must not replace intact UI merely
                // because M13 work has not yet finished its bounded pump.
                defer_unsettled_paint(&mut world, &binding, mapping, "pre-publication");
                continue;
            }
            Err(error) => {
                withdraw_paint_and_mapping(&mut world, &binding);
                return Err(anyhow!("RunenUI slot settlement failed: {error:?}"));
            }
        }

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
        let publication_attempt = publish_observed_surface(&mut world, slot, &context);
        let publication = match publication_attempt {
            Ok(publication) => publication,
            Err(error)
                if error.is::<super::native_transaction::NativeUiMutationNeedsSettlement>() =>
            {
                // Publishing can itself admit new M13 work. The canonical
                // transaction already revoked old gameplay access. Never
                // publish its unqualified result or promote stale input, but
                // keep previously staged visual paint on an unchanged target.
                defer_unsettled_paint(&mut world, &binding, mapping, "post-publication");
                continue;
            }
            Err(error) => {
                // Genuine binding loss, terminal runtime or other publication
                // failure remains fatal; no old paint can cross this boundary.
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

/// A selected native slot must settle publication-created pointer re-hit work
/// before the World transaction can certify its final ownership observation.
/// One ordinary bounded pump permits progress without retry-publication loops;
/// an incomplete pump still reaches the existing typed deferral path.
fn publish_observed_surface(
    world: &mut runen_ecs::World,
    slot: super::UiRuntimeSlotId,
    context: &SurfaceBuildContext<'_>,
) -> Result<runenui_runtime::SurfacePublication> {
    let selected = world
        .resource::<super::UiNativeInputRecipientsResource>()
        .ok()
        .and_then(|recipients| recipients.selected_slot_binding(slot))
        .is_some();
    super::transact_ui_slot(world, slot, |slots| {
        let publication = slots.publish_surface_for_slot(slot, context)?;
        if selected {
            match slots.settle_slot(slot) {
                Ok(()) | Err(super::UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(publication)
    })
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

/// A pending M13 observation revokes input, not necessarily the valid
/// previously authored visual contribution. No new snapshot or Present receipt
/// is synthesized; RunenUI and Render retain their existing semantic owners.
fn defer_unsettled_paint(
    world: &mut WorldMut,
    binding: &UiRuntimePresentationBinding,
    mapping: UiRuntimeNativeMapping,
    phase: &'static str,
) {
    let compatible = world
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .ok()
        .is_some_and(|associations| associations.defer_unsettled_publication(binding, mapping));
    let previous = world
        .resource::<RunenUiPaintSubmissionRegistryResource>()
        .ok()
        .and_then(|paints| paints.get_for_surface(&binding.producer_id, binding.render_surface_id))
        .map(|paint| paint.publication_id);
    if compatible && previous.is_some() {
        tracing::trace!(
            target: "engine.ui.publication",
            ?phase,
            slot = ?binding.slot_id,
            render_surface = ?binding.render_surface_id,
            native_window = ?mapping.native_window_id(),
            visual_publication = ?previous.map(|id| id.raw()),
            "RunenUI M13 pending; old visual retained without displayed-input permission"
        );
    } else {
        tracing::trace!(
            target: "engine.ui.publication",
            ?phase,
            slot = ?binding.slot_id,
            render_surface = ?binding.render_surface_id,
            compatible,
            has_prior_paint = previous.is_some(),
            "RunenUI M13 pending; incompatible/absent old paint retired"
        );
        withdraw_paint_and_mapping(world, binding);
    }
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

#[cfg(test)]
mod native_publication_tests {
    use crate::plugins::ui::{
        NativeInputRecipient, UiNativeInputRecipientsResource, UiRuntimeFontConfiguration,
        UiRuntimeSlotsResource, UiTypedScreenId, settle_native_ui_slot, transact_ui_slot,
    };
    use crate::runtime::WindowStateRegistryResource;
    use crate::runtime::native_host_wake::NativeHostClock;
    use runen_ecs::World;
    use runenui_core::{
        Element, InputDeviceId, LogicalPoint, NoHostProtocol, PointerDeviceKind, PointerEvent,
        PointerId, PointerPhase, StyleEnvironment, UiApp, View,
    };
    use runenui_runtime::{LogicalSize, SurfaceBuildContext};

    struct StationaryApp;
    impl UiApp for StationaryApp {
        type State = ();
        type Action = ();
        type HostProtocol = NoHostProtocol;
        fn root(_: &()) -> impl View<()> {
            runenui_core::column(Vec::<Element<()>>::new())
        }
        fn update(_: &mut (), _: ()) -> impl runenui_core::IntoUpdateOutput<(), NoHostProtocol> {}
    }

    #[test]
    fn native_stationary_pointer_publication_reaches_observable_m13_ownership() {
        for (native, selected, with_pointer) in [
            (false, false, false),
            (false, false, true),
            (true, false, false),
            (true, false, true),
            (true, true, false),
            (true, true, true),
        ] {
            let mut slots = UiRuntimeSlotsResource::default();
            if native {
                slots
                    .configure_host_clock(Some(NativeHostClock::new()))
                    .unwrap();
            }
            let slot = slots
                .mount_app::<StationaryApp>(
                    UiTypedScreenId::new("native.publication.stationary"),
                    (),
                    &UiRuntimeFontConfiguration::default(),
                )
                .slot_id()
                .unwrap();
            let mut windows = WindowStateRegistryResource::default();
            let window = windows.register_primary_window("stationary", (200, 120), 1.0, true);
            let mut recipients = UiNativeInputRecipientsResource::default();
            if selected {
                recipients
                    .bind(
                        window,
                        NativeInputRecipient::RunenUiSlot(slot),
                        &windows,
                        &mut slots,
                    )
                    .unwrap();
            }
            let mut world = World::new();
            world.insert_resource(slots);
            world.insert_resource(windows);
            world.insert_resource(recipients);
            let style = StyleEnvironment::default();
            let context =
                SurfaceBuildContext::tight(&style, LogicalSize::try_new(200.0, 120.0).unwrap());
            let first = super::publish_observed_surface(&mut world, slot, &context).unwrap();
            if with_pointer {
                let event = PointerEvent::new(
                    PointerId::new(1).unwrap(),
                    PointerDeviceKind::Mouse,
                    PointerPhase::Move,
                    LogicalPoint::new(10.0, 10.0).unwrap(),
                    first.input_context().clone(),
                )
                .with_device_id(InputDeviceId::new(1).unwrap());
                let scope = world
                    .resource_mut::<UiRuntimeSlotsResource>()
                    .unwrap()
                    .input_ownership(slot)
                    .unwrap()
                    .scope()
                    .clone();
                let submission = world
                    .resource_mut::<UiRuntimeSlotsResource>()
                    .unwrap()
                    .submit_native_pointer(slot, event)
                    .unwrap();
                assert!(
                    super::publish_observed_surface(&mut world, slot, &context).is_err(),
                    "publication must not consume an unsettled original native event"
                );
                let slots = world.resource_mut::<UiRuntimeSlotsResource>().unwrap();
                let (batch, status) = slots.pump_native_input_records(slot).unwrap();
                assert!(matches!(
                    crate::plugins::ui::native_pointer_finality(
                        &scope,
                        InputDeviceId::new(1).unwrap(),
                        PointerId::new(1).unwrap(),
                        first.input_context().surface_id(),
                        submission,
                        &batch,
                        &status
                    ),
                    Ok(runenui_runtime::UiInputFinality::Committed(_))
                ));
                assert_eq!(
                    batch
                        .ordered_records()
                        .iter()
                        .filter(|record| matches!(record,
                    runenui_runtime::InputArbitrationRecord::InputSettled(settled)
                        if settled.sequence() == submission.sequence()))
                        .count(),
                    1
                );
                assert_eq!(slots.input_ownership(slot).unwrap().pointers().len(), 1);
            }
            // A stationary stream must not make every new publication pending
            // forever: the normal next-frame settlement must permit progress.
            for cycle in 1..=3 {
                settle_native_ui_slot(&mut world, slot).unwrap();
                super::publish_observed_surface(&mut world, slot, &context).unwrap_or_else(
                    |error| panic!("pointer={with_pointer} cycle={cycle}: {error:#}"),
                );
                if !selected && with_pointer {
                    assert!(
                        world
                            .resource_mut::<UiRuntimeSlotsResource>()
                            .unwrap()
                            .input_ownership(slot)
                            .is_err(),
                        "ordinary publication retains its existing callback timing"
                    );
                    settle_native_ui_slot(&mut world, slot).unwrap();
                }
                let ownership = world
                    .resource_mut::<UiRuntimeSlotsResource>()
                    .unwrap()
                    .input_ownership(slot)
                    .unwrap();
                assert_eq!(ownership.pointers().len(), usize::from(with_pointer));
            }
        }
    }

    struct CallbackState {
        shown: bool,
        burst: usize,
    }
    enum CallbackAction {
        Reveal,
        Tick,
    }
    struct CallbackApp;
    impl UiApp for CallbackApp {
        type State = CallbackState;
        type Action = CallbackAction;
        type HostProtocol = NoHostProtocol;
        fn root(state: &CallbackState) -> impl View<CallbackAction> {
            if state.shown {
                Element::new(CallbackProbe(state.burst))
                    .id("rehit.target")
                    .key("rehit.target")
                    .focusable(true)
            } else {
                runenui_core::column(Vec::<Element<CallbackAction>>::new()).into_element()
            }
        }
        fn update(
            state: &mut CallbackState,
            action: CallbackAction,
        ) -> impl runenui_core::IntoUpdateOutput<CallbackAction, NoHostProtocol> {
            if matches!(action, CallbackAction::Reveal) {
                state.shown = true;
            }
        }
    }
    #[derive(Debug)]
    struct CallbackProbe(usize);
    impl runenui_core::Widget<CallbackAction> for CallbackProbe {
        type State = ();
        fn create_state(&self) {}
        fn measure(
            &self,
            _: &(),
            _: runenui_core::WidgetMeasureInput,
        ) -> runenui_core::WidgetMeasure {
            runenui_core::WidgetMeasure::measured(32_u16.into(), 32_u16.into())
        }
        fn hit_test(
            &self,
            _: &(),
            context: runenui_core::HitContributionContext,
        ) -> runenui_core::HitContribution {
            let size = context.local_size();
            runenui_core::HitContribution::single_rect(
                runenui_core::LogicalRect::try_new(0.0, 0.0, size.width(), size.height()).unwrap(),
            )
        }
        fn event(
            &mut self,
            _: &mut (),
            event: &runenui_core::UiEvent,
            context: &mut runenui_core::EventContext<'_, CallbackAction>,
        ) -> runenui_core::WidgetEventOutput {
            if context.phase() == runenui_core::EventPhase::Target
                && matches!(event, runenui_core::UiEvent::PointerBoundary(boundary)
                    if boundary.kind() == runenui_core::PointerBoundaryKind::Enter)
            {
                for _ in 0..self.0 {
                    context.emit(CallbackAction::Tick);
                }
                context.emit_command(runenui_core::SemanticCommand::RequestFocus);
            }
            runenui_core::WidgetEventOutput::none()
        }
    }

    #[test]
    fn native_publication_callback_revokes_game_and_defers_bounded_pending_work() {
        use crate::plugins::input::{ActionState, InputState, domain::action};
        use crate::plugins::render::host::{RenderSurfaceId, RenderSurfaceRegistryResource};
        use crate::plugins::render::{
            RenderFrameProducerId, RunenUiPaintSubmission, RunenUiPaintSubmissionRegistryResource,
        };
        use crate::plugins::ui::{
            UiRuntimeNativeMapping, UiRuntimePresentationAssociationsResource,
            UiRuntimePresentationBinding,
        };
        use crate::runtime::winit_input::WinitInputAdapter;
        use crate::runtime::{SystemMobilityExt, Update};
        use runen_input::{
            DigitalState, KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey,
            ObservationOrigin, PhysicalKeyIdentity,
        };
        for burst in [0, 130] {
            let mut slots = UiRuntimeSlotsResource::default();
            slots
                .configure_host_clock(Some(NativeHostClock::new()))
                .unwrap();
            let slot = slots
                .mount_app::<CallbackApp>(
                    UiTypedScreenId::new("native.rehit.callback"),
                    CallbackState {
                        shown: false,
                        burst,
                    },
                    &UiRuntimeFontConfiguration::default(),
                )
                .slot_id()
                .unwrap();
            let mut windows = WindowStateRegistryResource::default();
            let window = windows.register_primary_window("callback", (200, 120), 1.0, true);
            let mut recipients = UiNativeInputRecipientsResource::default();
            recipients
                .bind(
                    window,
                    NativeInputRecipient::RunenUiSlot(slot),
                    &windows,
                    &mut slots,
                )
                .unwrap();
            let mut adapter = WinitInputAdapter::default();
            let original = adapter
                .window_context(window, winit::event::DeviceId::dummy())
                .unwrap();
            let mut input = InputState::new();
            input.handle_normalized_keyboard(
                original,
                &KeyboardInput {
                    physical_key: PhysicalKeyIdentity::code("KeyW"),
                    logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
                    location: KeyLocation::Standard,
                    state: DigitalState::Pressed,
                    repeat: false,
                    origin: ObservationOrigin::SourceReport,
                },
            );
            let mut world = World::new();
            world.insert_resource(slots);
            world.insert_resource(windows);
            world.insert_resource(recipients);
            world.insert_resource(adapter);
            world.insert_resource(input);
            let style = StyleEnvironment::default();
            let context =
                SurfaceBuildContext::tight(&style, LogicalSize::try_new(200.0, 120.0).unwrap());
            let first = super::publish_observed_surface(&mut world, slot, &context).unwrap();
            let surface = RenderSurfaceId::primary();
            let producer = RenderFrameProducerId::try_from_raw(57).unwrap();
            let binding = UiRuntimePresentationBinding::new(slot, producer, surface, style.clone());
            let mapping = UiRuntimeNativeMapping::new(window, surface, (200, 120), 1.0).unwrap();
            let mut associations = UiRuntimePresentationAssociationsResource::default();
            associations.bind(binding.clone()).unwrap();
            associations.update_mapping(slot, mapping);
            let first_id = associations
                .stage(binding, mapping, 1, first.clone())
                .unwrap();
            assert!(associations.associate_accepted_submission(first_id));
            assert!(
                associations
                    .promote_completed_publication(first_id)
                    .is_some()
            );
            let mut paints = RunenUiPaintSubmissionRegistryResource::default();
            paints.replace(
                RunenUiPaintSubmission::new(producer, first_id, first.paint_publication().clone())
                    .with_render_surface(surface),
            );
            let mut surfaces = RenderSurfaceRegistryResource::default();
            surfaces
                .confirm_surface_attachment(surface, window, (200, 120))
                .unwrap();
            world.insert_resource(associations);
            world.insert_resource(paints);
            world.insert_resource(surfaces);
            let event = PointerEvent::new(
                PointerId::new(1).unwrap(),
                PointerDeviceKind::Mouse,
                PointerPhase::Move,
                LogicalPoint::new(10.0, 10.0).unwrap(),
                first.input_context().clone(),
            )
            .with_device_id(InputDeviceId::new(1).unwrap());
            let slots = world.resource_mut::<UiRuntimeSlotsResource>().unwrap();
            slots.submit_native_pointer(slot, event).unwrap();
            slots.pump_native_input_records(slot).unwrap().1.unwrap();
            transact_ui_slot(&mut world, slot, |slots| {
                slots.submit_app_action::<CallbackApp>(slot, CallbackAction::Reveal)
            })
            .unwrap();
            // Establish the pre-publication projected held permission. The
            // callback must revoke it in this same World transaction.
            let mut actions = ActionState::new();
            actions.project(world.resource::<InputState>().unwrap());
            assert!(actions.action_down(action::WORLD_MOVE_UP));
            world.insert_resource(actions);
            let mut runtime = runen_ecs::Runtime::new();
            runtime
                .add_systems(
                    Update,
                    super::publish_runenui_bound_surfaces_system.on_invoker_thread(),
                )
                .unwrap();
            runtime.run_schedule::<Update>(&mut world).unwrap();
            assert!(
                !world
                    .resource::<ActionState>()
                    .unwrap()
                    .action_down(action::WORLD_MOVE_UP)
            );
            assert!(
                world
                    .resource::<InputState>()
                    .unwrap()
                    .keyboard_down_generation_in(original, &PhysicalKeyIdentity::code("KeyW"))
                    .is_some(),
                "no physical Up is fabricated"
            );
            let associations = world
                .resource::<UiRuntimePresentationAssociationsResource>()
                .unwrap();
            if burst != 0 {
                assert!(
                    associations
                        .displayed_for_native_recipient(slot, window)
                        .is_none(),
                    "an unsettled callback revokes old completed input"
                );
                assert!(
                    associations.pending(slot).is_none(),
                    "over-budget callback cannot publish"
                );
                assert_eq!(
                    world
                        .resource::<RunenUiPaintSubmissionRegistryResource>()
                        .unwrap()
                        .get_for_surface(&producer, surface)
                        .unwrap()
                        .publication_id,
                    first_id,
                    "compatible old visual survives bounded deferral"
                );
                assert!(matches!(
                    world
                        .resource_mut::<UiRuntimeSlotsResource>()
                        .unwrap()
                        .input_ownership(slot),
                    Err(crate::plugins::ui::UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
                ));
                settle_native_ui_slot(&mut world, slot).unwrap();
                runtime.run_schedule::<Update>(&mut world).unwrap();
            }
            let slots = world.resource_mut::<UiRuntimeSlotsResource>().unwrap();
            assert!(
                slots
                    .input_ownership(slot)
                    .unwrap()
                    .keyboard()
                    .focused_node()
                    .is_some()
            );
            assert!(
                world
                    .resource::<UiRuntimePresentationAssociationsResource>()
                    .unwrap()
                    .pending(slot)
                    .is_some(),
                "a fresh publication becomes available after bounded settlement"
            );
        }
    }
}
