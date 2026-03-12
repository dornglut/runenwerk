use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureResourceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureResourceEntry {
    pub id: TextureResourceId,
    pub label: String,
}

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct TextureResourceRegistry {
    entries: BTreeMap<TextureResourceId, TextureResourceEntry>,
}

impl TextureResourceRegistry {
    pub fn upsert(&mut self, id: impl Into<String>, label: impl Into<String>) {
        let id = TextureResourceId(id.into());
        self.entries.insert(
            id.clone(),
            TextureResourceEntry {
                id,
                label: label.into(),
            },
        );
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.entries
            .remove(&TextureResourceId(id.to_string()))
            .is_some()
    }

    pub fn entries(&self) -> Vec<TextureResourceEntry> {
        self.entries.values().cloned().collect()
    }
}
