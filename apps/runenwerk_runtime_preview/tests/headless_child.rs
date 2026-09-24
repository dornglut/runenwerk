use anyhow::{Context, Result, anyhow};
use editor_preview::{PreviewBootstrap, decode_lower_hex};
use runenwerk_runtime_preview::{RuntimePreviewConfig, RuntimePreviewHost};
use std::io::ErrorKind;
use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::process::{Child, Command, Stdio};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn headless_child_process_emits_valid_runennet_bootstrap() -> Result<()> {
    if runtime_preview_transport_permission_denied().await? {
        eprintln!(
            "skipping runtime preview headless-child transport test: local socket bind is denied"
        );
        return Ok(());
    }

    let executable = env!("CARGO_BIN_EXE_runenwerk_runtime_preview");
    let mut child = Command::new(executable)
        .arg("--headless")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("runtime preview child should spawn")?;

    let inspection = inspect_bootstrap(&mut child);
    let cleanup = stop_child(&mut child);
    inspection?;
    cleanup?;
    Ok(())
}

fn inspect_bootstrap(child: &mut Child) -> Result<()> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("runtime preview child stdout was not captured"))?;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .context("runtime preview child should print bootstrap line")?;
    let bootstrap = PreviewBootstrap::parse_stdout_line(&line)
        .context("runtime preview child bootstrap should parse")?;

    bootstrap
        .endpoint
        .parse::<SocketAddr>()
        .context("runtime preview bootstrap endpoint should be a socket address")?;
    if bootstrap.server_name.is_empty() {
        return Err(anyhow!("runtime preview bootstrap server name was empty"));
    }
    if decode_lower_hex(&bootstrap.trusted_certificate_der_hex)?.is_empty() {
        return Err(anyhow!(
            "runtime preview bootstrap trust certificate was empty"
        ));
    }
    Ok(())
}

async fn runtime_preview_transport_permission_denied() -> Result<bool> {
    match RuntimePreviewHost::spawn(RuntimePreviewConfig::headless()) {
        Ok(host) => {
            let _ = host.shutdown().await;
            Ok(false)
        }
        Err(error) if is_permission_denied(&error) => Ok(true),
        Err(error) => Err(error).context("runtime preview transport preflight should spawn"),
    }
}

fn is_permission_denied(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io_error| io_error.kind() == ErrorKind::PermissionDenied)
    })
}

fn stop_child(child: &mut Child) -> Result<()> {
    if child.try_wait()?.is_none() {
        child.kill()?;
    }
    child.wait()?;
    Ok(())
}
