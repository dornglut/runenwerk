//! File: apps/runenwerk_editor/src/editor_runtime/realities/simulated.rs
//! Purpose: Read-only simulated-reality boundary for runtime world state.

#[derive(Clone, Copy)]
pub struct SimulatedSceneReality<'a> {
    world: &'a ecs::World,
}

impl<'a> SimulatedSceneReality<'a> {
    pub fn new(world: &'a ecs::World) -> Self {
        Self { world }
    }

    pub fn world(&self) -> &'a ecs::World {
        self.world
    }
}
