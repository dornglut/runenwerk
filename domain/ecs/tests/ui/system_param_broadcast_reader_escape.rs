#[derive(ecs::Resource)]
struct Escaped(Option<ecs::BroadcastReader<'static, 'static, u32>>);

fn escape(
    mut destination: ecs::ResMut<'_, Escaped>,
    reader: ecs::BroadcastReader<'_, '_, u32>,
) {
    destination.0 = Some(reader);
}

fn main() {}
