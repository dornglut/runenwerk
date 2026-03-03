/// Marker trait for component types.
pub trait Component: 'static {
    fn component_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}
