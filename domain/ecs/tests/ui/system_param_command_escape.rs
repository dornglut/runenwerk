#[derive(ecs::Resource)]
struct EscapedCommands(Option<ecs::Commands<'static>>);

fn escape(mut escaped: ecs::ResMut<'_, EscapedCommands>, commands: ecs::Commands<'_>) {
    escaped.0 = Some(commands);
}

fn main() {}
