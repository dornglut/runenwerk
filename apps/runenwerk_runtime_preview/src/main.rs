use anyhow::Result;
use runenwerk_runtime_preview::{RuntimePreviewConfig, RuntimePreviewHost, build_preview_app};
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    let headless = std::env::args().any(|arg| arg == "--headless");
    let config = RuntimePreviewConfig {
        headless,
        ..RuntimePreviewConfig::default()
    };
    let mut host = RuntimePreviewHost::spawn(config.clone())?;
    let mut stdout = std::io::stdout();
    writeln!(stdout, "{}", host.bootstrap().to_stdout_line()?)?;
    stdout.flush()?;

    let app = build_preview_app(config.headless);
    if config.headless {
        host.run_command_loop().await?;
        host.shutdown().await?;
        app.run_for_frames(1)?;
    } else {
        app.run()?;
        host.shutdown().await?;
    }
    Ok(())
}
