use super::*;
use engine::plugins::world::WorldRuntimeSet;

// Owner: Cavern Hunt Combat Plugin - Plugin Wiring and Local Aim
pub struct CavernHuntCombatPlugin;

impl Plugin for CavernHuntCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_local_aim_system.in_set(CoreSet::Input));
        app.add_systems(
            FixedUpdate,
            fixed_step_combat_system
                .in_set(CoreSet::Simulation)
                .after(WorldRuntimeSet::BuildIntegrate)
                .before(CoreSet::Replication),
        );
    }
}

fn update_local_aim_system(mut world: WorldMut) -> Result<()> {
    update_local_aim(&mut world)
}

pub(crate) fn update_local_aim(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if matches!(authority, AuthorityRole::Server) {
        return Ok(());
    }

    let layout = world.resource::<CavernLayout>()?.clone();
    let camera = world.resource::<crate::CavernCameraState>()?.clone();
    let window_size = {
        let window = world.resource::<WindowState>()?;
        window.clone()
    };
    let cursor = {
        let input = world.resource::<InputState>()?;
        input.mouse_position
    };
    let local_entity = world.resource::<LocalPlayerRef>()?.entity;

    let aim_world = render_sdf::project_mouse_to_world(&camera, &window_size, &layout, cursor);
    world.insert_resource(CavernAimState {
        world_point: aim_world,
    });
    let (movement, fire_pressed, dash_pressed) = {
        let input = world.resource::<InputState>()?;
        let movement = camera_relative_movement(&camera, &input);
        (
            [movement.0, movement.1],
            input.left_mouse_down(),
            input.right_mouse_down(),
        )
    };
    let next_tick = world
        .resource::<SimulationTick>()
        .copied()
        .map(|tick| SimulationTick(tick.0.saturating_add(1)))
        .unwrap_or_default();
    if let Ok(mut control) = world.resource_mut::<CavernControlState>() {
        control.movement = movement;
        control.aim_world = aim_world;
        control.fire_pressed = fire_pressed;
        control.dash_pressed = dash_pressed;
        control.source_tick = next_tick;
    }

    if let Some(entity) = local_entity {
        if let Some(mut aim) = world.get_mut::<AimTarget2>(entity) {
            aim.x = aim_world[0];
            aim.y = aim_world[1];
        }
    }

    Ok(())
}
