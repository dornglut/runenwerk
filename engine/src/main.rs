use anyhow::Result;
use engine::platform::App;
use engine::utils::setup_tracing;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<()> {
    let _tracing_guard = setup_tracing();
    tracing::info!("starting engine (no default plugins)");
    tracing::warn!("run `cargo run -p game` for the fully wired game runtime");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).map_err(Into::into)
}
