use super::*;

pub(crate) fn apply_runtime_geometry_edit(world: &mut World, edit: &GeometryEdit) -> bool {
    let mut graph = match world.resource_mut::<CavernGeometryGraph>() {
        Ok(graph) => graph,
        Err(_) => return false,
    };
    let affected = graph.apply_edit(edit);
    let revision = graph.revision;
    let world_bounds = graph.bounds;
    let event = GeometryEditEvent {
        revision,
        edit: edit.clone(),
    };
    drop(graph);

    if let Some(bounds) = affected
        && let Ok(mut field) = world.resource_mut::<CavernCollisionField>()
    {
        field.invalidate_bounds(bounds);
        field.revision_seen = revision;
        field.world_bounds = world_bounds;
    }

    if let Ok(mut runtime) = world.resource_mut::<CavernGeometryRuntimeState>() {
        runtime.edit_events.push(event);
    }
    true
}
