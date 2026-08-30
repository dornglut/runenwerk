use anyhow::{Result, anyhow};
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
        let run_result = host.run_command_loop().await;
        let shutdown_result = host.shutdown().await;
        finish_with_shutdown(run_result, shutdown_result)?;
        app.run_for_frames(1)?;
    } else {
        let run_result = app.run();
        let shutdown_result = host.shutdown().await;
        finish_with_shutdown(run_result, shutdown_result)?;
    }
    Ok(())
}

fn finish_with_shutdown<T>(run_result: Result<T>, shutdown_result: Result<()>) -> Result<T> {
    match (run_result, shutdown_result) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Err(run_error), Err(shutdown_error)) => Err(anyhow!(
            "runtime preview failed: {run_error:#}; shutdown also failed: {shutdown_error:#}"
        )),
    }
}
