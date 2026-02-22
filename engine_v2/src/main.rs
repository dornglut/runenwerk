use anyhow::Result;
use engine_v2::platform::App;
use engine_v2::utils::setup_tracing;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<()> {
    setup_tracing();
    tracing::info!("starting engine_v2");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).map_err(Into::into)
}
