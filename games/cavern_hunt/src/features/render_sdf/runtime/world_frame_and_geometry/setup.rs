use super::*;

// Owner: Cavern Hunt SDF Renderer - Resource Setup
pub(crate) fn setup_render_resources(world: &mut World) -> Result<()> {
    if world.resource::<CavernSdfWorldFrame>().is_err() {
        world.insert_resource(CavernSdfWorldFrame::default());
    }
    Ok(())
}
