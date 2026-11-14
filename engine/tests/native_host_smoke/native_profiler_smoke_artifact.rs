//! CI-only formatter for the real native RunenUI profiler smoke.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::Path;

const ARTIFACT_DIR_ENV: &str = "RUNENWERK_PROFILE_SMOKE_ARTIFACT_DIR";
const REVISION_ENV: &str = "RUNENWERK_PROFILE_SMOKE_REVISION";

pub(super) fn retain(report: &Value, raw: &[u8]) -> Result<()> {
    let Some(directory) = std::env::var_os(ARTIFACT_DIR_ENV) else {
        return Ok(());
    };
    let directory = std::path::PathBuf::from(directory);
    ensure!(
        directory.is_dir(),
        "native profiler artifact directory must exist"
    );
    ensure!(
        raw.len() < 8 * 1024 * 1024,
        "native profiler smoke capture exceeded 8 MiB"
    );
    let revision =
        std::env::var(REVISION_ENV).context("missing exact profiler smoke source revision")?;
    ensure!(
        revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "invalid profiler smoke source revision"
    );
    let cpu_events = report["cpu_events"]
        .as_array()
        .context("native profiler capture is missing CPU events")?;
    let stages = report["stage_statistics"]
        .as_array()
        .context("native profiler capture is missing CPU stage statistics")?;
    ensure!(
        !stages.is_empty(),
        "native profiler capture lacks stage statistics"
    );
    let bounds = stages[0]["duration_histogram_ns"]["upper_bounds_exclusive_ns"]
        .as_array()
        .context("native profiler histogram bounds missing")?;
    ensure!(
        bounds.len() == 10,
        "native profiler histogram must have ten bounds"
    );
    for stage in stages {
        ensure!(
            stage["duration_histogram_ns"]["upper_bounds_exclusive_ns"].as_array() == Some(bounds),
            "native profiler histogram bounds are inconsistent"
        );
        let sample_count = stage["sample_count"]
            .as_u64()
            .context("stage sample count missing")?;
        let counts = stage["duration_histogram_ns"]["counts"]
            .as_array()
            .context("stage duration histogram missing")?;
        let histogram_total = counts.iter().try_fold(0_u64, |total, count| {
            count.as_u64().and_then(|value| total.checked_add(value))
        });
        ensure!(
            counts.len() == 11 && histogram_total == Some(sample_count),
            "native profiler stage histogram is inconsistent"
        );
    }

    let render_frames = report["render"]["observations"]
        .as_array()
        .context("native profiler capture is missing Render observations")?;
    let summary = json!({
        "schema_version": 1,
        "kind": "runenwerk_native_profiler_smoke_summary",
        "classification": "functional_smoke_not_performance_qualification",
        "sample_count_cpu": cpu_events.len(),
        "dropped_cpu_events": report["dropped_cpu_events"],
        "render_observation_count": render_frames.len(),
        "render_status": report["render"]["status"],
        "stage_statistics": stages,
        "histogram_semantics": "counts correspond to successive upper-exclusive nanosecond bounds; final bin is unbounded",
        "limitations": [
            "short_smoke_sample_not_statistically_qualified",
            "no_native_user_typing_or_focus_stimulation",
            "host_to_render_join_unavailable",
            "gpu_clock_not_cpu_synchronized",
            "no_on_off_observer_overhead_evidence"
        ]
    });
    let manifest = json!({
        "schema_version": 1,
        "kind": "runenwerk_native_profiler_smoke",
        "source_revision": revision,
        "captured_os": report["os"],
        "build_profile": report["profile"],
        "fixture": "native_managed_typed_ui_with_render",
        "gpu_backend_requested": std::env::var("WGPU_BACKEND").unwrap_or_else(|_| "unspecified".to_owned()),
        "virtual_display": "xvfb_1280x720x24",
        "driver_provenance": "vulkaninfo_proof_in_native_job_log",
        "ci_run_id": std::env::var("GITHUB_RUN_ID").unwrap_or_else(|_| "unavailable".to_owned()),
        "ci_run_attempt": std::env::var("GITHUB_RUN_ATTEMPT").unwrap_or_else(|_| "unavailable".to_owned()),
        "window_snapshot": report["native_windows"],
        "host_pacing_close_snapshot": report["host_pacing"],
        "capture_clock": report["clock"],
        "gpu_clock_relation": report["gpu_clock_relation"]
    });
    let validation = json!({
        "schema_version": 1,
        "kind": "runenwerk_native_profiler_smoke_validation",
        "status": "passed",
        "privacy_reviewed_fixture": true,
        "performance_qualification": false,
        "checked": [
            "native_managed_typed_ui_and_render_completed",
            "versioned_profile_json_parsed",
            "cpu_host_update_ui_settlement_stages_present",
            "render_history_nonempty",
            "fixture_ui_label_title_and_identity_not_exported",
            "histogram_counts_equal_retained_stage_samples"
        ],
        "unproven": [
            "physical_windows_behavior",
            "actual_todo_input_latency",
            "profiler_on_off_overhead",
            "statistically_meaningful_frame_distribution"
        ]
    });
    let mut readable = format!(
        "# Native Runenwerk Profiler — CI Smoke\n\n**Functional smoke only — not a performance qualification.**\n\nSource revision: {revision}\n\nCaptured CPU observations: {} (evicted: {}). Render observations: {}.\n\nThe histogram bounds are exclusive and expressed in nanoseconds; the final count is unbounded. This short native Render smoke cannot establish performance percentiles.\n",
        cpu_events.len(),
        report["dropped_cpu_events"],
        render_frames.len()
    );
    let mut previous = 0_u64;
    let mut labels = Vec::new();
    for bound in bounds {
        let upper = bound.as_u64().context("histogram bound is not numeric")?;
        ensure!(
            upper > previous,
            "histogram bounds must be strictly increasing"
        );
        labels.push(format!("[{previous}, {upper})"));
        previous = upper;
    }
    labels.push(format!("[{previous}, unbounded)"));
    writeln!(
        &mut readable,
        "\nHistogram bins in order: {}\n\n| Domain | Stage | Window | UI slot | Samples | Median (ns) | P95 (ns) | Histogram counts, ascending bins |\n|---|---|---:|---:|---:|---:|---:|---|",
        labels.join(", ")
    )?;
    for stage in stages {
        let counts = stage["duration_histogram_ns"]["counts"]
            .as_array()
            .context("native profiler histogram counts missing")?
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            &mut readable,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            stage["domain"].as_str().unwrap_or("unknown"),
            stage["stage"].as_str().unwrap_or("unknown"),
            stage["window"],
            stage["ui_slot"],
            stage["sample_count"],
            stage["median_ns"],
            stage["p95_ns"],
            counts
        )?;
    }
    if let Some(host_frame) = stages.iter().find(|stage| {
        stage["domain"].as_str() == Some("host")
            && stage["stage"].as_str() == Some("ordinary_frame")
    }) {
        let counts = host_frame["duration_histogram_ns"]["counts"]
            .as_array()
            .context("Host-frame histogram counts unavailable")?;
        let maximum = counts.iter().filter_map(|value| value.as_u64()).max().unwrap_or(0);
        writeln!(
            &mut readable,
            "\n## Host ordinary-frame histogram (smoke only)\n\n```text"
        )?;
        for (label, value) in labels.iter().zip(counts) {
            let count = value.as_u64().context("Host-frame bin count is not numeric")?;
            let bar_width = if maximum <= 40 {
                count as usize
            } else {
                count.saturating_mul(40).div_ceil(maximum) as usize
            };
            writeln!(
                &mut readable,
                "{label:<25} | {:<40} ({count})",
                "#".repeat(bar_width)
            )?;
        }
        writeln!(&mut readable, "```")?;
    }
    readable.push_str(
        "\n**Interpretation:** the histogram describes the retained CPU spans only, grouped by owner/stage/window/slot. It cannot prove physical display FPS or CPU–GPU synchronization. Refer to the raw capture for each measured value.\n",
    );

    write_new(&directory.join("capture.json"), raw)?;
    write_new(
        &directory.join("summary.json"),
        &serde_json::to_vec_pretty(&summary)?,
    )?;
    write_new(
        &directory.join("manifest.json"),
        &serde_json::to_vec_pretty(&manifest)?,
    )?;
    write_new(
        &directory.join("validation.json"),
        &serde_json::to_vec_pretty(&validation)?,
    )?;
    write_new(&directory.join("report.md"), readable.as_bytes())?;
    println!("native_typed_ui_profiler_artifact=pass");
    Ok(())
}

fn write_new(path: &Path, contents: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .with_context(|| {
            format!(
                "cannot create new native profiler smoke artifact {}",
                path.display()
            )
        })?;
    file.write_all(contents)?;
    file.sync_all()?;
    Ok(())
}
