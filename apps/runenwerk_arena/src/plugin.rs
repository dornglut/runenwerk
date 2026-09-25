use engine::plugins::{ActionState, AppActionBindingsExt};
use engine::prelude::{
    App, Commands, CoreSet, FixedUpdate, Plugin, PreUpdate, Res, ResMut, SimulationTick,
    SystemConfigExt, SystemMobilityExt, WorldMut,
};
use runen_input::PhysicalKeyIdentity;

use crate::command::{TickCommandBatch, apply_game_commands};
use crate::input::{
    ACTION_INTERACT, ACTION_JUMP, ACTION_MOVE_DOWN, ACTION_MOVE_LEFT, ACTION_MOVE_RIGHT,
    ACTION_MOVE_UP, GameActionSnapshot, GameInputAccumulator,
};
use crate::player::{ArenaPlayer, LOCAL_PARTICIPANT_ID, ParticipantId, PlayerControlState};

#[derive(
    Debug, Clone, Default, PartialEq, Eq, engine::prelude::Component, engine::prelude::Resource,
)]
pub struct LastLocalCommandBatch(pub Option<TickCommandBatch>);

pub struct ArenaGamePlugin;

impl Plugin for ArenaGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameInputAccumulator>();
        app.init_resource::<LastLocalCommandBatch>();
        app.add_input_bindings([
            (ACTION_MOVE_LEFT, PhysicalKeyIdentity::code("KeyA")),
            (ACTION_MOVE_RIGHT, PhysicalKeyIdentity::code("KeyD")),
            (ACTION_MOVE_UP, PhysicalKeyIdentity::code("KeyW")),
            (ACTION_MOVE_DOWN, PhysicalKeyIdentity::code("KeyS")),
            (ACTION_JUMP, PhysicalKeyIdentity::code("Space")),
            (ACTION_INTERACT, PhysicalKeyIdentity::code("KeyE")),
        ]);
        app.add_systems(engine::prelude::Startup, spawn_local_player);
        app.add_systems(PreUpdate, collect_game_actions.after(CoreSet::Input));
        app.add_systems(
            FixedUpdate,
            run_local_game_tick
                .on_invoker_thread()
                .in_set(CoreSet::Simulation),
        );
    }
}

fn spawn_local_player(mut commands: Commands) {
    commands.spawn((
        ArenaPlayer {
            participant: LOCAL_PARTICIPANT_ID,
        },
        PlayerControlState::default(),
    ));
}

fn collect_game_actions(actions: Res<ActionState>, mut accumulator: ResMut<GameInputAccumulator>) {
    accumulator.collect(GameActionSnapshot::from_actions(&actions));
}

fn run_local_game_tick(mut world: WorldMut) -> Result<(), crate::command::GameCommandError> {
    let tick = *world
        .resource::<SimulationTick>()
        .expect("ArenaGamePlugin requires SimulationPlugin");

    let mut accumulator = world
        .remove_resource::<GameInputAccumulator>()
        .expect("ArenaGamePlugin should retain GameInputAccumulator");
    let batch = accumulator.form_command_batch(LOCAL_PARTICIPANT_ID, tick);
    world.insert_resource(accumulator);

    apply_game_commands(&mut world, tick, &batch)?;

    *world
        .resource_mut::<LastLocalCommandBatch>()
        .expect("ArenaGamePlugin should retain LastLocalCommandBatch") =
        LastLocalCommandBatch(Some(batch));
    Ok(())
}

pub fn player_state_for(
    world: &engine::prelude::World,
    participant: ParticipantId,
) -> Option<PlayerControlState> {
    let query = world.query::<(&ArenaPlayer, &PlayerControlState)>();
    query
        .iter(world)
        .find_map(|(player, state)| (player.participant == participant).then_some(*state))
}
