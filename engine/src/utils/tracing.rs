use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter, Layer, filter::filter_fn, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

#[cfg(feature = "tracy")]
const TRACY_ENV_VAR: &str = "ENGINE_TRACY";
#[cfg(feature = "tracy")]
const TRACY_ENV_VAR_LEGACY: &str = "ENGINE_V2_TRACY";

pub struct TracingGuards {
    _engine_log_guard: WorkerGuard,
    _events_log_guard: WorkerGuard,
}

#[cfg(feature = "tracy")]
fn tracy_enabled() -> bool {
    std::env::var(TRACY_ENV_VAR)
        .or_else(|_| std::env::var(TRACY_ENV_VAR_LEGACY))
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

pub fn setup_tracing() -> Option<TracingGuards> {
    let stdout_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let file_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let log_dir = Path::new("logs");
    if let Err(err) = std::fs::create_dir_all(log_dir) {
        eprintln!("failed creating log directory {}: {err}", log_dir.display());
    }
    let engine_file_appender = tracing_appender::rolling::never(log_dir, "engine.log");
    let (engine_file_writer, engine_log_guard) =
        tracing_appender::non_blocking(engine_file_appender);
    let events_file_appender = tracing_appender::rolling::never(log_dir, "events.log");
    let (events_file_writer, events_log_guard) =
        tracing_appender::non_blocking(events_file_appender);

    let stdout_layer = fmt::layer().with_target(true).with_ansi(true);
    let stdout_layer = stdout_layer.with_filter(stdout_filter);
    let file_layer = fmt::layer()
        .with_target(true)
        .with_ansi(false)
        .with_writer(engine_file_writer)
        .with_filter(file_filter);
    let events_layer = fmt::layer()
        .with_target(true)
        .with_ansi(false)
        .with_writer(events_file_writer)
        .with_filter(filter_fn(|metadata| metadata.target() == "events"));

    #[cfg(feature = "tracy")]
    {
        let enable_tracy = tracy_enabled();
        let subscriber = tracing_subscriber::registry()
            .with(stdout_layer)
            .with(file_layer)
            .with(events_layer)
            .with(enable_tracy.then_some(tracing_tracy::TracyLayer::default()));

        match subscriber.try_init() {
            Ok(()) => {
                if enable_tracy {
                    tracing::info!(
                        env = TRACY_ENV_VAR,
                        legacy_env = TRACY_ENV_VAR_LEGACY,
                        "tracy profiling enabled (use Tracy profiler to connect)"
                    );
                } else {
                    tracing::info!(
                        env = TRACY_ENV_VAR,
                        legacy_env = TRACY_ENV_VAR_LEGACY,
                        "tracy profiling disabled (set env to 1 to enable)"
                    );
                }
                tracing::info!(
                    target: "events",
                    event = "events.trace.ready",
                    file = "logs/events.log",
                    "events trace log initialized"
                );
                Some(TracingGuards {
                    _engine_log_guard: engine_log_guard,
                    _events_log_guard: events_log_guard,
                })
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
            .with(stdout_layer)
            .with(file_layer)
            .with(events_layer);
        match subscriber.try_init() {
            Ok(()) => {
                tracing::info!(
                    target: "events",
                    event = "events.trace.ready",
                    file = "logs/events.log",
                    "events trace log initialized"
                );
                Some(TracingGuards {
                    _engine_log_guard: engine_log_guard,
                    _events_log_guard: events_log_guard,
                })
            }
            Err(err) => {
                eprintln!("failed to initialize tracing subscriber: {err}");
                None
            }
        }
    }
}
