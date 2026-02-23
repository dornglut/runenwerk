/// Marker trait for world-level singleton resources.
pub trait Resource: 'static {}

impl<T: 'static> Resource for T {}
