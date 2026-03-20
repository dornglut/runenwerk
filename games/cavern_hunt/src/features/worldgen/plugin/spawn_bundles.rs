use super::*;

#[derive(Bundle, ecs::Resource)]
pub(super) struct PlayerSpawnBundle {
    pub(super) player: Player,
    pub(super) player_id: PlayerId,
    pub(super) player_roster_identity: PlayerRosterIdentity,
    pub(super) transform: Transform2,
    pub(super) velocity: Velocity2,
    pub(super) health: Health,
    pub(super) faction: Faction,
    pub(super) collider_radius: ColliderRadius,
    pub(super) aim_target: AimTarget2,
    pub(super) dash_state: DashState,
    pub(super) weapon_state: WeaponState,
    pub(super) inventory: InventoryRunState,
    pub(super) room_anchor: RoomAnchor,
}

#[derive(Bundle, ecs::Resource)]
pub(super) struct EnemySpawnBundle {
    pub(super) enemy: Enemy,
    pub(super) enemy_kind: EnemyKind,
    pub(super) transform: Transform2,
    pub(super) velocity: Velocity2,
    pub(super) health: Health,
    pub(super) faction: Faction,
    pub(super) collider_radius: ColliderRadius,
    pub(super) aggro_state: AggroState,
    pub(super) spawn_room: SpawnRoom,
    pub(super) room_anchor: RoomAnchor,
}
