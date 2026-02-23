/// Marker trait for ECS component types.
pub trait Component: 'static {
    /// Human-readable component name used in registry metadata.
    fn component_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}
