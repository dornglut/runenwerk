use crate::features::render_sdf::plugin as render_sdf;
use crate::features::timing::fixed_step_seconds;
use crate::{
    AimTarget2, CAVERN_GAMEPLAY_HEIGHT, CavernAimState, CavernCollisionField, CavernControlState,
    CavernGeometryGraph, CavernLayout, CavernRunPhase, CavernRunState,
    CavernServerAppliedInputTickMap, CavernServerControlMap, ColliderRadius, DamageFeedbackState,
    DashState, EnemyKind, Faction, Health, HitFlashState, LocalPlayerRef, PlayerActive,
    PlayerCombatTuning, PlayerCompanion, PlayerId, PlayerSpectator, Projectile,
    ProjectileVisualState, Transform2, Velocity2, WeaponState, is_active_player_entity,
};
use anyhow::Result;
use engine::prelude::{
    App, AuthorityRole, CoreSet, Entity, FixedUpdate, InputState, Plugin, PreUpdate,
    SimulationProfileConfig, SimulationTick, SystemConfigExt, WindowState, World, WorldMut,
};
use std::collections::BTreeSet;

#[path = "runtime/mod.rs"]
mod runtime;

pub use runtime::CavernHuntCombatPlugin;
pub(crate) use runtime::{constrained_move, replay_predicted_local_frame, spawn_projectile};

fn combat_fixed_step_seconds(world: &World) -> f32 {
    fixed_step_seconds(world)
}
