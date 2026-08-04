use crate::plugins::gpu::{
    GpuAdmittedProgramSource, GpuProgramSourceCause, GpuProgramSourceError,
    GpuProgramSourceIdentity, GpuProgramSourceKey, GpuProgramSourceOwnerId,
    GpuProgramSourceProvenance, GpuProgramSourceRegistry, GpuProgramSourceRegistryStats,
    GpuProgramSourceRevision,
};

/// Renderer-lifetime owner of the one bounded G4B source-consistency registry.
///
/// Runenwerk source production policy remains outside this type. The renderer
/// owns one process-local source-owner identity and one registry so consumer
/// flows cannot create parallel source-consistency authority. Every source
/// admitted through the retaining path remains reachable for the renderer
/// lifetime, independently of flow and backend-artifact cache retirement.
#[derive(Debug)]
pub(crate) struct RendererProgramSourceAuthority {
    owner: GpuProgramSourceOwnerId,
    registry: GpuProgramSourceRegistry,
    retained_sources: Vec<GpuAdmittedProgramSource>,
}

impl RendererProgramSourceAuthority {
    pub(crate) fn new(
        max_records: usize,
        max_retained_source_bytes: usize,
    ) -> Result<Self, GpuProgramSourceError> {
        Ok(Self {
            owner: GpuProgramSourceOwnerId::allocate()?,
            registry: GpuProgramSourceRegistry::new(max_records, max_retained_source_bytes)?,
            retained_sources: Vec::new(),
        })
    }

    pub(crate) fn admit_wgsl(
        &mut self,
        key: GpuProgramSourceKey,
        renderer_revision: u64,
        canonical_wgsl: impl Into<String>,
        provenance: GpuProgramSourceProvenance,
    ) -> Result<GpuAdmittedProgramSource, GpuProgramSourceError> {
        let normalized_revision = renderer_revision.checked_add(1).ok_or_else(|| {
            GpuProgramSourceError::Invalid {
                operation: "normalize renderer program-source revision",
                label: renderer_revision.to_string(),
                cause: GpuProgramSourceCause::InvalidSourceRevision,
                correction: "start a fresh renderer source owner before the zero-based revision space is exhausted",
            }
        })?;
        let identity = GpuProgramSourceIdentity::new(
            self.owner,
            key,
            GpuProgramSourceRevision::try_from_raw(normalized_revision)?,
        );
        self.registry
            .admit_wgsl(identity, canonical_wgsl, provenance)
    }

    pub(crate) fn admit_and_retain_wgsl(
        &mut self,
        key: GpuProgramSourceKey,
        renderer_revision: u64,
        canonical_wgsl: impl Into<String>,
        provenance: GpuProgramSourceProvenance,
    ) -> Result<GpuAdmittedProgramSource, GpuProgramSourceError> {
        let admitted = self.admit_wgsl(key, renderer_revision, canonical_wgsl, provenance)?;
        if !self
            .retained_sources
            .iter()
            .any(|retained| retained.is_same_record(&admitted))
        {
            self.retained_sources.push(admitted.clone());
        }
        Ok(admitted)
    }

    pub(crate) const fn owner(&self) -> GpuProgramSourceOwnerId {
        self.owner
    }

    pub(crate) fn stats(&self) -> GpuProgramSourceRegistryStats {
        self.registry.stats()
    }

    pub(crate) fn retained_source_count(&self) -> usize {
        self.retained_sources.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn authority() -> RendererProgramSourceAuthority {
        RendererProgramSourceAuthority::new(4, 4096)
            .expect("test source authority should construct")
    }

    fn key() -> GpuProgramSourceKey {
        GpuProgramSourceKey::new("renderer.test.shader").expect("test key should be valid")
    }

    fn provenance(detail: &str) -> GpuProgramSourceProvenance {
        GpuProgramSourceProvenance::new("renderer-program-source-test", Some(detail.to_owned()))
            .expect("test provenance should be valid")
    }

    #[test]
    fn renderer_revisions_normalize_into_one_nonzero_owner_domain() {
        let mut authority = authority();
        let fallback = authority
            .admit_wgsl(
                key(),
                0,
                "@compute @workgroup_size(1) fn cs_main() {}",
                provenance("fallback"),
            )
            .expect("fallback source should admit");
        let loaded = authority
            .admit_wgsl(
                key(),
                1,
                "@compute @workgroup_size(2) fn cs_main() {}",
                provenance("loaded"),
            )
            .expect("loaded source should admit");

        assert_eq!(fallback.identity().owner(), authority.owner());
        assert_eq!(loaded.identity().owner(), authority.owner());
        assert_eq!(fallback.identity().revision().get(), 1);
        assert_eq!(loaded.identity().revision().get(), 2);
        assert_eq!(authority.stats().retained_records(), 2);
        assert_eq!(authority.retained_source_count(), 0);
    }

    #[test]
    fn retaining_admission_is_idempotent_and_renderer_lifetime_owned() {
        let mut authority = authority();
        let source = "@compute @workgroup_size(1) fn cs_main() {}";
        let first = authority
            .admit_and_retain_wgsl(key(), 7, source, provenance("first consumer"))
            .expect("first source should admit");
        let repeated = authority
            .admit_and_retain_wgsl(key(), 7, source, provenance("second consumer"))
            .expect("identical source should remain idempotent");

        assert!(first.is_same_record(&repeated));
        assert_eq!(authority.stats().retained_records(), 1);
        assert_eq!(authority.retained_source_count(), 1);
    }

    #[test]
    fn identical_source_reuses_one_record_and_conflicting_revision_fails_closed() {
        let mut authority = authority();
        let source = "@compute @workgroup_size(1) fn cs_main() {}";
        let first = authority
            .admit_wgsl(key(), 7, source, provenance("first consumer"))
            .expect("first source should admit");
        let repeated = authority
            .admit_wgsl(key(), 7, source, provenance("second consumer"))
            .expect("identical source should remain idempotent");
        assert!(first.is_same_record(&repeated));

        let error = authority
            .admit_wgsl(
                key(),
                7,
                "@compute @workgroup_size(8) fn cs_main() {}",
                provenance("conflict"),
            )
            .expect_err("conflicting source text must allocate a new renderer revision");
        assert_eq!(error.cause(), GpuProgramSourceCause::SourceRevisionConflict);
    }

    #[test]
    fn renderer_revision_overflow_is_explicit_and_non_mutating() {
        let mut authority = authority();
        let error = authority
            .admit_wgsl(
                key(),
                u64::MAX,
                "@compute @workgroup_size(1) fn cs_main() {}",
                provenance("overflow"),
            )
            .expect_err("renderer revision overflow must fail closed");

        assert_eq!(error.cause(), GpuProgramSourceCause::InvalidSourceRevision);
        assert_eq!(authority.stats().retained_records(), 0);
        assert_eq!(authority.retained_source_count(), 0);
    }
}
