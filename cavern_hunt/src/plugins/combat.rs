use crate::domain::{
    AimTarget2, CAVERN_GAMEPLAY_HEIGHT, CavernAimState, CavernCollisionField, CavernControlState,
    CavernGeometryGraph, CavernLayout, CavernRunPhase, CavernRunState,
    CavernServerAppliedInputTickMap, CavernServerControlMap, ColliderRadius, DamageFeedbackState,
    DashState, EnemyKind, Faction, Health, HitFlashState, LocalPlayerRef, PlayerActive,
    PlayerCombatTuning, PlayerCompanion, PlayerId, PlayerSpectator, Projectile,
    ProjectileVisualState, Transform2, Velocity2, WeaponState, is_active_player_entity,
};
use crate::plugins::render_sdf;
use crate::plugins::timing::fixed_step_seconds;
use anyhow::Result;
use engine::prelude::{
    App, AuthorityRole, CoreSet, Entity, FixedUpdate, InputState, Plugin, PreUpdate,
    SimulationProfileConfig, SimulationTick, SystemConfigExt, WindowState, World, WorldMut,
};
use std::collections::BTreeSet;

include!("combat_internal/plugin_aim.rs");

include!("combat_internal/control_step.rs");

include!("combat_internal/movement_fire.rs");

include!("combat_internal/projectiles.rs");

fn combat_fixed_step_seconds(world: &World) -> f32 {
    fixed_step_seconds(world)
}

include!("combat_internal/tests.rs");
