mod bindings;
mod debug_views;
mod extract;
mod fields;
mod materials;
mod raymarch;

pub use bindings::*;
pub use debug_views::*;
pub use extract::*;
pub use fields::*;
pub use materials::*;
pub use raymarch::*;

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct SdfRenderFeatureState {
    pub enabled: bool,
    pub last_graph_revision: u64,
}
