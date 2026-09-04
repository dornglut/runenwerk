#[derive(ecs::Resource)]
struct Counter;

#[derive(ecs::SystemParam)]
struct TooMany<'w, 's, 'a> {
    value: ecs::Res<'w, Counter>,
    other: ecs::Res<'a, Counter>,
    reader: ecs::BroadcastReader<'w, 's, u32>,
}

fn main() {}
