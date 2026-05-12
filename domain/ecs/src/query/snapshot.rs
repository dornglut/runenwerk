use std::any::TypeId;

use crate::{query::QueryAccess, world::World};

pub fn query_snapshot_source_generation(world: &World, access: &QueryAccess) -> u64 {
    let component_types = accessed_component_types(access);
    let resource_types = accessed_resource_types(access);
    let mut generation = 0;

    for change in world.component_changes_since(0) {
        if contains_type(&component_types, change.component_type) {
            generation = generation.max(change.tick);
        }
    }
    for change in world.resource_changes_since(0) {
        if contains_type(&resource_types, change.resource_type) {
            generation = generation.max(change.tick);
        }
    }

    generation
}

fn accessed_component_types(access: &QueryAccess) -> Vec<TypeId> {
    access
        .component_reads()
        .iter()
        .chain(access.component_writes())
        .chain(access.orphaned_component_reads())
        .map(|access| access.type_id())
        .collect()
}

fn accessed_resource_types(access: &QueryAccess) -> Vec<TypeId> {
    access
        .resource_reads()
        .iter()
        .chain(access.resource_writes())
        .map(|access| access.type_id())
        .collect()
}

fn contains_type(types: &[TypeId], type_id: TypeId) -> bool {
    types.contains(&type_id)
}
