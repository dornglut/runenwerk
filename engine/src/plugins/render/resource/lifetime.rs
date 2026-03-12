#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceLifetime {
    Imported,
    Persistent,
    Transient,
}

impl ResourceLifetime {
    pub fn is_imported(self) -> bool {
        matches!(self, Self::Imported)
    }

    pub fn is_persistent(self) -> bool {
        matches!(self, Self::Persistent)
    }

    pub fn is_transient(self) -> bool {
        matches!(self, Self::Transient)
    }
}
