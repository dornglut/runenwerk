use anyhow::Result;
use engine::platform::App;

fn main() -> Result<()> {
    App::new().run()
}
