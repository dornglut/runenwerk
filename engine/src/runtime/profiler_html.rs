//! Private, source-qualified offline HTML summary from the exact typed v3 capture.
//! No input payloads, external resources, browser access or synthetic joins.
use super::Capture;
use std::fmt::Write as _;

const STYLE: &str = r#"<style>
:root{color-scheme:dark;font-family:system-ui,Segoe UI,sans-serif;background:#0d121b;color:#eef3fa}
*{box-sizing:border-box}body{margin:0;line-height:1.5}
main{max-width:1160px;padding:32px 24px 80px;margin:auto}
h1{font-size:2rem;margin:0 0 8px}h2{font-size:1.25rem;margin:36px 0 12px}
p,small{color:#aebed0}.intro{max-width:870px}
.cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(175px,1fr));gap:12px;margin:26px 0}
.card{background:#161f2e;border:1px solid #304158;border-radius:10px;padding:16px}
.card strong{display:block;font-size:1.45rem;font-variant-numeric:tabular-nums}
.card small{display:block}table{width:100%;border-collapse:collapse;font-variant-numeric:tabular-nums}
th,td{padding:10px 12px;text-align:left;border-bottom:1px solid #304158;vertical-align:top}
th{color:#bacce0;background:#172235;position:sticky;top:0}code{font-size:.88em;overflow-wrap:anywhere}
.scroll{overflow-x:auto}.note{border-left:3px solid #8bb8ed;padding:8px 16px;background:#162233}
.muted{color:#aebed0}.hist{min-width:200px}
.bucket{display:grid;grid-template-columns:125px 1fr 45px;align-items:center;gap:8px;margin:5px 0}
.bar-track{height:8px;border-radius:6px;background:#263449;overflow:hidden}
.bar{height:100%;background:#80b7e8;border-radius:6px}
details{min-width:220px}summary{cursor:pointer;color:#99c9fb}
ul{padding-left:22px}li{margin-bottom:5px}footer{margin-top:40px;color:#aebed0}
</style>"#;

fn escape(value: &str) -> String {
    value.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn ms(ns: u64) -> String {
    format!("{:.3}", ns as f64 / 1_000_000.0)
}

fn percentile_ms(samples: impl Iterator<Item = f32>) -> Option<(f64, f64, usize)> {
    let mut values: Vec<f64> = samples
        .filter(|v| v.is_finite() && *v >= 0.0)
        .map(f64::from)
        .collect();
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let n = values.len();
    let p95 = (n * 95).div_ceil(100).saturating_sub(1);
    Some((
        if n % 2 == 0 {
            (values[n / 2 - 1] + values[n / 2]) / 2.0
        } else {
            values[n / 2]
        },
        values[p95],
        n,
    ))
}

pub(super) fn render_report(report: &Capture) -> String {
    let mut out = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
<title>Runenwerk native profile</title>{STYLE}</head><body><main>\
<h1>Runenwerk native profile</h1>\
<p class=\"intro\">Source-owned CPU durations and separately owned Render observations. \
Overlapping Host/ECS spans must not be summed. CPU and GPU clocks are not synchronized; \
native input promotion is not physical scanout.</p>"
    );
    let status = escape(report.status);
    let _ = write!(
        out,
        "<div class=\"cards\">\
<div class=\"card\"><strong>{status}</strong><small>Application session</small></div>\
<div class=\"card\"><strong>{}</strong><small>Retained CPU observations</small></div>\
<div class=\"card\"><strong>{}</strong><small>CPU event evictions</small></div>\
<div class=\"card\"><strong>{}</strong><small>Retained Render observations</small></div>\
</div>",
        report.cpu_events.len(),
        report.dropped_cpu_events,
        report.render.observations.len(),
    );
    out.push_str("<h2>Provenance and coverage</h2><div class=\"scroll\"><table><tbody>");
    for (name, value) in [
        ("Schema", report.schema_version.to_string()),
        ("Session", report.session_id.clone()),
        ("Source revision", report.source_revision.to_owned()),
        ("Operating system", report.os.to_owned()),
        ("Build", report.profile.to_owned()),
        ("PID", report.process_id.to_string()),
        ("Render capture", report.render.status.to_owned()),
        ("Render clock-to-Host join", report.render.host_frame_join_status.to_owned()),
        ("Input promotion", report.native_ui_presentations.status.to_owned()),
    ] {
        let _ = write!(out, "<tr><th>{}</th><td><code>{}</code></td></tr>", escape(name), escape(&value));
    }
    if let Some(pacing) = &report.host_pacing {
        let _ = write!(
            out, "<tr><th>Host pacing (close snapshot)</th><td>{} — {} FPS target</td></tr>",
            escape(pacing.mode),
            pacing.target_fps.map_or_else(|| "none".to_string(), |fps| fps.to_string()),
        );
    }
    let _ = write!(
        out,
        "<tr><th>Native windows (close snapshot)</th><td>{}</td></tr></tbody></table></div>",
        report.native_windows.windows.len()
    );

    out.push_str("<h2>CPU stage durations</h2>\
<p>Each row is a source-recorded timed scope, grouped by owner, stage, window and UI slot. \
Percentiles are CPU elapsed time within that scope, not independent frame budgets.</p>\
<div class=\"scroll\"><table><thead><tr><th>Source stage</th><th>Window / slot</th>\
<th>Samples</th><th>Median (ms)</th><th>P95 (ms)</th><th>Distribution</th></tr></thead><tbody>");
    let mut stages: Vec<_> = report.stage_statistics.iter().collect();
    stages.sort_by(|a, b| b.median_ns.cmp(&a.median_ns)
        .then_with(|| a.domain.cmp(b.domain)).then_with(|| a.stage.cmp(b.stage)));
    if stages.is_empty() {
        out.push_str("<tr><td colspan=\"6\">No CPU stage observations retained.</td></tr>");
    }
    for stage in stages {
        let context = format!(
            "{} / {}",
            stage.window.map_or_else(|| "—".to_owned(), |v| v.to_string()),
            stage.ui_slot.map_or_else(|| "—".to_owned(), |v| v.to_string())
        );
        let _ = write!(
            out,
            "<tr><td><code>{}.{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class=\"hist\"><details><summary>Histogram</summary>",
            escape(stage.domain),
            escape(stage.stage),
            escape(&context),
            stage.sample_count,
            ms(stage.median_ns),
            ms(stage.p95_ns),
        );
        let histogram = &stage.duration_histogram_ns;
        let max = histogram.counts.iter().copied().max().unwrap_or(0);
        for (index, count) in histogram.counts.iter().enumerate() {
            let label = match (index, histogram.upper_bounds_exclusive_ns.get(index)) {
                (0, Some(upper)) => format!("< {} ms", ms(*upper)),
                (index, Some(upper)) => format!(
                    "{} – {} ms",
                    ms(histogram.upper_bounds_exclusive_ns[index - 1]),
                    ms(*upper)
                ),
                (_, None) => format!(
                    "≥ {} ms",
                    ms(*histogram.upper_bounds_exclusive_ns.last().unwrap())
                ),
            };
            let width = if max == 0 { 0 } else { 100 * count / max };
            let _ = write!(
                out,
                "<div class=\"bucket\"><small>{}</small><div class=\"bar-track\"><div class=\"bar\" style=\"width:{}%\"></div></div><small>{}</small></div>",
                escape(&label),
                width,
                count
            );
        }
        out.push_str("</details></td></tr>");
    }
    out.push_str("</tbody></table></div>");

    out.push_str("<h2>Render-owned frame phase observations</h2>\
<p>Separate Render clock and frame identity; each value is a CPU phase duration exported by Render. \
These phases must not be joined to a particular Host frame without source-owned correlation.</p>\
<div class=\"scroll\"><table><thead><tr><th>Render CPU phase</th><th>Samples</th>\
<th>Median (ms)</th><th>P95 (ms)</th></tr></thead><tbody>");
    for (name, phase) in [
        ("Surface acquire", 0u8),
        ("UI preparation", 1),
        ("Preflight", 2),
        ("RenderFlow encoding", 3),
        ("Encoding / submission", 4),
    ] {
        let values = report.render.observations.iter().map(|frame| match phase {
            0 => frame.acquire_cpu_ms,
            1 => frame.prepare_ui_cpu_ms,
            2 => frame.preflight_cpu_ms,
            3 => frame.flow_encode_cpu_ms,
            _ => frame.encode_submit_cpu_ms,
        });
        if let Some((median, p95, n)) = percentile_ms(values) {
            let _ = write!(
                out,
                "<tr><td>{}</td><td>{}</td><td>{:.3}</td><td>{:.3}</td></tr>",
                escape(name),
                n,
                median,
                p95
            );
        } else {
            let _ = write!(
                out,
                "<tr><td>{}</td><td>0</td><td>unavailable</td><td>unavailable</td></tr>",
                escape(name)
            );
        }
    }
    out.push_str("</tbody></table></div>");

    let gpu_available = report
        .render
        .observations
        .iter()
        .filter(|item| item.composed_gpu_duration_ms.is_some())
        .count();
    let _ = write!(
        out,
        "<h2>Missing and lost evidence</h2><p class=\"note\">GPU-composed timing available for \
{} of {} Render observations. Missing timings are not zero. No inferred GPU/display FPS or input-to-photon latency.</p>",
        gpu_available,
        report.render.observations.len()
    );
    out.push_str("<ul>");
    for (label, n) in [
        ("CPU events evicted", report.dropped_cpu_events),
        (
            "Native UI transition events evicted",
            report.native_ui_presentations.dropped_events,
        ),
        ("Render events evicted", report.render.evicted),
        ("Render events sampled out", report.render.sampled_out),
        (
            "Render observations omitted at export",
            report.render.omitted_by_export_limit as u64,
        ),
        ("Render evidence uncorrelated", report.render.uncorrelated),
    ] {
        let _ = write!(out, "<li>{}: {}</li>", escape(label), n);
    }
    for name in report.unavailable {
        let _ = write!(out, "<li>Unavailable: <code>{}</code></li>", escape(name));
    }
    out.push_str(
        "</ul><footer>First-party Runenwerk profiler · Static offline report · \
This is observed diagnostic evidence, not a performance regression verdict. \
See capture.json for bounded raw events, identity and receipt details.</footer>\
</main></body></html>",
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::profiler::{
        DurationHistogram, NativeUiPresentationCapture, NativeWindowCapture, RenderCapture,
        StageStatistics,
    };

    fn fixture() -> Capture {
        Capture {
            schema_version: 3,
            session_id: "<unsafe>&sample".into(),
            kind: "runenwerk_native_application_profile",
            status: "completed",
            clock: "cpu_monotonic_process_local_nanoseconds",
            gpu_clock_relation: "not_synchronized_to_cpu",
            process_id: 1,
            os: "test",
            profile: "optimized",
            source_revision: "<script>not a revision</script>",
            host_pacing: None,
            native_windows: NativeWindowCapture {
                status: "available",
                observed_at: "capture_close_not_display_timestamp",
                windows: vec![],
            },
            event_capacity: 16_384,
            dropped_cpu_events: 2,
            cpu_events: vec![],
            stage_statistics: vec![StageStatistics {
                domain: "host",
                stage: "ordinary_frame",
                window: None,
                ui_slot: None,
                sample_count: 1,
                median_ns: 7_000_000,
                p95_ns: 7_000_000,
                duration_histogram_ns: DurationHistogram {
                    upper_bounds_exclusive_ns: super::super::DURATION_HISTOGRAM_UPPER_NS,
                    counts: [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0],
                },
            }],
            native_ui_presentations: NativeUiPresentationCapture {
                status: "source_observed_not_physical_scanout",
                event_capacity: 1024,
                dropped_events: 0,
                events: vec![],
            },
            render: RenderCapture {
                status: "no_observations",
                observations: vec![],
                omitted_by_export_limit: 0,
                evicted: 0,
                sampled_out: 0,
                dropped_correlated: 0,
                uncorrelated: 0,
                host_frame_join_status: "unavailable_no_owner_correlation",
            },
            unavailable: &["process_cpu_and_memory"],
        }
    }

    #[test]
    fn report_is_standalone_and_escapes_untrusted_metadata() {
        let html = render_report(&fixture());
        assert!(html.starts_with("<!doctype html>"));
        assert!(!html.contains("<script>not a revision</script>"));
        assert!(html.contains("&lt;script&gt;not a revision&lt;/script&gt;"));
        assert!(html.contains("&lt;unsafe&gt;&amp;sample"));
        assert!(html.contains("7.000"));
        assert!(html.contains("Histogram"));
        assert!(html.contains("unavailable"));
        assert!(!html.contains("<script src="));
        assert!(!html.contains("https://"));
    }

    #[test]
    fn missing_stage_histories_stay_explicit() {
        let mut capture = fixture();
        capture.stage_statistics.clear();
        let html = render_report(&capture);
        assert!(html.contains("No CPU stage observations retained."));
        assert!(html.contains("GPU-composed timing available for 0 of 0"));
    }
}
