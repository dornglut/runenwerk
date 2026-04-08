#[derive(Debug, Clone, Default, ecs::Reflect)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::ReflectComponent)]
pub struct Position {
    pub value: Vec2,
    pub speed: f32,
    pub label: String,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::ReflectComponent)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}
