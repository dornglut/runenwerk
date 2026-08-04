use crate::plugins::gpu::{
    GpuProgramSourceError, GpuProgramSourceOwnerId, GpuProgramSourceRegistry,
    GpuProgramSourceRegistryStats,
};

/// Renderer-lifetime owner of the one bounded G4B source-consistency registry.
///
/// Runenwerk source production policy remains outside this type. The renderer
/// owns one process-local source-owner identity and one registry so consumer
/// flows cannot create parallel source-consistency authority.
#[derive(Debug)]
pub(crate) struct RendererProgramSourceAuthority {
    owner: GpuProgramSourceOwnerId,
    registry: GpuProgramSourceRegistry,
}

impl RendererProgramSourceAuthority {
    pub(crate) fn new(
        max_records: usize,
        max_retained_source_bytes: usize,
    ) -> Result<Self, GpuProgramSourceError> {
        Ok(Self {
            owner: GpuProgramSourceOwnerId::allocate()?,
            registry: GpuProgramSourceRegistry::new(max_records, max_retained_source_bytes)?,
        })
    }

    pub(crate) const fn owner(&self) -> GpuProgramSourceOwnerId {
        self.owner
    }

    pub(crate) fn stats(&self) -> GpuProgramSourceRegistryStats {
        self.registry.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_source_authority_is_bounded_empty_and_process_local() {
        let authority = RendererProgramSourceAuthority::new(4, 4096)
            .expect("test source authority should construct");
        let stats = authority.stats();

        assert_ne!(authority.owner().diagnostic_raw(), 0);
        assert_eq!(stats.retained_records(), 0);
        assert_eq!(stats.retained_source_bytes(), 0);
        assert_eq!(stats.max_records(), 4);
        assert_eq!(stats.max_retained_source_bytes(), 4096);
    }

    #[test]
    fn renderer_source_authorities_receive_distinct_owner_identity() {
        let first = RendererProgramSourceAuthority::new(1, 1)
            .expect("first test source authority should construct");
        let second = RendererProgramSourceAuthority::new(1, 1)
            .expect("second test source authority should construct");

        assert_ne!(first.owner(), second.owner());
    }
}
