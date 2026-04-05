//! File: domain/ecs/src/reflect/mod.rs
//! Purpose: ECS reflection foundation.

pub mod field_info;
pub mod registry;
pub mod struct_info;
pub mod traits;
pub mod type_id;
pub mod type_info;
pub mod value;
pub mod primitives;
pub mod component_registration;
pub mod resource_registration;

pub use field_info::*;
pub use registry::*;
pub use struct_info::*;
pub use traits::*;
pub use type_id::*;
pub use type_info::*;
pub use value::*;
pub use component_registration::*;
pub use resource_registration::*;