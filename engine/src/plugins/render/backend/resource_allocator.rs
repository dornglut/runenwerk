use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureResourceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureResourceEntry {
    pub id: TextureResourceId,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferResourceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferResourceEntry {
    pub id: BufferResourceId,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransientResourceClaim {
    pub id: String,
    pub owner_pass: String,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct BackendResourceAllocatorResource {
    textures: BTreeMap<TextureResourceId, TextureResourceEntry>,
    buffers: BTreeMap<BufferResourceId, BufferResourceEntry>,
    transients: BTreeMap<String, TransientResourceClaim>,
}

impl BackendResourceAllocatorResource {
    pub fn upsert_texture(&mut self, id: impl Into<String>, label: impl Into<String>) {
        let id = TextureResourceId(id.into());
        self.textures.insert(
            id.clone(),
            TextureResourceEntry {
                id,
                label: label.into(),
            },
        );
    }

    pub fn upsert_buffer(&mut self, id: impl Into<String>, label: impl Into<String>) {
        let id = BufferResourceId(id.into());
        self.buffers.insert(
            id.clone(),
            BufferResourceEntry {
                id,
                label: label.into(),
            },
        );
    }

    pub fn claim_transient(&mut self, id: impl Into<String>, owner_pass: impl Into<String>) {
        let id = id.into();
        self.transients.insert(
            id.clone(),
            TransientResourceClaim {
                id,
                owner_pass: owner_pass.into(),
            },
        );
    }

    pub fn release_transient(&mut self, id: &str) -> bool {
        self.transients.remove(id).is_some()
    }

    pub fn remove_texture(&mut self, id: &str) -> bool {
        self.textures
            .remove(&TextureResourceId(id.to_string()))
            .is_some()
    }

    pub fn remove_buffer(&mut self, id: &str) -> bool {
        self.buffers
            .remove(&BufferResourceId(id.to_string()))
            .is_some()
    }

    pub fn texture_entries(&self) -> Vec<TextureResourceEntry> {
        self.textures.values().cloned().collect()
    }

    pub fn buffer_entries(&self) -> Vec<BufferResourceEntry> {
        self.buffers.values().cloned().collect()
    }

    pub fn transient_claims(&self) -> Vec<TransientResourceClaim> {
        self.transients.values().cloned().collect()
    }
}
