use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

// src/tracing.rs
pub fn setup_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(
        "engine=debug,scheduler=none,game=info"
    ));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_thread_names(false)
        .with_line_number(false)
        .with_span_events(FmtSpan::NONE)
        .with_file(false)
        .init();
}