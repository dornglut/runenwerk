/// Marker trait for resource types.
pub trait Resource: 'static {}

impl<T: 'static> Resource for T {}
