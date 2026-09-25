use runenwerk_arena::build_game_app;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_game_app(false).run()?;
    Ok(())
}
