use std::hash::Hash;

/// Marker trait for reusable spatial index keys.
///
/// Typical examples:
/// - ECS entity ids
/// - Godot object ids
/// - world object ids
/// - chunk ids
/// - replication ids
pub trait SpatialKey: Copy + Eq + Ord + Hash {}

impl<T> SpatialKey for T where T: Copy + Eq + Ord + Hash {}