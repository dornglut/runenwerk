use super::enemy_resolution::resolve_enemy_deaths;
use super::pickups::apply_pickup;
use crate::features::worldgen::plugin as worldgen;
use crate::{
    CavernMetaProfile, CavernRunConfig, CavernRunState, Health, InventoryRunState,
    LootTableRegistry, PickupKind, SpawnDirector,
};
use engine::prelude::{Entity, SimulationRng, SimulationSeed, World};

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
    let geometry = world.resource::<crate::CavernGeometryGraph>().unwrap();
    let runtime = world
        .resource::<crate::CavernGeometryRuntimeState>()
        .unwrap();
    let seal_id = runtime
        .extraction_seal_primitive
        .expect("extraction seal primitive should be tracked");
    let seal = geometry
        .primitive(seal_id)
        .expect("extraction seal primitive should still exist");
    assert!(!seal.enabled);
    assert!(runtime.edit_events.iter().any(|event| matches!(
        event.edit.kind,
        crate::GeometryEditKind::DisablePrimitive(id) if id == seal_id
    )));
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
