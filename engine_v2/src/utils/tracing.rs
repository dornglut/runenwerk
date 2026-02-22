use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "tracy")]
const TRACY_ENV_VAR: &str = "ENGINE_V2_TRACY";

#[cfg(feature = "tracy")]
fn tracy_enabled() -> bool {
    std::env::var(TRACY_ENV_VAR)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

pub fn setup_tracing() -> Option<WorkerGuard> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let log_dir = Path::new("logs");
    if let Err(err) = std::fs::create_dir_all(log_dir) {
        eprintln!("failed creating log directory {}: {err}", log_dir.display());
    }
    let file_appender = tracing_appender::rolling::never(log_dir, "engine_v2.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let stdout_layer = fmt::layer().with_target(true).with_ansi(true);
    let file_layer = fmt::layer()
        .with_target(true)
        .with_ansi(false)
        .with_writer(file_writer);

    #[cfg(feature = "tracy")]
    {
        let enable_tracy = tracy_enabled();
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer)
            .with(enable_tracy.then_some(tracing_tracy::TracyLayer::default()));

        match subscriber.try_init() {
            Ok(()) => {
                if enable_tracy {
                    tracing::info!(
                        env = TRACY_ENV_VAR,
                        "tracy profiling enabled (use Tracy profiler to connect)"
                    );
                } else {
                    tracing::info!(
                        env = TRACY_ENV_VAR,
                        "tracy profiling disabled (set env to 1 to enable)"
                    );
                }
                Some(guard)
            }
            Err(err) => {
                eprintln!("failed to initialize tracing subscriber: {err}");
                None
            }
        }
    }

    #[cfg(not(feature = "tracy"))]
    {
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer);
        match subscriber.try_init() {
            Ok(()) => Some(guard),
            Err(err) => {
                eprintln!("failed to initialize tracing subscriber: {err}");
                None
            }
        }
    }
}
