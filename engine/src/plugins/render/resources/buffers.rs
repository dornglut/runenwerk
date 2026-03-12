use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferResourceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferResourceEntry {
    pub id: BufferResourceId,
    pub label: String,
}

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct BufferResourceRegistry {
    entries: BTreeMap<BufferResourceId, BufferResourceEntry>,
}

impl BufferResourceRegistry {
    pub fn upsert(&mut self, id: impl Into<String>, label: impl Into<String>) {
        let id = BufferResourceId(id.into());
        self.entries.insert(
            id.clone(),
            BufferResourceEntry {
                id,
                label: label.into(),
            },
        );
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.entries
            .remove(&BufferResourceId(id.to_string()))
            .is_some()
    }

    pub fn entries(&self) -> Vec<BufferResourceEntry> {
        self.entries.values().cloned().collect()
    }
}
