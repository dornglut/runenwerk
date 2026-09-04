#[derive(ecs::SystemParam)]
struct WrongOrder<'w, 's> {
    reader: ecs::BroadcastReader<'s, 'w, u32>,
}

fn main() {}
