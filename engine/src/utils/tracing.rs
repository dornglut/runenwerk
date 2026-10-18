use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const TRACY_ENV_VAR: &str = "ENGINE_TRACY";
const TRACY_ENV_VAR_LEGACY: &str = "ENGINE_V2_TRACY";
const DEFAULT_TRACING_FILTER: &str = "info,wgpu_core=warn,wgpu_hal=warn,naga=warn";
const DEFAULT_TRACY_FILTER: &str =
    "info,wgpu_core=warn,wgpu_hal=warn,naga=warn,runenwerk_profile=trace";

fn fallback_tracing_filter(tracy_active: bool) -> &'static str {
    if tracy_active {
        DEFAULT_TRACY_FILTER
    } else {
        DEFAULT_TRACING_FILTER
    }
}

fn tracy_flag_enabled(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn tracy_requested() -> bool {
    std::env::var(TRACY_ENV_VAR)
        .or_else(|_| std::env::var(TRACY_ENV_VAR_LEGACY))
        .map(|value| tracy_flag_enabled(&value))
        .unwrap_or(false)
}

/// Initializes the process-global tracing subscriber only when Tracy is
/// explicitly requested by the application environment.
///
/// The caller owns the returned file-appender guard for the whole process
/// lifetime. If Tracy was requested without the Cargo feature, or a subscriber
/// was already installed, this reports the failure instead of claiming a capture.
/// Applications that need ordinary tracing without Tracy use `setup_tracing`.
pub fn setup_tracing_if_requested() -> Option<WorkerGuard> {
    if !tracy_requested() {
        return None;
    }

    #[cfg(feature = "tracy")]
    {
        setup_tracing()
    }

    #[cfg(not(feature = "tracy"))]
    {
        eprintln!(
            "Tracy requested by ENGINE_TRACY, but the Engine 'tracy' Cargo feature is disabled"
        );
        None
    }
}

pub fn setup_tracing() -> Option<WorkerGuard> {
    // The explicit RUST_LOG filter remains authoritative. Without one, enabling
    // Tracy also admits the Engine's opt-in profile spans at TRACE level.
    let tracy_active = cfg!(feature = "tracy") && tracy_requested();
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(fallback_tracing_filter(tracy_active)));
    let log_dir = Path::new("logs");
    if let Err(err) = std::fs::create_dir_all(log_dir) {
        eprintln!("failed creating log directory {}: {err}", log_dir.display());
    }
    let file_appender = tracing_appender::rolling::never(log_dir, "engine.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let stdout_layer = fmt::layer().with_target(true).with_ansi(true);
    let file_layer = fmt::layer()
        .with_target(true)
        .with_ansi(false)
        .with_writer(file_writer);

    #[cfg(feature = "tracy")]
    {
        let enable_tracy = tracy_requested();
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

#[cfg(test)]
mod tests {
    use super::{fallback_tracing_filter, tracy_flag_enabled};

    #[test]
    fn tracy_default_filter_admits_profile_spans_only_when_enabled() {
        assert!(!fallback_tracing_filter(false).contains("runenwerk_profile=trace"));
        assert!(fallback_tracing_filter(true).contains("runenwerk_profile=trace"));
    }

    #[test]
    fn tracy_runtime_opt_in_accepts_only_explicit_truthy_values() {
        for value in ["1", "true", "YES", " On "] {
            assert!(tracy_flag_enabled(value), "{value:?} should enable Tracy");
        }
        for value in ["", "0", "false", "off", "disabled", "2"] {
            assert!(!tracy_flag_enabled(value), "{value:?} should disable Tracy");
        }
    }
}
