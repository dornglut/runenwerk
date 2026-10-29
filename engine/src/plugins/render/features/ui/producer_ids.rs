//! Render-owned issuance of producer identities for managed UI composition.
//! Explicit/expert producer callers remain responsible for their own IDs.
use std::collections::BTreeSet;

use crate::plugins::render::RenderFrameProducerId;

/// A monotonic lease of Render producer IDs. This owns neither UI slots nor paint.
#[derive(Debug, Default, runen_ecs::Resource)]
pub struct ManagedRenderProducerIds {
    last_issued: u64,
    leased: BTreeSet<RenderFrameProducerId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ManagedRenderProducerError {
    #[error("Render producer identity space is exhausted")]
    Exhausted,
}

impl ManagedRenderProducerIds {
    /// Allocates a non-reused producer identity, excluding all externally occupied
    /// producer identities collected from the existing Render/UI source registries.
    /// No publication or target attachment is implied.
    pub fn allocate(
        &mut self,
        externally_occupied: &BTreeSet<RenderFrameProducerId>,
    ) -> Result<RenderFrameProducerId, ManagedRenderProducerError> {
        loop {
            let raw = self
                .last_issued
                .checked_add(1)
                .ok_or(ManagedRenderProducerError::Exhausted)?;
            self.last_issued = raw;
            let id = RenderFrameProducerId::try_from_raw(raw)
                .map_err(|_| ManagedRenderProducerError::Exhausted)?;
            if !self.leased.contains(&id) && !externally_occupied.contains(&id) {
                self.leased.insert(id);
                return Ok(id);
            }
        }
    }

    #[must_use]
    pub fn is_leased(&self, id: RenderFrameProducerId) -> bool {
        self.leased.contains(&id)
    }

    /// Retires a managed producer identity without ever reissuing its old value.
    pub fn release(&mut self, id: RenderFrameProducerId) -> bool {
        self.leased.remove(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn producer_leases_skip_existing_identities_and_never_reuse_retired_ids() {
        let mut leases = ManagedRenderProducerIds::default();
        let occupied = BTreeSet::from([RenderFrameProducerId::try_from_raw(1).unwrap()]);
        let first = leases.allocate(&occupied).unwrap();
        assert_eq!(first.raw(), 2);
        assert!(leases.is_leased(first));
        assert!(leases.release(first));
        assert!(!leases.is_leased(first));
        assert_eq!(leases.allocate(&BTreeSet::new()).unwrap().raw(), 3);
    }

    #[test]
    fn producer_id_exhaustion_does_not_wrap() {
        let mut leases = ManagedRenderProducerIds {
            last_issued: u64::MAX,
            leased: BTreeSet::new(),
        };
        assert_eq!(
            leases.allocate(&BTreeSet::new()),
            Err(ManagedRenderProducerError::Exhausted)
        );
    }
}
