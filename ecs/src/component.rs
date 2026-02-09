// component.rs
use std::any::Any;

/// Trait for components
pub trait Component: Any + Send + Sync {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + Send + Sync> Component for T {
	fn as_any(&self) -> &dyn Any { self }
	fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// Trait for type-erased component vectors
pub trait ComponentVec {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Component> ComponentVec for Vec<T> {
	fn as_any(&self) -> &dyn Any { self }
	fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
