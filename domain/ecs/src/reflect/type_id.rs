//! File: domain/ecs/src/reflect/type_id.rs
//! Purpose: Stable runtime identifier for reflected ECS types.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ReflectTypeId(pub u64);
