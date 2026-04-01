/// Marker trait for component types.
pub trait Component: 'static {
    fn component_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Opt-in marker trait for components that expose explicit lifecycle state.
pub trait StatefulComponent: Component {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ComponentState {
    pub generation: u64,
    pub version: u64,
}

/// Marker trait for world resource types.
pub trait Resource: 'static {
    fn resource_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}
