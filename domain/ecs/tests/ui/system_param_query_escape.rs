#[derive(ecs::Component)]
struct Position;

#[derive(ecs::Resource)]
struct Escaped(Option<ecs::Query<'static, 'static, &'static mut Position>>);

fn escape(
    mut destination: ecs::ResMut<'_, Escaped>,
    query: ecs::Query<'_, '_, &'static mut Position>,
) {
    destination.0 = Some(query);
}

fn main() {}
