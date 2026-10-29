//! Render-owned issuance of producer identities for managed UI composition.
//! Explicit/expert producer callers remain responsible for their own IDs.
use std::collections::{BTreeMap, BTreeSet};

use crate::plugins::render::RenderFrameProducerId;

/// A monotonic lease of Render producer IDs. This owns neither UI slots nor paint.
#[derive(Debug, Default, runen_ecs::Resource)]
pub struct ManagedRenderProducerIds {
    last_issued: u64,
    leased: BTreeMap<RenderFrameProducerId, u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ManagedRenderProducerError {
    #[error("Render producer identity space is exhausted")]
    Exhausted,
    #[error("this integration owner already holds a managed Render producer")]
    AlreadyLeased,
}

impl ManagedRenderProducerIds {
    /// Allocates a non-reused producer identity, excluding all externally occupied
    /// producer identities collected from the existing Render/UI source registries.
    /// No publication or target attachment is implied.
    pub fn allocate(
        &mut self,
        owner: u64,
        externally_occupied: &BTreeSet<RenderFrameProducerId>,
    ) -> Result<RenderFrameProducerId, ManagedRenderProducerError> {
        // A slot can have only one managed presentation binding. An expert
        // unbind does not implicitly release its producer lease; refuse to
        // mint a second one until that first lease is explicitly retired.
        if self.leased.values().any(|held_by| *held_by == owner) {
            return Err(ManagedRenderProducerError::AlreadyLeased);
        }
        loop {
            let raw = self
                .last_issued
                .checked_add(1)
                .ok_or(ManagedRenderProducerError::Exhausted)?;
            self.last_issued = raw;
            let id = RenderFrameProducerId::try_from_raw(raw)
                .map_err(|_| ManagedRenderProducerError::Exhausted)?;
            if !self.leased.contains_key(&id) && !externally_occupied.contains(&id) {
                self.leased.insert(id, owner);
                return Ok(id);
            }
        }
    }

    /// Exposes active Render leases for source admission validation.
    pub fn active_leases(&self) -> impl Iterator<Item = (RenderFrameProducerId, u64)> + '_ {
        self.leased.iter().map(|(&id, &owner)| (id, owner))
    }

    /// An opaque integration owner is part of the lease identity.
    /// A matching raw Render ID owned by a different UI slot is not authorization.
    #[must_use]
    pub fn is_owned_by(&self, id: RenderFrameProducerId, owner: u64) -> bool {
        self.leased.get(&id) == Some(&owner)
    }

    /// Retires only the caller's exact lease; old values are never reissued.
    pub fn release_owned(&mut self, id: RenderFrameProducerId, owner: u64) -> bool {
        if !self.is_owned_by(id, owner) {
            return false;
        }
        self.leased.remove(&id);
        true
    }

    /// Reconciles a managed lease after an expert already removed its binding.
    pub fn release_for_owner(&mut self, owner: u64) -> bool {
        let id = self
            .leased
            .iter()
            .find_map(|(id, held_by)| (*held_by == owner).then_some(*id));
        match id {
            Some(id) => self.leased.remove(&id).is_some(),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn producer_leases_skip_existing_identities_and_never_reuse_retired_ids() {
        let mut leases = ManagedRenderProducerIds::default();
        let occupied = BTreeSet::from([RenderFrameProducerId::try_from_raw(1).unwrap()]);
        let first = leases.allocate(101, &occupied).unwrap();
        assert_eq!(first.raw(), 2);
        assert!(leases.is_owned_by(first, 101));
        assert_eq!(
            leases.allocate(101, &BTreeSet::new()),
            Err(ManagedRenderProducerError::AlreadyLeased)
        );
        assert!(!leases.is_owned_by(first, 102));
        assert!(!leases.release_owned(first, 102));
        assert!(leases.release_owned(first, 101));
        assert!(!leases.is_owned_by(first, 101));
        let second = leases.allocate(102, &BTreeSet::new()).unwrap();
        assert_eq!(second.raw(), 3);
        assert!(leases.release_for_owner(102));
        assert!(!leases.is_owned_by(second, 102));
    }

    #[test]
    fn producer_id_exhaustion_does_not_wrap() {
        let mut leases = ManagedRenderProducerIds {
            last_issued: u64::MAX,
            leased: BTreeMap::new(),
        };
        assert_eq!(
            leases.allocate(1, &BTreeSet::new()),
            Err(ManagedRenderProducerError::Exhausted)
        );
    }
}
