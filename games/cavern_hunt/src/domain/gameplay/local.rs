use ecs::Entity;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LocalPlayerRef {
	pub player_id: Option<u32>,
	pub entity: Option<Entity>,
}