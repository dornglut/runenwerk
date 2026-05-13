//! File: domain/procgen/src/ids.rs
//! Purpose: Stable typed identifiers for procgen documents and planning artifacts.

macro_rules! procgen_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $name(pub u64);

        impl $name {
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }

            pub const fn raw(self) -> u64 {
                self.0
            }

            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }
        }
    };
}

procgen_id!(ProcgenDocumentId);
procgen_id!(ProcgenGeneratorId);
procgen_id!(ProcgenReservationId);
procgen_id!(ProcgenCandidateId);
procgen_id!(ProcgenRealizationId);
