#[derive(ecs::Resource)]
struct Counter;

#[derive(ecs::SystemParam)]
struct Unrelated<'a> {
    value: ecs::Res<'a, Counter>,
}

fn main() {}
