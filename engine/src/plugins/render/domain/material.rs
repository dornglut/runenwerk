#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderMaterialId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderMaterialDescriptor {
    pub id: RenderMaterialId,
    pub label: String,
}

impl RenderMaterialDescriptor {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: RenderMaterialId(id.into()),
            label: label.into(),
        }
    }
}
