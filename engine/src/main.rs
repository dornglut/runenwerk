use anyhow::Result;
use winit::event_loop::{ControlFlow, EventLoop};
use engine::platform::App;
use engine::utils::setup_tracing;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();

    event_loop.run_app(&mut app)?;

    Ok(())
}