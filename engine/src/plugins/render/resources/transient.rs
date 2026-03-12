use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransientResourceClaim {
    pub id: String,
    pub owner_pass: String,
}

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct TransientResourceTracker {
    claims: BTreeMap<String, TransientResourceClaim>,
}

impl TransientResourceTracker {
    pub fn claim(&mut self, id: impl Into<String>, owner_pass: impl Into<String>) {
        let id = id.into();
        self.claims.insert(
            id.clone(),
            TransientResourceClaim {
                id,
                owner_pass: owner_pass.into(),
            },
        );
    }

    pub fn release(&mut self, id: &str) -> bool {
        self.claims.remove(id).is_some()
    }

    pub fn claims(&self) -> Vec<TransientResourceClaim> {
        self.claims.values().cloned().collect()
    }
}
