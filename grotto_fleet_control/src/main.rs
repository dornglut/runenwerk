use anyhow::{Context, Result};
use grotto_fleet_control::{
    FleetServiceConfig, KubernetesFleetProvider, load_fleet_service_config_from_path,
    run_fleet_service,
};
use std::path::PathBuf;

const DEFAULT_CONFIG_PATH: &str = "ops/fleet/kubernetes.ron";

#[tokio::main]
async fn main() -> Result<()> {
    let launch = parse_launch_options(std::env::args().skip(1))?;
    let mut config =
        load_fleet_service_config_from_path(&launch.config_path).with_context(|| {
            format!(
                "failed loading fleet config {}",
                launch.config_path.display()
            )
        })?;
    apply_axiom_bridge_overrides(&mut config, &launch)?;

    if !config.axiom.enabled {
        println!(
            "fleet service is disabled (`axiom.enabled = false`) in {}",
            launch.config_path.display()
        );
        return Ok(());
    }

    let provider = KubernetesFleetProvider::from_default(config.kubernetes.clone())
        .await
        .context("failed to initialize kubernetes fleet provider")?;
    run_fleet_service(&provider, config).await
}

struct FleetLaunchOptions {
    config_path: PathBuf,
    axiom_enabled_override: Option<bool>,
    axiom_ws_url_override: Option<String>,
    axiom_command_token_override: Option<String>,
    axiom_service_id_override: Option<String>,
    axiom_heartbeat_seconds_override: Option<u64>,
    axiom_reconnect_backoff_ms_override: Option<u64>,
    axiom_max_buffered_events_override: Option<usize>,
}

fn parse_launch_options<I>(args: I) -> Result<FleetLaunchOptions>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut config_path = PathBuf::from(DEFAULT_CONFIG_PATH);
    let mut axiom_enabled_override = None;
    let mut axiom_ws_url_override = None;
    let mut axiom_command_token_override = None;
    let mut axiom_service_id_override = None;
    let mut axiom_heartbeat_seconds_override = None;
    let mut axiom_reconnect_backoff_ms_override = None;
    let mut axiom_max_buffered_events_override = None;

    while let Some(arg) = args.next() {
        if arg == "--config" {
            let value = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("--config requires a path"))?;
            config_path = PathBuf::from(value);
            continue;
        }
        if arg == "--axiom-enabled" {
            let value = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("--axiom-enabled requires a bool"))?;
            axiom_enabled_override = Some(parse_bool_flag("--axiom-enabled", value.as_str())?);
            continue;
        }
        if arg == "--axiom-ws-url" {
            let value = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("--axiom-ws-url requires a URL"))?;
            axiom_ws_url_override = Some(value);
            continue;
        }
        if arg == "--axiom-command-token" {
            let value = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("--axiom-command-token requires a token"))?;
            axiom_command_token_override = Some(value);
            continue;
        }
        if arg == "--axiom-service-id" {
            let value = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("--axiom-service-id requires a value"))?;
            axiom_service_id_override = Some(value);
            continue;
        }
        if arg == "--axiom-heartbeat-seconds" {
            let value = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("--axiom-heartbeat-seconds requires a value"))?;
            axiom_heartbeat_seconds_override =
                Some(parse_u64_flag("--axiom-heartbeat-seconds", value.as_str())?);
            continue;
        }
        if arg == "--axiom-reconnect-backoff-ms" {
            let value = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("--axiom-reconnect-backoff-ms requires a value"))?;
            axiom_reconnect_backoff_ms_override = Some(parse_u64_flag(
                "--axiom-reconnect-backoff-ms",
                value.as_str(),
            )?);
            continue;
        }
        if arg == "--axiom-max-buffered-events" {
            let value = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("--axiom-max-buffered-events requires a value"))?;
            axiom_max_buffered_events_override = Some(parse_usize_flag(
                "--axiom-max-buffered-events",
                value.as_str(),
            )?);
            continue;
        }
        if arg == "-h" || arg == "--help" {
            print_help();
            std::process::exit(0);
        }
        anyhow::bail!("unknown argument '{arg}'");
    }

    Ok(FleetLaunchOptions {
        config_path,
        axiom_enabled_override,
        axiom_ws_url_override,
        axiom_command_token_override,
        axiom_service_id_override,
        axiom_heartbeat_seconds_override,
        axiom_reconnect_backoff_ms_override,
        axiom_max_buffered_events_override,
    })
}

fn print_help() {
    println!("Usage: cargo run -p grotto_fleet_control -- --config <path>");
    println!();
    println!(
        "Defaults to {} when --config is not provided.",
        DEFAULT_CONFIG_PATH
    );
    println!();
    println!("Optional overrides:");
    println!("  --axiom-enabled <bool>");
    println!("  --axiom-ws-url <url>");
    println!("  --axiom-command-token <token>");
    println!("  --axiom-service-id <id>");
    println!("  --axiom-heartbeat-seconds <u64>");
    println!("  --axiom-reconnect-backoff-ms <u64>");
    println!("  --axiom-max-buffered-events <usize>");
}

fn apply_axiom_bridge_overrides(
    config: &mut FleetServiceConfig,
    launch: &FleetLaunchOptions,
) -> Result<()> {
    if let Some(value) = launch.axiom_enabled_override {
        config.axiom.enabled = value;
    }
    if let Some(value) = &launch.axiom_ws_url_override {
        config.axiom.ws_url = value.clone();
    }
    if let Some(value) = &launch.axiom_command_token_override {
        config.axiom.command_token = value.clone();
    }
    if let Some(value) = &launch.axiom_service_id_override {
        config.axiom.service_id = value.clone();
    }
    if let Some(value) = launch.axiom_heartbeat_seconds_override {
        config.axiom.heartbeat_seconds = value.max(1);
    }
    if let Some(value) = launch.axiom_reconnect_backoff_ms_override {
        config.axiom.reconnect_backoff_ms = value.max(100);
    }
    if let Some(value) = launch.axiom_max_buffered_events_override {
        config.axiom.max_buffered_events = value.max(1);
    }

    if let Some(raw) = read_env("GROTTO_FLEET_AXIOM_ENABLED") {
        config.axiom.enabled = parse_bool_flag("GROTTO_FLEET_AXIOM_ENABLED", &raw)?;
    }
    if let Some(raw) = read_env("GROTTO_FLEET_AXIOM_WS_URL") {
        config.axiom.ws_url = raw;
    }
    if let Some(raw) = read_env("GROTTO_FLEET_AXIOM_COMMAND_TOKEN") {
        config.axiom.command_token = raw;
    }
    if let Some(raw) = read_env("GROTTO_FLEET_AXIOM_SERVICE_ID") {
        config.axiom.service_id = raw;
    }
    if let Some(raw) = read_env("GROTTO_FLEET_AXIOM_HEARTBEAT_SECONDS") {
        config.axiom.heartbeat_seconds =
            parse_u64_flag("GROTTO_FLEET_AXIOM_HEARTBEAT_SECONDS", &raw)?.max(1);
    }
    if let Some(raw) = read_env("GROTTO_FLEET_AXIOM_RECONNECT_BACKOFF_MS") {
        config.axiom.reconnect_backoff_ms =
            parse_u64_flag("GROTTO_FLEET_AXIOM_RECONNECT_BACKOFF_MS", &raw)?.max(100);
    }
    if let Some(raw) = read_env("GROTTO_FLEET_AXIOM_MAX_BUFFERED_EVENTS") {
        config.axiom.max_buffered_events =
            parse_usize_flag("GROTTO_FLEET_AXIOM_MAX_BUFFERED_EVENTS", &raw)?.max(1);
    }

    Ok(())
}

fn read_env(key: &str) -> Option<String> {
    let value = std::env::var(key).ok()?;
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn parse_bool_flag(name: &str, raw: &str) -> Result<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => anyhow::bail!("{name} expects one of: 1|0|true|false|yes|no|on|off"),
    }
}

fn parse_u64_flag(name: &str, raw: &str) -> Result<u64> {
    raw.trim()
        .parse::<u64>()
        .map_err(|error| anyhow::anyhow!("{name} expects an unsigned integer: {error}"))
}

fn parse_usize_flag(name: &str, raw: &str) -> Result<usize> {
    raw.trim()
        .parse::<usize>()
        .map_err(|error| anyhow::anyhow!("{name} expects an unsigned integer: {error}"))
}
