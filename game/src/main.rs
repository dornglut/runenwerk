use anyhow::Result;
use engine::platform::App;
use engine::utils::setup_tracing;
use game::plugins::full_game_plugins;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<()> {
    let _tracing_guard = setup_tracing();
    tracing::info!("starting grotto game");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::with_plugins(full_game_plugins());
    event_loop.run_app(&mut app).map_err(Into::into)
}
