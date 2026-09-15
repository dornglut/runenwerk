use runen_ecs::SystemSet;

/// Stable system labels reserved for the engine-owned UI runtime.
#[derive(Debug, Copy, Clone, PartialEq, Eq, SystemSet)]
pub enum UiRuntimeSet {
    Foundation,
    Report,
    RenderPublication,
}
