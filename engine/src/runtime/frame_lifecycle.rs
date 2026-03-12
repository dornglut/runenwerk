use crate::runtime::fixed_step_executor::run_fixed_update_frame;
use crate::runtime::schedules::{
    FrameEnd, PreUpdate, RenderPrepare, RenderSubmit, Startup, Update,
};
use crate::runtime::window::WindowState;
use anyhow::Result;
use ecs::{Runtime, World};

/// Applies builtin runtime run-state before startup/frame execution.
///
/// This does not install resources. Builtin resources are installed by
/// `App::install_builtin_resources` during app construction.
pub(crate) fn prepare_world_for_run(world: &mut World, title: &str, headless: bool) {
    if let Ok(mut window) = world.resource_mut::<WindowState>() {
        window.set_headless(headless);
        window.redraw_requested = false;
        window.close_requested = false;
        window.title = title.to_string();
    }
}

/// Runs `Startup` at most once for a runtime state.
pub(crate) fn run_startup_if_needed(
    world: &mut World,
    scheduler: &mut Runtime,
    startup_ran: &mut bool,
) -> Result<()> {
    if *startup_ran {
        return Ok(());
    }

    scheduler.run_schedule::<Startup>(world)?;
    *startup_ran = true;
    Ok(())
}

/// Runs one runtime frame using the canonical schedule order:
///
/// 1. `PreUpdate`
/// 2. fixed-step loop (`FixedUpdate` zero or more times)
/// 3. `Update`
/// 4. `RenderPrepare`
/// 5. `RenderSubmit`
/// 6. `FrameEnd`
pub(crate) fn run_frame(world: &mut World, scheduler: &mut Runtime) -> Result<()> {
    scheduler.run_schedule::<PreUpdate>(world)?;
    run_fixed_update_frame(world, scheduler)?;
    scheduler.run_schedule::<Update>(world)?;
    scheduler.run_schedule::<RenderPrepare>(world)?;
    scheduler.run_schedule::<RenderSubmit>(world)?;
    scheduler.run_schedule::<FrameEnd>(world)?;
    Ok(())
}
