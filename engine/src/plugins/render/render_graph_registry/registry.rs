use super::*;

// Owner: Engine Render Graph Registry - Registry Resource
pub struct RenderGraphRegistryResource {
    owners: BTreeMap<String, OwnerRenderGraphRegistration>,
    revision: u64,
}

impl std::fmt::Debug for RenderGraphRegistryResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderGraphRegistryResource")
            .field("owner_count", &self.owner_count())
            .field("revision", &self.revision())
            .finish()
    }
}

impl Default for RenderGraphRegistryResource {
    fn default() -> Self {
        Self {
            owners: BTreeMap::new(),
            revision: 0,
        }
    }
}

impl RenderGraphRegistryResource {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn owner_count(&self) -> usize {
        self.owners.len()
    }

    pub fn upsert_owner(&mut self, registration: OwnerRenderGraphRegistration) {
        self.owners.insert(registration.owner.clone(), registration);
        self.bump_revision();
    }

    pub fn register_feature_graph(&mut self, spec: RenderFeatureGraphSpec) {
        self.upsert_owner(spec.into_owner_registration());
    }

    pub fn replace_feature_graph(
        &mut self,
        feature: impl Into<RenderFeatureId>,
        spec: RenderFeatureGraphSpec,
    ) -> Result<()> {
        let expected = feature.into();
        if expected != spec.feature {
            bail!(
                "feature graph id mismatch: expected '{}', got '{}'",
                expected.as_str(),
                spec.feature.as_str()
            );
        }
        self.register_feature_graph(spec);
        Ok(())
    }

    pub fn clear_owner(&mut self, owner: &str) -> bool {
        let owner = owner.trim();
        if owner.is_empty() {
            return false;
        }
        let removed = self.owners.remove(owner).is_some();
        if removed {
            self.bump_revision();
        }
        removed
    }

    pub fn remove_feature_graph(&mut self, feature: impl AsRef<str>) -> bool {
        self.clear_owner(feature.as_ref())
    }

    pub fn clear(&mut self) {
        if self.owners.is_empty() {
            return;
        }
        self.owners.clear();
        self.bump_revision();
    }

    pub fn owners(&self) -> Vec<OwnerRenderGraphRegistration> {
        self.owners.values().cloned().collect()
    }

    fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }
}
