//! Owner-side correlation of mounted RunenUI publications with Engine presentation.
//!
//! This is an integration ledger, not a renderer, compositor, input router, or
//! replacement for RunenUI's surface/input publication authority. Completion
//! evidence is supplied by the one renderer-owned RunenGPU observation path.

use std::collections::BTreeMap;

use runenui_core::{StyleEnvironment, SurfaceId, SurfaceInputContext};
use runenui_runtime::{PaintRevision, SurfacePublication};

use crate::plugins::render::host::RenderSurfaceId;
use crate::plugins::render::{
    RenderFrameProducerId, RunenUiPublicationId, SurfaceFrameRoute, SurfaceFrameSubmissionOrder,
};
use crate::runtime::NativeWindowId;

use super::UiRuntimeSlotId;

/// The producer, target, and exact host style assigned to one mounted UI slot.
#[derive(Clone, Debug, PartialEq)]
pub struct UiRuntimePresentationBinding {
    pub slot_id: UiRuntimeSlotId,
    pub producer_id: RenderFrameProducerId,
    pub render_surface_id: RenderSurfaceId,
    pub route: SurfaceFrameRoute,
    pub order: SurfaceFrameSubmissionOrder,
    pub style: StyleEnvironment,
}

impl UiRuntimePresentationBinding {
    #[must_use]
    pub fn new(
        slot_id: UiRuntimeSlotId,
        producer_id: RenderFrameProducerId,
        render_surface_id: RenderSurfaceId,
        style: StyleEnvironment,
    ) -> Self {
        Self {
            slot_id,
            producer_id,
            render_surface_id,
            route: SurfaceFrameRoute::Screen,
            order: SurfaceFrameSubmissionOrder::default(),
            style,
        }
    }

    #[must_use]
    pub const fn with_route(mut self, route: SurfaceFrameRoute) -> Self {
        self.route = route;
        self
    }

    #[must_use]
    pub const fn with_order(mut self, order: SurfaceFrameSubmissionOrder) -> Self {
        self.order = order;
        self
    }
}

/// A mapping of the exact attached render target to its native scale.
/// NativeWindowId, RenderSurfaceId and RunenUI SurfaceId remain distinct.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiRuntimeNativeMapping {
    native_window_id: NativeWindowId,
    render_surface_id: RenderSurfaceId,
    target_size_px: (u32, u32),
    native_scale_factor: f64,
}

impl UiRuntimeNativeMapping {
    /// Validates the physical target and native-scale inputs before UI publication.
    ///
    /// # Errors
    ///
    /// Zero physical dimensions, non-finite scale, and non-positive scale reject.
    pub fn new(
        native_window_id: NativeWindowId,
        render_surface_id: RenderSurfaceId,
        target_size_px: (u32, u32),
        native_scale_factor: f64,
    ) -> Result<Self, UiRuntimeMappingError> {
        if target_size_px.0 == 0 || target_size_px.1 == 0 {
            return Err(UiRuntimeMappingError::EmptySurface);
        }
        if !native_scale_factor.is_finite() || native_scale_factor <= 0.0 {
            return Err(UiRuntimeMappingError::InvalidScale);
        }
        Ok(Self {
            native_window_id,
            render_surface_id,
            target_size_px,
            native_scale_factor,
        })
    }

    #[must_use]
    pub const fn native_window_id(self) -> NativeWindowId {
        self.native_window_id
    }

    #[must_use]
    pub const fn render_surface_id(self) -> RenderSurfaceId {
        self.render_surface_id
    }

    #[must_use]
    pub const fn target_size_px(self) -> (u32, u32) {
        self.target_size_px
    }

    #[must_use]
    pub const fn native_scale_factor(self) -> f64 {
        self.native_scale_factor
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum UiRuntimeMappingError {
    #[error("a RunenUI render target must have positive physical dimensions")]
    EmptySurface,
    #[error("a RunenUI native raster scale must be positive and finite")]
    InvalidScale,
}

#[derive(Clone, Debug)]
pub struct UiRuntimePendingPresentation {
    binding: UiRuntimePresentationBinding,
    mapping: UiRuntimeNativeMapping,
    publication: SurfacePublication,
    ingress_epoch: u64,
    publication_id: RunenUiPublicationId,
}

impl UiRuntimePendingPresentation {
    #[must_use]
    pub const fn publication(&self) -> &SurfacePublication {
        &self.publication
    }

    #[must_use]
    pub const fn binding(&self) -> &UiRuntimePresentationBinding {
        &self.binding
    }

    #[must_use]
    pub const fn mapping(&self) -> UiRuntimeNativeMapping {
        self.mapping
    }

    #[must_use]
    pub const fn ingress_epoch(&self) -> u64 {
        self.ingress_epoch
    }

    #[must_use]
    pub const fn publication_id(&self) -> RunenUiPublicationId {
        self.publication_id
    }
}

/// An accepted displayed input context, never inferred from prepared paint.
#[derive(Clone, Debug)]
pub struct UiRuntimeDisplayedPresentation {
    pub slot_id: UiRuntimeSlotId,
    pub producer_id: RenderFrameProducerId,
    pub render_surface_id: RenderSurfaceId,
    pub mapping: UiRuntimeNativeMapping,
    pub input_context: SurfaceInputContext,
    pub publication_id: RunenUiPublicationId,
    pub paint_revision: PaintRevision,
    pub surface_id: SurfaceId,
}

/// Renderer-integration receipt, to be created only from the exact successful
/// RunenRender 2D contribution and the same terminal RunenGPU Present submission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiRuntimePresentedReceipt {
    pub slot_id: UiRuntimeSlotId,
    pub producer_id: RenderFrameProducerId,
    pub render_surface_id: RenderSurfaceId,
    pub surface_id: SurfaceId,
    pub paint_revision: PaintRevision,
    pub publication_id: RunenUiPublicationId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum UiRuntimePresentationAssociationError {
    #[error("UI runtime slot is not bound to the claimed presentation target")]
    MissingBinding,
    #[error("UI runtime slot or render surface no longer matches the staged publication")]
    TargetChanged,
    #[error("RunenUI slots cannot share the same producer and render surface")]
    DuplicateProducer,
    #[error("RunenUI publication correlation identity exhausted")]
    PublicationIdentityExhausted,
}

#[derive(Debug, Default, runen_ecs::Resource)]
pub struct UiRuntimePresentationAssociationsResource {
    bindings: BTreeMap<UiRuntimeSlotId, UiRuntimePresentationBinding>,
    pending: BTreeMap<UiRuntimeSlotId, UiRuntimePendingPresentation>,
    /// Retains complete source/input snapshots only for renderer-accepted GPU
    /// submissions. The GPU observation owner bounds the accepted population.
    in_flight: BTreeMap<RunenUiPublicationId, UiRuntimePendingPresentation>,
    displayed: BTreeMap<UiRuntimeSlotId, UiRuntimeDisplayedPresentation>,
    current_mappings: BTreeMap<UiRuntimeSlotId, UiRuntimeNativeMapping>,
    last_publication: BTreeMap<UiRuntimeSlotId, (UiRuntimeNativeMapping, u64)>,
    /// Render-submission withdrawals owed after unbind/rebind. Never lose ownership cleanup.
    withdrawn: Vec<(RenderFrameProducerId, RenderSurfaceId)>,
    last_issued_publication_id: u64,
}

impl UiRuntimePresentationAssociationsResource {
    /// Rejects an ambiguous producer/surface ownership before any state mutation.
    ///
    /// # Errors
    ///
    /// A different slot may not claim the same producer on the same render surface.
    pub fn bind(
        &mut self,
        binding: UiRuntimePresentationBinding,
    ) -> Result<(), UiRuntimePresentationAssociationError> {
        if self.bindings.values().any(|existing| {
            existing.slot_id != binding.slot_id
                && existing.producer_id == binding.producer_id
                && existing.render_surface_id == binding.render_surface_id
        }) {
            return Err(UiRuntimePresentationAssociationError::DuplicateProducer);
        }
        let slot = binding.slot_id;
        if self.bindings.get(&slot) != Some(&binding) {
            if let Some(previous) = self.bindings.get(&slot) {
                self.withdrawn
                    .push((previous.producer_id, previous.render_surface_id));
            }
            self.pending.remove(&slot);
            self.in_flight
                .retain(|_, pending| pending.binding.slot_id != slot);
            self.displayed.remove(&slot);
            self.current_mappings.remove(&slot);
            self.last_publication.remove(&slot);
        }
        self.bindings.insert(slot, binding);
        Ok(())
    }

    pub fn unbind(&mut self, slot: UiRuntimeSlotId) {
        if let Some(previous) = self.bindings.remove(&slot) {
            self.withdrawn
                .push((previous.producer_id, previous.render_surface_id));
        }
        self.pending.remove(&slot);
        self.in_flight
            .retain(|_, pending| pending.binding.slot_id != slot);
        self.displayed.remove(&slot);
        self.current_mappings.remove(&slot);
        self.last_publication.remove(&slot);
    }

    /// Returns and clears Render withdrawal obligations; called before publishing live bindings.
    /// The outbox persists while the Engine is running headlessly without Render installed.
    pub(crate) fn take_withdrawals(&mut self) -> Vec<(RenderFrameProducerId, RenderSurfaceId)> {
        std::mem::take(&mut self.withdrawn)
    }

    #[must_use]
    pub fn binding(&self, slot: UiRuntimeSlotId) -> Option<&UiRuntimePresentationBinding> {
        self.bindings.get(&slot)
    }

    pub fn bindings(&self) -> impl Iterator<Item = &UiRuntimePresentationBinding> {
        self.bindings.values()
    }

    /// A changed physical mapping invalidates pending work and guards displayed
    /// input against the previous native transform, without inventing input facts.
    /// True when a new physical mapping invalidated the previous paint intake.
    /// The caller MUST retire that surface's renderer paint before publishing again.
    pub fn update_mapping(
        &mut self,
        slot: UiRuntimeSlotId,
        mapping: UiRuntimeNativeMapping,
    ) -> bool {
        let changed = self.current_mappings.insert(slot, mapping) != Some(mapping);
        if changed {
            self.pending.remove(&slot);
            self.in_flight
                .retain(|_, pending| pending.binding.slot_id != slot);
            self.last_publication.remove(&slot);
        }
        changed
    }

    /// Refuses old pending/display mappings as soon as the host surface detaches.
    pub fn invalidate_mapping(&mut self, slot: UiRuntimeSlotId) {
        self.in_flight
            .retain(|_, pending| pending.binding.slot_id != slot);
        self.pending.remove(&slot);
        self.current_mappings.remove(&slot);
        self.last_publication.remove(&slot);
    }

    /// Returns whether a new host ingress generation or native mapping needs publication.
    /// The runtime remains authority for internal paint/redraw facts.
    #[must_use]
    pub fn needs_publication(
        &self,
        slot: UiRuntimeSlotId,
        mapping: UiRuntimeNativeMapping,
        ingress_epoch: u64,
    ) -> bool {
        self.last_publication.get(&slot) != Some(&(mapping, ingress_epoch))
    }

    /// Retains the *whole* aligned publication; only its paint sibling goes to Render.
    ///
    /// # Errors
    ///
    /// Unbound or changed target/host mapping rejects without replacing pending work.
    pub fn stage(
        &mut self,
        binding: UiRuntimePresentationBinding,
        mapping: UiRuntimeNativeMapping,
        ingress_epoch: u64,
        publication: SurfacePublication,
    ) -> Result<RunenUiPublicationId, UiRuntimePresentationAssociationError> {
        if self.bindings.get(&binding.slot_id) != Some(&binding) {
            return Err(UiRuntimePresentationAssociationError::MissingBinding);
        }
        if binding.render_surface_id != mapping.render_surface_id
            || self.current_mappings.get(&binding.slot_id) != Some(&mapping)
        {
            return Err(UiRuntimePresentationAssociationError::TargetChanged);
        }
        let slot = binding.slot_id;
        let next = self
            .last_issued_publication_id
            .checked_add(1)
            .and_then(RunenUiPublicationId::try_from_raw)
            .ok_or(UiRuntimePresentationAssociationError::PublicationIdentityExhausted)?;
        self.pending.insert(
            slot,
            UiRuntimePendingPresentation {
                binding,
                mapping,
                publication,
                ingress_epoch,
                publication_id: next,
            },
        );
        self.last_publication.insert(slot, (mapping, ingress_epoch));
        self.last_issued_publication_id = next.raw();
        Ok(next)
    }

    #[must_use]
    pub fn pending(&self, slot: UiRuntimeSlotId) -> Option<&UiRuntimePendingPresentation> {
        self.pending.get(&slot)
    }

    /// Retains the immutable aligned source/input snapshot after the renderer
    /// reports its exact GPU submission was accepted. This is only a correlation
    /// transition; Accepted is never a displayed-input receipt.
    pub(crate) fn associate_accepted_submission(
        &mut self,
        publication_id: RunenUiPublicationId,
    ) -> bool {
        let Some(slot) = self.pending.iter().find_map(|(&slot, pending)| {
            (pending.publication_id == publication_id).then_some(slot)
        }) else {
            return false;
        };
        let Some(snapshot) = self.pending.remove(&slot) else {
            return false;
        };
        if self.bindings.get(&slot) != Some(&snapshot.binding)
            || self.current_mappings.get(&slot) != Some(&snapshot.mapping)
        {
            return false;
        }
        self.in_flight.insert(publication_id, snapshot);
        true
    }

    #[must_use]
    pub(crate) fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    /// Promotes only an exact GPU-accepted immutable input snapshot after
    /// renderer-proven terminal completion and Present. A stale/cross-slot/
    /// cross-surface receipt never consumes the newer pending generation.
    pub(crate) fn promote(
        &mut self,
        receipt: &UiRuntimePresentedReceipt,
    ) -> Option<&UiRuntimeDisplayedPresentation> {
        let pending = self.in_flight.get(&receipt.publication_id)?;
        if pending.binding.slot_id != receipt.slot_id {
            return None;
        }
        let paint = pending.publication.paint_publication();
        if self.bindings.get(&receipt.slot_id) != Some(&pending.binding)
            || self.current_mappings.get(&receipt.slot_id) != Some(&pending.mapping)
            || pending.binding.producer_id != receipt.producer_id
            || pending.binding.render_surface_id != receipt.render_surface_id
            || paint.surface_id() != &receipt.surface_id
            || paint.revision() != receipt.paint_revision
            || pending.publication_id != receipt.publication_id
        {
            return None;
        }
        let accepted = UiRuntimeDisplayedPresentation {
            slot_id: receipt.slot_id,
            producer_id: receipt.producer_id,
            render_surface_id: receipt.render_surface_id,
            mapping: pending.mapping,
            input_context: pending.publication.input_context().clone(),
            publication_id: pending.publication_id,
            paint_revision: paint.revision(),
            surface_id: paint.surface_id().clone(),
        };
        // A late receipt for an older accepted frame must not regress the
        // input surface after a newer frame already became displayed.
        if self
            .displayed
            .get(&receipt.slot_id)
            .is_some_and(|displayed| displayed.publication_id >= receipt.publication_id)
        {
            self.in_flight.remove(&receipt.publication_id);
            return None;
        }
        self.displayed.insert(receipt.slot_id, accepted);
        self.in_flight.remove(&receipt.publication_id);
        self.displayed.get(&receipt.slot_id)
    }

    /// Accepts a renderer-owned successful GPU and terminal-Present publication
    /// correlation. All source and host coordinates are recovered from this
    /// ledger's still-live immutable pending publication, never caller-supplied.
    ///
    /// This must only be called for publication IDs received from Renderer
    /// GPU-observation completion; it is not a render submission signal.
    pub(crate) fn promote_completed_publication(
        &mut self,
        publication_id: RunenUiPublicationId,
    ) -> Option<&UiRuntimeDisplayedPresentation> {
        let pending = self.in_flight.get(&publication_id)?;
        let slot = pending.binding.slot_id;
        let paint = pending.publication.paint_publication();
        let receipt = UiRuntimePresentedReceipt {
            slot_id: slot,
            producer_id: pending.binding.producer_id,
            render_surface_id: pending.binding.render_surface_id,
            surface_id: paint.surface_id().clone(),
            paint_revision: paint.revision(),
            publication_id,
        };
        self.promote(&receipt)
    }

    /// Retires only the exact failed in-flight source generation. A stale failure
    /// cannot evict a newer pending publication, or previously displayed input.
    /// The host revision witness is invalidated so a failed submission will be
    /// republished on a later frame instead of silently reusing its old paint.
    pub(crate) fn reject_terminal_publication(
        &mut self,
        publication_id: RunenUiPublicationId,
    ) -> bool {
        let affected_slot = if let Some(in_flight) = self.in_flight.remove(&publication_id) {
            Some(in_flight.binding.slot_id)
        } else {
            let slot = self.pending.iter().find_map(|(&slot, pending)| {
                (pending.publication_id == publication_id).then_some(slot)
            });
            if let Some(slot) = slot {
                self.pending.remove(&slot);
            }
            slot
        };
        let Some(slot) = affected_slot else {
            return false;
        };
        let later_is_live = self
            .pending
            .get(&slot)
            .is_some_and(|pending| pending.publication_id > publication_id)
            || self.in_flight.values().any(|pending| {
                pending.binding.slot_id == slot && pending.publication_id > publication_id
            })
            || self
                .displayed
                .get(&slot)
                .is_some_and(|displayed| displayed.publication_id > publication_id);
        if !later_is_live {
            self.last_publication.remove(&slot);
        }
        true
    }

    /// Hides input authority immediately when the native mapping changes.
    #[must_use]
    pub fn displayed_for_mapping(
        &self,
        slot: UiRuntimeSlotId,
        current: UiRuntimeNativeMapping,
    ) -> Option<&UiRuntimeDisplayedPresentation> {
        let displayed = self.displayed.get(&slot)?;
        (displayed.mapping == current
            && self.current_mappings.get(&slot) == Some(&current)
            && self.bindings.get(&slot).is_some_and(|binding| {
                binding.producer_id == displayed.producer_id
                    && binding.render_surface_id == displayed.render_surface_id
            }))
        .then_some(displayed)
    }
}
