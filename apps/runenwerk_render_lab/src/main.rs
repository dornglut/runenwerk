use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{Read, Take};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use engine::automation::{
    AutomationExecutionMode, AutomationPersistedReplayError, AutomationScenarioStepV1,
    AutomationSession, AutomationSessionId, AutomationStepResult, InputSourceId,
    MAX_ARTIFACT_BYTES, MAX_AUTOMATION_SCENARIO_BYTES,
    execute_automation_scenario_step_with_history, import_automation_scenario_v1,
};
use runenwerk_render_lab::automation::{
    RENDER_LAB_AUTOMATION_SCENARIO_PRODUCT_ID, RENDER_LAB_AUTOMATION_SCENARIO_PRODUCT_VERSION,
    RenderLabAutomationAdapter, RenderLabAutomationQuery, RenderLabAutomationScenarioStepV1,
    RenderLabAutomationTarget, RenderLabCameraObservation, build_headless_automation_app,
    validate_render_lab_automation_scenario_step_v1,
};

fn main() -> anyhow::Result<()> {
    match parse_command(env::args_os().skip(1))? {
        Command::Native => runenwerk_render_lab::run_native(),
        Command::Comparison {
            window_size_px,
            candidate_size_px,
        } => runenwerk_render_lab::run_native_comparison(window_size_px, candidate_size_px),
        Command::ReplayTrace(path) => run_replay_trace(&path),
        Command::AutomationScenario(path) => run_automation_scenario(&path),
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
        Command::TemporalCameraQuality {
            output_root,
            submitted_frame_limit,
            window_size_px,
        } => runenwerk_render_lab::run_native_temporal_camera_quality(
            output_root,
            submitted_frame_limit,
            window_size_px,
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
    Comparison {
        window_size_px: (u32, u32),
        candidate_size_px: (u32, u32),
    },
    ReplayTrace(PathBuf),
    AutomationScenario(PathBuf),
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
    TemporalCameraQuality {
        output_root: PathBuf,
        submitted_frame_limit: Option<usize>,
        window_size_px: (u32, u32),
    },
}

fn parse_command(args: impl IntoIterator<Item = OsString>) -> anyhow::Result<Command> {
    let mut args = args.into_iter();
    let first = args.next();
    if matches!(first.as_deref(), Some(value) if value == "--rl2" || value == "--native") {
        return Ok(Command::Native);
    }
    if matches!(first.as_deref(), Some(value) if value == "--rl2-compare") {
        let mut window_size_px = None;
        let mut candidate_size_px = None;
        while let Some(flag) = args.next() {
            if flag == "--window-size-px" {
                if window_size_px.is_some() {
                    bail!("duplicate --window-size-px argument");
                }
                window_size_px = Some(parse_window_size_px(args.next())?);
            } else if flag == "--candidate-size-px" {
                if candidate_size_px.is_some() {
                    bail!("duplicate --candidate-size-px argument");
                }
                candidate_size_px = Some(parse_candidate_size_px(args.next())?);
            } else {
                bail!(
                    "unexpected RL2 comparison argument '{}'",
                    flag.to_string_lossy()
                );
            }
        }
        let window_size_px = window_size_px.ok_or_else(|| {
            anyhow::anyhow!("--rl2-compare requires --window-size-px WIDTHxHEIGHT")
        })?;
        let candidate_size_px = candidate_size_px.ok_or_else(|| {
            anyhow::anyhow!("--rl2-compare requires --candidate-size-px WIDTHxHEIGHT")
        })?;
        return Ok(Command::Comparison {
            window_size_px,
            candidate_size_px,
        });
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
    if matches!(first.as_deref(), Some(value) if value == "--automation-scenario") {
        let Some(path) = args.next() else {
            bail!("--automation-scenario requires a persisted scenario path");
        };
        if let Some(extra) = args.next() {
            bail!(
                "unexpected --automation-scenario argument '{}'",
                extra.to_string_lossy()
            );
        }
        return Ok(Command::AutomationScenario(PathBuf::from(path)));
    }
    if matches!(first.as_deref(), Some(value) if value == "--rl2-camera-quality") {
        let default_output = PathBuf::from("render-lab/rl2-camera-motion-quality");
        let mut args = args.peekable();
        let output_root = match args.peek() {
            Some(value) if !value.to_string_lossy().starts_with("--") => {
                PathBuf::from(args.next().expect("peeked camera quality output root"))
            }
            _ => default_output,
        };
        let mut submitted_frame_limit = None;
        let mut window_size_px = None;
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
            } else {
                bail!(
                    "unexpected RL2 camera-motion quality argument '{}'",
                    flag.to_string_lossy()
                );
            }
        }
        let window_size_px = window_size_px.ok_or_else(|| {
            anyhow::anyhow!("--rl2-camera-quality requires --window-size-px WIDTHxHEIGHT")
        })?;
        return Ok(Command::TemporalCameraQuality {
            output_root,
            submitted_frame_limit,
            window_size_px,
        });
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

fn run_automation_scenario(path: &Path) -> anyhow::Result<()> {
    let canonical_scenario = fs::canonicalize(path)
        .with_context(|| format!("resolve persisted automation scenario {}", path.display()))?;
    let scenario_root = canonical_scenario
        .parent()
        .ok_or_else(|| anyhow::anyhow!("automation scenario has no parent directory"))?
        .to_path_buf();
    let scenario_bytes =
        read_bounded_scenario(File::open(&canonical_scenario).with_context(|| {
            format!(
                "open persisted automation scenario {}",
                canonical_scenario.display()
            )
        })?)?;
    let scenario = import_automation_scenario_v1::<RenderLabAutomationScenarioStepV1, _>(
        &scenario_bytes,
        RENDER_LAB_AUTOMATION_SCENARIO_PRODUCT_ID,
        RENDER_LAB_AUTOMATION_SCENARIO_PRODUCT_VERSION,
        validate_render_lab_automation_scenario_step_v1,
    )
    .map_err(anyhow::Error::new)?;

    let mut app = build_headless_automation_app();
    let mut session =
        AutomationSession::new(AutomationSessionId::new(1_015), InputSourceId::new(40_115));
    let mut history_ranges = Vec::with_capacity(scenario.steps().len());

    for (ordinal, step) in scenario.steps().iter().enumerate() {
        let (step_result, history_range) =
            execute_automation_scenario_step_with_history(ordinal, &mut session, |session| {
                match step {
                    AutomationScenarioStepV1::ReplayNormalizedTrace(reference) => {
                        run_render_lab_scenario_replay_step(
                            session,
                            &mut app,
                            &scenario_root,
                            reference.as_str(),
                        )
                    }
                    AutomationScenarioStepV1::Owner(step) => {
                        run_render_lab_scenario_owner_step(session, &mut app, step)
                    }
                }
            });
        history_ranges.push(history_range);

        if let Err(primary) = step_result {
            let cleanup = session.cancel(&mut app);
            return match cleanup {
                AutomationStepResult::Cancelled => Err(anyhow::anyhow!(
                    "automation scenario step {ordinal} failed: {primary}"
                )),
                other => Err(anyhow::anyhow!(
                    "automation scenario step {ordinal} failed: {primary}; cleanup failed: {other:?}"
                )),
            };
        }
    }

    match session.finish(&mut app) {
        AutomationStepResult::EffectConfirmed(()) => {}
        other => bail!("finish Render Lab automation scenario failed: {other:?}"),
    }
    debug_assert_eq!(history_ranges.len(), scenario.steps().len());

    println!(
        "automation scenario completed: product={}/v{} steps={}",
        scenario.product_contract_id(),
        scenario.product_contract_version(),
        scenario.steps().len(),
    );
    Ok(())
}

fn run_render_lab_scenario_replay_step(
    session: &mut AutomationSession,
    app: &mut engine::prelude::App,
    scenario_root: &Path,
    reference: &str,
) -> Result<(), String> {
    let trace_path = resolve_scenario_artifact_path(scenario_root, reference)
        .map_err(|error| error.to_string())?;
    let trace_bytes = read_trace_file(&trace_path).map_err(|error| error.to_string())?;
    session
        .replay_persisted_normalized_trace(
            AutomationExecutionMode::NormalizedInput,
            app,
            &trace_bytes,
            engine::automation::AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn run_render_lab_scenario_owner_step(
    session: &mut AutomationSession,
    app: &mut engine::prelude::App,
    step: &RenderLabAutomationScenarioStepV1,
) -> Result<(), String> {
    match step {
        RenderLabAutomationScenarioStepV1::AssertCamera { expected } => {
            let observed = {
                let mut adapter = RenderLabAutomationAdapter::new(app);
                match session.query_owner(
                    &mut adapter,
                    &RenderLabAutomationTarget,
                    RenderLabAutomationQuery::Camera,
                ) {
                    AutomationStepResult::EffectConfirmed(observation) => observation,
                    other => return Err(format!("query Render Lab camera failed: {other:?}")),
                }
            };
            match session
                .assert_observation(&observed, |observation| expected.matches(*observation))
            {
                AutomationStepResult::AssertionPassed => Ok(()),
                other => Err(format!("assert Render Lab camera failed: {other:?}")),
            }
        }
    }
}

fn read_bounded_scenario(reader: impl Read) -> anyhow::Result<Vec<u8>> {
    let limit = u64::try_from(MAX_AUTOMATION_SCENARIO_BYTES)
        .expect("scenario byte limit fits u64")
        .checked_add(1)
        .expect("scenario byte limit leaves room for sentinel byte");
    let mut reader: Take<_> = reader.take(limit);
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .context("read persisted automation scenario bytes")?;
    if bytes.len() > MAX_AUTOMATION_SCENARIO_BYTES {
        bail!(
            "persisted automation scenario exceeds the {} byte limit",
            MAX_AUTOMATION_SCENARIO_BYTES
        );
    }
    Ok(bytes)
}

fn resolve_scenario_artifact_path(root: &Path, reference: &str) -> anyhow::Result<PathBuf> {
    let candidate = root.join(reference);
    let resolved = fs::canonicalize(&candidate).with_context(|| {
        format!(
            "resolve automation scenario artifact {}",
            candidate.display()
        )
    })?;
    if !resolved.starts_with(root) {
        bail!(
            "automation scenario artifact escapes scenario root: {}",
            resolved.display()
        );
    }
    Ok(resolved)
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
    let mut app = build_headless_automation_app();
    let mut session =
        AutomationSession::new(AutomationSessionId::new(900), InputSourceId::new(40_100));

    let report = match session.replay_persisted_normalized_trace(
        AutomationExecutionMode::NormalizedInput,
        &mut app,
        bytes,
        engine::automation::AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    ) {
        Ok(report) => report,
        Err(AutomationPersistedReplayError::Import(error)) => {
            return Err(anyhow::Error::new(error));
        }
        Err(error) => return Err(anyhow::Error::new(error)),
    };

    let camera_result = {
        let mut adapter = RenderLabAutomationAdapter::new(&mut app);
        session.query_owner(
            &mut adapter,
            &RenderLabAutomationTarget,
            RenderLabAutomationQuery::Camera,
        )
    };
    let finish_result = session.finish(&mut app);

    let camera = match camera_result {
        AutomationStepResult::EffectConfirmed(camera) => camera,
        other => bail!("query Render Lab camera after replay failed: {other:?}"),
    };
    match finish_result {
        AutomationStepResult::EffectConfirmed(()) => {}
        other => bail!("finish Render Lab automation replay failed: {other:?}"),
    }

    Ok(ReplayTraceSummary {
        completed_frames: report.completed_frames(),
        camera,
    })
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

fn parse_candidate_size_px(value: Option<OsString>) -> anyhow::Result<(u32, u32)> {
    let Some(value) = value else {
        bail!("--candidate-size-px requires WIDTHxHEIGHT");
    };
    let Some(value) = value.to_str() else {
        bail!("--candidate-size-px requires a UTF-8 WIDTHxHEIGHT value");
    };
    let Some((width, height)) = value.split_once('x') else {
        bail!("invalid --candidate-size-px value '{value}'; expected WIDTHxHEIGHT");
    };
    let width = width
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("invalid --candidate-size-px width in '{value}'"))?;
    let height = height
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("invalid --candidate-size-px height in '{value}'"))?;
    if width == 0 || height == 0 {
        bail!("--candidate-size-px requires positive WIDTHxHEIGHT");
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
    fn comparison_mode_requires_explicit_reference_and_candidate_extents() {
        assert_eq!(
            parse_command(args(&[
                "--rl2-compare",
                "--window-size-px",
                "1920x1080",
                "--candidate-size-px",
                "960x540",
            ]))
            .unwrap(),
            Command::Comparison {
                window_size_px: (1920, 1080),
                candidate_size_px: (960, 540),
            }
        );

        for values in [
            vec!["--rl2-compare"],
            vec!["--rl2-compare", "--window-size-px", "1920x1080"],
            vec!["--rl2-compare", "--candidate-size-px", "960x540"],
            vec![
                "--rl2-compare",
                "--window-size-px",
                "1920x1080",
                "--candidate-size-px",
                "960X540",
            ],
            vec![
                "--rl2-compare",
                "--window-size-px",
                "1920x1080",
                "--candidate-size-px",
                "0x540",
            ],
            vec![
                "--rl2-compare",
                "--window-size-px",
                "1920x1080",
                "--candidate-size-px",
                "960x540",
                "--candidate-size-px",
                "1280x720",
            ],
        ] {
            assert!(parse_command(args(&values)).is_err(), "{values:?}");
        }
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
    fn camera_motion_quality_mode_is_explicit_and_p100_by_construction() {
        assert_eq!(
            parse_command(args(&[
                "--rl2-camera-quality",
                "evidence/camera",
                "--window-size-px",
                "1920x1080",
                "--submitted-frames",
                "4"
            ]))
            .unwrap(),
            Command::TemporalCameraQuality {
                output_root: PathBuf::from("evidence/camera"),
                submitted_frame_limit: Some(4),
                window_size_px: (1920, 1080),
            }
        );
        assert!(parse_command(args(&["--rl2-camera-quality"])).is_err());
        assert!(
            parse_command(args(&[
                "--rl2-camera-quality",
                "--window-size-px",
                "1920x1080",
                "--internal-size-px",
                "960x540"
            ]))
            .is_err()
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
