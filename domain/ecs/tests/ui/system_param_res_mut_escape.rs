#[derive(ecs::Resource)]
struct Counter;

#[derive(ecs::Resource)]
struct Escaped(Option<ecs::ResMut<'static, Counter>>);

fn escape(mut destination: ecs::ResMut<'_, Escaped>, value: ecs::ResMut<'_, Counter>) {
    destination.0 = Some(value);
}

fn main() {}
