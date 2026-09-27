use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Take};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use engine::automation::{
    AppAutomationInputReplayExt, AutomationInputReplayOutcome, AutomationInputReplaySourceMap,
    AutomationInputReplayStateAssumption, AutomationInputTraceRecordingWitness,
    AutomationOwnerAdapter, InputSourceId, MAX_ARTIFACT_BYTES, import_automation_input_trace_v1,
};
use runenwerk_render_lab::automation::{
    RenderLabAutomationAdapter, RenderLabAutomationQuery, RenderLabAutomationTarget,
    RenderLabCameraObservation, build_headless_automation_app,
};

fn main() -> anyhow::Result<()> {
    match parse_command(env::args_os().skip(1))? {
        Command::Native => runenwerk_render_lab::run_native(),
        Command::ReplayTrace(path) => run_replay_trace(&path),
        Command::NativeMeasurement {
            output_path,
            submitted_frame_limit,
            window_size_px,
            radiance_size_px,
        } => runenwerk_render_lab::run_native_measurement(
            output_path,
            submitted_frame_limit,
            window_size_px,
            radiance_size_px,
        ),
        Command::TemporalQuality {
            output_root,
            submitted_frame_limit,
            window_size_px,
            internal_size_px,
        } => runenwerk_render_lab::run_native_temporal_quality(
            output_root,
            submitted_frame_limit,
            window_size_px,
            internal_size_px,
        ),
        Command::FoundingDirect(output_root) => {
            runenwerk_render_lab::run_founding_direct(output_root)?;
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    FoundingDirect(PathBuf),
    Native,
    ReplayTrace(PathBuf),
    NativeMeasurement {
        output_path: PathBuf,
        submitted_frame_limit: Option<usize>,
        window_size_px: Option<(u32, u32)>,
        radiance_size_px: Option<(u32, u32)>,
    },
    TemporalQuality {
        output_root: PathBuf,
        submitted_frame_limit: Option<usize>,
        window_size_px: (u32, u32),
        internal_size_px: (u32, u32),
    },
}

fn parse_command(args: impl IntoIterator<Item = OsString>) -> anyhow::Result<Command> {
    let mut args = args.into_iter();
    let first = args.next();
    if matches!(first.as_deref(), Some(value) if value == "--rl2" || value == "--native") {
        return Ok(Command::Native);
    }
    if matches!(first.as_deref(), Some(value) if value == "--replay-trace") {
        let Some(path) = args.next() else {
            bail!("--replay-trace requires a persisted trace path");
        };
        if let Some(extra) = args.next() {
            bail!(
                "unexpected --replay-trace argument '{}'",
                extra.to_string_lossy()
            );
        }
        return Ok(Command::ReplayTrace(PathBuf::from(path)));
    }
    if matches!(first.as_deref(), Some(value) if value == "--rl2-quality") {
        let default_output = PathBuf::from("render-lab/rl2-temporal-quality");
        let mut args = args.peekable();
        let output_root = match args.peek() {
            Some(value) if !value.to_string_lossy().starts_with("--") => {
                PathBuf::from(args.next().expect("peeked temporal quality output root"))
            }
            _ => default_output,
        };
        let mut submitted_frame_limit = None;
        let mut window_size_px = None;
        let mut internal_size_px = None;
        while let Some(flag) = args.next() {
            if flag == "--submitted-frames" {
                if submitted_frame_limit.is_some() {
                    bail!("duplicate --submitted-frames argument");
                }
                submitted_frame_limit = Some(parse_frame_limit(args.next())?);
            } else if flag == "--window-size-px" {
                if window_size_px.is_some() {
                    bail!("duplicate --window-size-px argument");
                }
                window_size_px = Some(parse_window_size_px(args.next())?);
            } else if flag == "--internal-size-px" {
                if internal_size_px.is_some() {
                    bail!("duplicate --internal-size-px argument");
                }
                internal_size_px = Some(parse_internal_size_px(args.next())?);
            } else {
                bail!(
                    "unexpected RL2 temporal quality argument '{}'",
                    flag.to_string_lossy()
                );
            }
        }
        let window_size_px = window_size_px.ok_or_else(|| {
            anyhow::anyhow!("--rl2-quality requires --window-size-px WIDTHxHEIGHT")
        })?;
        let internal_size_px = internal_size_px.ok_or_else(|| {
            anyhow::anyhow!("--rl2-quality requires --internal-size-px WIDTHxHEIGHT")
        })?;
        return Ok(Command::TemporalQuality {
            output_root,
            submitted_frame_limit,
            window_size_px,
            internal_size_px,
        });
    }
    if matches!(first.as_deref(), Some(value) if value == "--rl2-measure") {
        let default_output = PathBuf::from("render-lab/rl2-measurement.json");
        let mut args = args.peekable();
        let output_path = match args.peek() {
            Some(value) if !value.to_string_lossy().starts_with("--") => {
                PathBuf::from(args.next().expect("peeked measurement output path"))
            }
            _ => default_output,
        };
        let mut submitted_frame_limit = None;
        let mut window_size_px = None;
        let mut radiance_size_px = None;
        while let Some(flag) = args.next() {
            if flag == "--submitted-frames" {
                if submitted_frame_limit.is_some() {
                    anyhow::bail!("duplicate --submitted-frames argument");
                }
                submitted_frame_limit = Some(parse_frame_limit(args.next())?);
            } else if flag == "--window-size-px" {
                if window_size_px.is_some() {
                    anyhow::bail!("duplicate --window-size-px argument");
                }
                window_size_px = Some(parse_window_size_px(args.next())?);
            } else if flag == "--radiance-size-px" {
                if radiance_size_px.is_some() {
                    anyhow::bail!("duplicate --radiance-size-px argument");
                }
                radiance_size_px = Some(parse_radiance_size_px(args.next())?);
            } else {
                anyhow::bail!(
                    "unexpected RL2 measurement argument '{}'",
                    flag.to_string_lossy()
                );
            }
        }
        return Ok(Command::NativeMeasurement {
            output_path,
            submitted_frame_limit,
            window_size_px,
            radiance_size_px,
        });
    }
    Ok(Command::FoundingDirect(
        first
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("render-lab")),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ReplayTraceSummary {
    completed_frames: u64,
    camera: RenderLabCameraObservation,
}

fn run_replay_trace(path: &Path) -> anyhow::Result<()> {
    let bytes = read_trace_file(path)?;
    let summary = replay_trace_bytes(&bytes)?;
    println!(
        "replay completed: frames={} camera_yaw_radians={} camera_pitch_radians={} camera_distance={} camera_pan_x={} camera_pan_y={}",
        summary.completed_frames,
        summary.camera.yaw_radians,
        summary.camera.pitch_radians,
        summary.camera.distance,
        summary.camera.pan[0],
        summary.camera.pan[1],
    );
    Ok(())
}

fn read_trace_file(path: &Path) -> anyhow::Result<Vec<u8>> {
    let file = File::open(path)
        .with_context(|| format!("open persisted automation trace {}", path.display()))?;
    read_bounded_trace(file)
        .with_context(|| format!("read persisted automation trace {}", path.display()))
}

fn read_bounded_trace(reader: impl Read) -> anyhow::Result<Vec<u8>> {
    let limit = u64::try_from(MAX_ARTIFACT_BYTES)
        .expect("persisted trace byte limit fits u64")
        .checked_add(1)
        .expect("persisted trace byte limit leaves room for sentinel byte");
    let mut reader: Take<_> = reader.take(limit);
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .context("read persisted automation trace bytes")?;
    if bytes.len() > MAX_ARTIFACT_BYTES {
        bail!(
            "persisted automation trace exceeds the {} byte limit",
            MAX_ARTIFACT_BYTES
        );
    }
    Ok(bytes)
}

fn replay_trace_bytes(bytes: &[u8]) -> anyhow::Result<ReplayTraceSummary> {
    let imported = import_automation_input_trace_v1(bytes)
        .context("import persisted normalized replay trace V1")?;
    if imported.recording_witness()
        != AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart
    {
        bail!("persisted automation trace has an unsupported recording-state witness");
    }

    let recorded_sources = recorded_sources(imported.trace());
    let source_map = fresh_replay_source_map(&recorded_sources)?;

    let mut app = build_headless_automation_app();
    let report = app.replay_automation_input_trace(
        imported.trace(),
        &source_map,
        AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    );
    if report.outcome() != AutomationInputReplayOutcome::Completed {
        bail!(
            "normalized replay failed: outcome={:?}, completed_frames={}, failing_frame={:?}, failing_group={:?}, detail={}",
            report.outcome(),
            report.completed_frames(),
            report.failing_frame_ordinal(),
            report.failing_group_index(),
            report.detail().unwrap_or("none"),
        );
    }

    let camera_result = {
        let mut adapter = RenderLabAutomationAdapter::new(&mut app);
        adapter
            .query(&RenderLabAutomationTarget, RenderLabAutomationQuery::Camera)
            .map_err(anyhow::Error::msg)
            .context("query Render Lab camera after replay")
    };
    let teardown_result = app
        .teardown_automation_input_replay()
        .context("tear down replay-owned normalized input state");

    let camera = camera_result?;
    teardown_result?;

    Ok(ReplayTraceSummary {
        completed_frames: report.completed_frames(),
        camera,
    })
}

fn recorded_sources(trace: &engine::automation::AutomationInputTrace) -> Vec<InputSourceId> {
    let mut sources = Vec::new();
    for frame in trace.frames() {
        for group in frame.groups() {
            if !sources.contains(&group.context.source) {
                sources.push(group.context.source);
            }
        }
    }
    sources
}

fn fresh_replay_source_map(
    recorded_sources: &[InputSourceId],
) -> anyhow::Result<AutomationInputReplaySourceMap> {
    let mut entries = Vec::with_capacity(recorded_sources.len());
    let mut next_raw = u64::MAX;

    for recorded in recorded_sources {
        let replay_source = loop {
            let candidate = InputSourceId::new(next_raw);
            next_raw = next_raw
                .checked_sub(1)
                .ok_or_else(|| anyhow::anyhow!("exhausted replay-owned input source identity"))?;
            if !recorded_sources.contains(&candidate)
                && !entries
                    .iter()
                    .any(|(_, replay): &(InputSourceId, InputSourceId)| *replay == candidate)
            {
                break candidate;
            }
        };
        entries.push((*recorded, replay_source));
    }

    Ok(AutomationInputReplaySourceMap::new(entries))
}

fn parse_frame_limit(value: Option<OsString>) -> anyhow::Result<usize> {
    let Some(value) = value else {
        anyhow::bail!("--submitted-frames requires a positive submitted-frame count");
    };
    let Some(value) = value.to_str() else {
        anyhow::bail!("--submitted-frames requires a UTF-8 integer");
    };
    let limit = value
        .parse::<usize>()
        .map_err(|_| anyhow::anyhow!("invalid --submitted-frames value '{value}'"))?;
    if limit == 0 {
        anyhow::bail!("--submitted-frames requires a positive submitted-frame count");
    }
    Ok(limit)
}

fn parse_window_size_px(value: Option<OsString>) -> anyhow::Result<(u32, u32)> {
    let Some(value) = value else {
        anyhow::bail!("--window-size-px requires WIDTHxHEIGHT");
    };
    let Some(value) = value.to_str() else {
        anyhow::bail!("--window-size-px requires a UTF-8 WIDTHxHEIGHT value");
    };
    let Some((width, height)) = value.split_once('x') else {
        anyhow::bail!("invalid --window-size-px value '{value}'; expected WIDTHxHEIGHT");
    };
    let width = width
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("invalid --window-size-px width in '{value}'"))?;
    let height = height
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("invalid --window-size-px height in '{value}'"))?;
    if width == 0 || height == 0 {
        anyhow::bail!("--window-size-px requires positive WIDTHxHEIGHT");
    }
    Ok((width, height))
}

fn parse_internal_size_px(value: Option<OsString>) -> anyhow::Result<(u32, u32)> {
    let Some(value) = value else {
        bail!("--internal-size-px requires WIDTHxHEIGHT");
    };
    let value = value.to_string_lossy();
    let Some((width, height)) = value.split_once('x') else {
        bail!("invalid --internal-size-px value '{value}'; expected WIDTHxHEIGHT");
    };
    let width = width
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("invalid --internal-size-px width in '{value}'"))?;
    let height = height
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("invalid --internal-size-px height in '{value}'"))?;
    if width == 0 || height == 0 {
        bail!("--internal-size-px requires positive WIDTHxHEIGHT");
    }
    Ok((width, height))
}

fn parse_radiance_size_px(value: Option<OsString>) -> anyhow::Result<(u32, u32)> {
    let Some(value) = value else {
        anyhow::bail!("--radiance-size-px requires WIDTHxHEIGHT");
    };
    let Some(value) = value.to_str() else {
        anyhow::bail!("--radiance-size-px requires a UTF-8 WIDTHxHEIGHT value");
    };
    let Some((width, height)) = value.split_once('x') else {
        anyhow::bail!("invalid --radiance-size-px value '{value}'; expected WIDTHxHEIGHT");
    };
    let width = width
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("invalid --radiance-size-px width in '{value}'"))?;
    let height = height
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("invalid --radiance-size-px height in '{value}'"))?;
    if width == 0 || height == 0 {
        anyhow::bail!("--radiance-size-px requires positive WIDTHxHEIGHT");
    }
    Ok((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn zero_arguments_keep_the_default_output_root() {
        assert_eq!(
            parse_command(args(&[])).unwrap(),
            Command::FoundingDirect("render-lab".into())
        );
    }

    #[test]
    fn one_positional_argument_is_the_output_root() {
        assert_eq!(
            parse_command(args(&["artifacts"])).unwrap(),
            Command::FoundingDirect("artifacts".into())
        );
    }

    #[test]
    fn multiple_positional_arguments_preserve_the_first_root() {
        assert_eq!(
            parse_command(args(&["first", "second"])).unwrap(),
            Command::FoundingDirect("first".into())
        );
    }

    #[test]
    fn native_mode_is_explicit() {
        assert_eq!(parse_command(args(&["--rl2"])).unwrap(), Command::Native);
        assert_eq!(parse_command(args(&["--native"])).unwrap(), Command::Native);
    }

    #[test]
    fn persisted_trace_replay_mode_requires_exactly_one_path() {
        assert_eq!(
            parse_command(args(&["--replay-trace", "evidence/input.ron"])).unwrap(),
            Command::ReplayTrace(PathBuf::from("evidence/input.ron"))
        );
        assert!(parse_command(args(&["--replay-trace"])).is_err());
        assert!(parse_command(args(&["--replay-trace", "first.ron", "second.ron"])).is_err());
    }

    #[test]
    fn persisted_trace_reader_rejects_oversize_input_before_unbounded_growth() {
        let reader = std::io::repeat(0).take(
            u64::try_from(MAX_ARTIFACT_BYTES)
                .unwrap()
                .checked_add(1)
                .unwrap(),
        );
        assert!(read_bounded_trace(reader).is_err());
    }

    #[test]
    fn persisted_trace_import_errors_remain_typed_causes() {
        let error = replay_trace_bytes(b"this is not RON").unwrap_err();
        assert!(
            error
                .downcast_ref::<engine::automation::AutomationInputTraceImportError>()
                .is_some()
        );
    }

    #[test]
    fn temporal_quality_mode_requires_explicit_extents_and_accepts_matrix_cases() {
        assert_eq!(
            parse_command(args(&[
                "--rl2-quality",
                "evidence/p067",
                "--window-size-px",
                "1920x1080",
                "--internal-size-px",
                "1280x720",
                "--submitted-frames",
                "600"
            ]))
            .unwrap(),
            Command::TemporalQuality {
                output_root: PathBuf::from("evidence/p067"),
                submitted_frame_limit: Some(600),
                window_size_px: (1920, 1080),
                internal_size_px: (1280, 720),
            }
        );
        assert_eq!(
            parse_command(args(&[
                "--rl2-quality",
                "--window-size-px",
                "1920x1080",
                "--internal-size-px",
                "1920x1080"
            ]))
            .unwrap(),
            Command::TemporalQuality {
                output_root: PathBuf::from("render-lab/rl2-temporal-quality"),
                submitted_frame_limit: None,
                window_size_px: (1920, 1080),
                internal_size_px: (1920, 1080),
            }
        );
    }

    #[test]
    fn temporal_quality_mode_preserves_invalid_fixed_policy_shapes_for_runtime_fallback() {
        assert_eq!(
            parse_command(args(&[
                "--rl2-quality",
                "--window-size-px",
                "1920x1080",
                "--internal-size-px",
                "1280x800"
            ]))
            .unwrap(),
            Command::TemporalQuality {
                output_root: PathBuf::from("render-lab/rl2-temporal-quality"),
                submitted_frame_limit: None,
                window_size_px: (1920, 1080),
                internal_size_px: (1280, 800),
            }
        );
    }

    #[test]
    fn temporal_quality_mode_rejects_missing_duplicate_or_malformed_arguments() {
        for values in [
            vec!["--rl2-quality"],
            vec!["--rl2-quality", "--window-size-px", "1920x1080"],
            vec!["--rl2-quality", "--internal-size-px", "1280x720"],
            vec![
                "--rl2-quality",
                "--window-size-px",
                "1920x1080",
                "--window-size-px",
                "1280x720",
                "--internal-size-px",
                "1280x720",
            ],
            vec![
                "--rl2-quality",
                "--window-size-px",
                "1920x1080",
                "--internal-size-px",
                "1280X720",
            ],
        ] {
            assert!(parse_command(args(&values)).is_err(), "{values:?}");
        }
    }

    #[test]
    fn native_measurement_mode_has_explicit_and_default_output_paths() {
        assert_eq!(
            parse_command(args(&["--rl2-measure", "evidence/run.json"])).unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("evidence/run.json"),
                submitted_frame_limit: None,
                window_size_px: None,
                radiance_size_px: None,
            }
        );
        assert_eq!(
            parse_command(args(&["--rl2-measure"])).unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("render-lab/rl2-measurement.json"),
                submitted_frame_limit: None,
                window_size_px: None,
                radiance_size_px: None,
            }
        );
    }

    #[test]
    fn native_measurement_mode_accepts_a_positive_bounded_frame_target() {
        assert_eq!(
            parse_command(args(&[
                "--rl2-measure",
                "evidence/run.json",
                "--submitted-frames",
                "600"
            ]))
            .unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("evidence/run.json"),
                submitted_frame_limit: Some(600),
                window_size_px: None,
                radiance_size_px: None,
            }
        );
        assert_eq!(
            parse_command(args(&["--rl2-measure", "--submitted-frames", "420"])).unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("render-lab/rl2-measurement.json"),
                submitted_frame_limit: Some(420),
                window_size_px: None,
                radiance_size_px: None,
            }
        );
    }

    #[test]
    fn native_measurement_mode_accepts_explicit_physical_window_size() {
        assert_eq!(
            parse_command(args(&[
                "--rl2-measure",
                "evidence/run.json",
                "--window-size-px",
                "1600x1200",
                "--submitted-frames",
                "600"
            ]))
            .unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("evidence/run.json"),
                submitted_frame_limit: Some(600),
                window_size_px: Some((1600, 1200)),
                radiance_size_px: None,
            }
        );
        assert_eq!(
            parse_command(args(&[
                "--rl2-measure",
                "--submitted-frames",
                "420",
                "--window-size-px",
                "800x600"
            ]))
            .unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("render-lab/rl2-measurement.json"),
                submitted_frame_limit: Some(420),
                window_size_px: Some((800, 600)),
                radiance_size_px: None,
            }
        );
    }

    #[test]
    fn native_measurement_mode_accepts_explicit_radiance_size() {
        assert_eq!(
            parse_command(args(&[
                "--rl2-measure",
                "evidence/run.json",
                "--window-size-px",
                "1920x1080",
                "--radiance-size-px",
                "1280x720",
                "--submitted-frames",
                "600"
            ]))
            .unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("evidence/run.json"),
                submitted_frame_limit: Some(600),
                window_size_px: Some((1920, 1080)),
                radiance_size_px: Some((1280, 720)),
            }
        );
    }

    #[test]
    fn native_measurement_mode_rejects_invalid_radiance_sizes() {
        for values in [
            vec!["--rl2-measure", "--radiance-size-px"],
            vec!["--rl2-measure", "--radiance-size-px", "0x720"],
            vec!["--rl2-measure", "--radiance-size-px", "1280x0"],
            vec!["--rl2-measure", "--radiance-size-px", "1280"],
            vec!["--rl2-measure", "--radiance-size-px", "1280X720"],
            vec!["--rl2-measure", "--radiance-size-px", "watx720"],
            vec![
                "--rl2-measure",
                "--radiance-size-px",
                "1280x720",
                "--radiance-size-px",
                "960x540",
            ],
        ] {
            assert!(parse_command(args(&values)).is_err(), "{values:?}");
        }
    }

    #[test]
    fn native_measurement_mode_rejects_invalid_physical_window_sizes() {
        for values in [
            vec!["--rl2-measure", "--window-size-px"],
            vec!["--rl2-measure", "--window-size-px", "0x1200"],
            vec!["--rl2-measure", "--window-size-px", "1600x0"],
            vec!["--rl2-measure", "--window-size-px", "1600"],
            vec!["--rl2-measure", "--window-size-px", "1600X1200"],
            vec!["--rl2-measure", "--window-size-px", "watx1200"],
            vec![
                "--rl2-measure",
                "--window-size-px",
                "800x600",
                "--window-size-px",
                "1600x1200",
            ],
        ] {
            assert!(parse_command(args(&values)).is_err(), "{values:?}");
        }
    }

    #[test]
    fn native_measurement_mode_rejects_invalid_or_ambiguous_frame_targets() {
        assert!(parse_command(args(&["--rl2-measure", "--submitted-frames"])).is_err());
        assert!(parse_command(args(&["--rl2-measure", "--submitted-frames", "0"])).is_err());
        assert!(parse_command(args(&["--rl2-measure", "--submitted-frames", "nope"])).is_err());
        assert!(
            parse_command(args(&[
                "--rl2-measure",
                "evidence/run.json",
                "--submitted-frames",
                "60",
                "extra"
            ]))
            .is_err()
        );
    }
}
