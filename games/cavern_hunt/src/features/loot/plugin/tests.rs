use super::enemy_resolution::resolve_enemy_deaths;
use super::pickups::apply_pickup;
use crate::features::worldgen::plugin as worldgen;
use crate::{
    CavernMetaProfile, CavernRunConfig, CavernRunState, Health, InventoryRunState,
    LootTableRegistry, PickupKind, SpawnDirector,
};
use engine::plugins::world::adapters::resources::OperationLogResource;
use engine::prelude::{Entity, SimulationRng, SimulationSeed, World};
use world_ops::Operation;

#[test]
fn elite_death_activates_extraction() {
    let mut world = World::new();
    world.insert_resource(CavernRunConfig::default());
    world.insert_resource(CavernRunState::default());
    world.insert_resource(SpawnDirector::default());
    world.insert_resource(LootTableRegistry::default());
    world.insert_resource(CavernMetaProfile::default());
    world.insert_resource(crate::LocalPlayerRef::default());
    world.insert_resource(crate::CavernCameraState::default());
    world.insert_resource(OperationLogResource::default());
    world.insert_resource(SimulationRng::from_seed(SimulationSeed(7)));
    worldgen::initialize_run_world(&mut world, true).unwrap();

    let enemy_query = world.query_state::<(Entity, &crate::EnemyKind), ()>();
    let elite = enemy_query
        .iter(&world)
        .find(|(_, kind)| **kind == crate::EnemyKind::NestGuardian)
        .map(|(entity, _)| entity)
        .unwrap();
    world.get_mut::<Health>(elite).unwrap().current = 0.0;
    resolve_enemy_deaths(&mut world).unwrap();

    let run_state = world.resource::<CavernRunState>().unwrap();
    assert!(run_state.elite_defeated);
    assert!(run_state.extraction_active);
    let op_log = world.resource::<OperationLogResource>().unwrap();
    assert!(
        op_log
            .operations
            .iter()
            .any(|record| matches!(record.operation, Operation::CsgSubtract { .. })),
        "elite objective completion should submit a world-authoritative seal removal operation"
    );
}

#[test]
fn scrap_pickup_adds_to_inventory() {
    let mut world = World::new();
    world.insert_resource(CavernRunConfig::default());
    world.insert_resource(CavernRunState::default());
    world.insert_resource(SpawnDirector::default());
    world.insert_resource(LootTableRegistry::default());
    world.insert_resource(CavernMetaProfile::default());
    world.insert_resource(crate::LocalPlayerRef::default());
    world.insert_resource(crate::CavernCameraState::default());
    world.insert_resource(OperationLogResource::default());
    world.insert_resource(SimulationRng::from_seed(SimulationSeed(9)));
    worldgen::initialize_run_world(&mut world, true).unwrap();
    let player = world
        .resource::<crate::LocalPlayerRef>()
        .unwrap()
        .entity
        .unwrap();

    apply_pickup(&mut world, player, PickupKind::Scrap(5)).unwrap();

    let inventory = world.get::<InventoryRunState>(player).unwrap();
    assert_eq!(inventory.scrap, 5);
}
