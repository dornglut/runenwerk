use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Take};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use editor_shell::ViewportToolKind;
use engine::automation::{
    AutomationExecutionMode, AutomationScenarioStepHistoryRange, AutomationScenarioStepV1,
    AutomationSession, AutomationSessionId, AutomationStepResult, InputSourceId,
    MAX_AUTOMATION_SCENARIO_BYTES, import_automation_scenario_v1,
};
use runenwerk_editor::automation::{
    EDITOR_AUTOMATION_SCENARIO_PRODUCT_ID, EDITOR_AUTOMATION_SCENARIO_PRODUCT_VERSION,
    EditorAutomationAdapter, EditorAutomationCommand, EditorAutomationObservation,
    EditorAutomationQuery, EditorAutomationScenarioStepV1,
    resolve_editor_automation_scenario_target_v1, resolve_unique_scene_viewport_target,
    validate_editor_automation_scenario_step_v1,
};
use runenwerk_editor::runtime::resources::EditorHostResource;

fn main() {
    if let Some(result) = try_run_automation(env::args_os().skip(1)) {
        if let Err(error) = result {
            eprintln!("{error:#}");
            std::process::exit(2);
        }
        return;
    }

    runenwerk_editor::runtime::run().expect("runenwerk editor runtime should start");
}

fn try_run_automation(args: impl IntoIterator<Item = OsString>) -> Option<anyhow::Result<()>> {
    let mut args = args.into_iter();
    let first = args.next()?;
    if first == "--automation-viewport-tool" {
        return Some(run_viewport_tool_automation(args));
    }
    if first == "--automation-scenario" {
        return Some(run_persisted_automation_scenario(args));
    }

    None
}

fn run_viewport_tool_automation(mut args: impl Iterator<Item = OsString>) -> anyhow::Result<()> {
    let tool = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("--automation-viewport-tool requires a tool"))?;
    if let Some(extra) = args.next() {
        bail!(
            "unexpected --automation-viewport-tool argument '{}'",
            extra.to_string_lossy()
        );
    }
    let tool = parse_viewport_tool(&tool)?;

    let mut app = runenwerk_editor::runtime::build_headless_app()
        .context("build headless Editor automation target")?;
    let target = {
        let host = app
            .world()
            .resource::<EditorHostResource>()
            .map_err(|_| anyhow::anyhow!("headless Editor automation host is unavailable"))?;
        resolve_unique_scene_viewport_target(host).map_err(anyhow::Error::msg)?
    };

    let mut session =
        AutomationSession::new(AutomationSessionId::new(1_009), InputSourceId::new(10_009));

    let observed = {
        let host = app
            .world_mut()
            .resource_mut::<EditorHostResource>()
            .map_err(|_| anyhow::anyhow!("headless Editor automation host is unavailable"))?;
        let mut adapter = EditorAutomationAdapter::new(host);

        match session.dispatch_product(
            AutomationExecutionMode::ProductSemantic,
            &mut adapter,
            &target,
            EditorAutomationCommand::ActivateViewportTool(tool),
        ) {
            AutomationStepResult::Dispatched => {}
            other => bail!("dispatch Editor viewport tool failed: {other:?}"),
        }

        match session.query_owner(&mut adapter, &target, EditorAutomationQuery::ViewportTool) {
            AutomationStepResult::EffectConfirmed(observation) => observation,
            other => bail!("query Editor viewport tool failed: {other:?}"),
        }
    };

    match session.assert_observation(&observed, |observation| {
        *observation == EditorAutomationObservation::ViewportTool(tool)
    }) {
        AutomationStepResult::AssertionPassed => {}
        other => bail!("assert Editor viewport tool failed: {other:?}"),
    }

    match session.finish(&mut app) {
        AutomationStepResult::EffectConfirmed(()) => {}
        other => bail!("finish Editor automation session failed: {other:?}"),
    }

    println!(
        "automation completed: viewport_tool={}",
        viewport_tool_name(tool)
    );
    Ok(())
}

fn run_persisted_automation_scenario(
    mut args: impl Iterator<Item = OsString>,
) -> anyhow::Result<()> {
    let path = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("--automation-scenario requires a scenario path"))?;
    if let Some(extra) = args.next() {
        bail!(
            "unexpected --automation-scenario argument '{}'",
            extra.to_string_lossy()
        );
    }
    let path = PathBuf::from(path);
    let bytes = read_scenario_file(&path)?;
    let scenario = import_automation_scenario_v1::<EditorAutomationScenarioStepV1, _>(
        &bytes,
        EDITOR_AUTOMATION_SCENARIO_PRODUCT_ID,
        EDITOR_AUTOMATION_SCENARIO_PRODUCT_VERSION,
        validate_editor_automation_scenario_step_v1,
    )
    .map_err(anyhow::Error::new)?;

    let mut app = runenwerk_editor::runtime::build_headless_app()
        .context("build headless Editor automation scenario target")?;
    let mut session =
        AutomationSession::new(AutomationSessionId::new(1_015), InputSourceId::new(10_015));
    let mut history_ranges = Vec::with_capacity(scenario.steps().len());

    for (ordinal, step) in scenario.steps().iter().enumerate() {
        let history_start = session.history().len();
        let step_result = match step {
            AutomationScenarioStepV1::Owner(step) => {
                run_editor_scenario_owner_step(&mut session, &mut app, step)
            }
            AutomationScenarioStepV1::ReplayNormalizedTrace(_) => Err(
                "Editor automation scenario V1 does not support normalized trace replay".to_owned(),
            ),
        };
        let history_end = session.history().len();
        history_ranges.push(AutomationScenarioStepHistoryRange::new(
            ordinal,
            history_start,
            history_end,
        ));

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
        other => bail!("finish Editor automation scenario failed: {other:?}"),
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

fn run_editor_scenario_owner_step(
    session: &mut AutomationSession,
    app: &mut engine::prelude::App,
    step: &EditorAutomationScenarioStepV1,
) -> Result<(), String> {
    match step {
        EditorAutomationScenarioStepV1::ActivateViewportTool { target, tool } => {
            let target = {
                let host = app
                    .world()
                    .resource::<EditorHostResource>()
                    .map_err(|_| "headless Editor automation host is unavailable".to_owned())?;
                resolve_editor_automation_scenario_target_v1(host, *target)?
            };
            let host = app
                .world_mut()
                .resource_mut::<EditorHostResource>()
                .map_err(|_| "headless Editor automation host is unavailable".to_owned())?;
            let mut adapter = EditorAutomationAdapter::new(host);
            match session.dispatch_product(
                AutomationExecutionMode::ProductSemantic,
                &mut adapter,
                &target,
                EditorAutomationCommand::ActivateViewportTool((*tool).into()),
            ) {
                AutomationStepResult::Dispatched => Ok(()),
                other => Err(format!("dispatch Editor viewport tool failed: {other:?}")),
            }
        }
        EditorAutomationScenarioStepV1::AssertViewportTool { target, tool } => {
            let target = {
                let host = app
                    .world()
                    .resource::<EditorHostResource>()
                    .map_err(|_| "headless Editor automation host is unavailable".to_owned())?;
                resolve_editor_automation_scenario_target_v1(host, *target)?
            };
            let observed = {
                let host = app
                    .world_mut()
                    .resource_mut::<EditorHostResource>()
                    .map_err(|_| "headless Editor automation host is unavailable".to_owned())?;
                let mut adapter = EditorAutomationAdapter::new(host);
                match session.query_owner(
                    &mut adapter,
                    &target,
                    EditorAutomationQuery::ViewportTool,
                ) {
                    AutomationStepResult::EffectConfirmed(observation) => observation,
                    other => return Err(format!("query Editor viewport tool failed: {other:?}")),
                }
            };
            match session.assert_observation(&observed, |observation| {
                *observation == EditorAutomationObservation::ViewportTool((*tool).into())
            }) {
                AutomationStepResult::AssertionPassed => Ok(()),
                other => Err(format!("assert Editor viewport tool failed: {other:?}")),
            }
        }
    }
}

fn read_scenario_file(path: &Path) -> anyhow::Result<Vec<u8>> {
    let file = File::open(path)
        .with_context(|| format!("open persisted automation scenario {}", path.display()))?;
    let limit = u64::try_from(MAX_AUTOMATION_SCENARIO_BYTES)
        .expect("scenario byte limit fits u64")
        .checked_add(1)
        .expect("scenario byte limit leaves room for sentinel byte");
    let mut reader: Take<_> = file.take(limit);
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .with_context(|| format!("read persisted automation scenario {}", path.display()))?;
    if bytes.len() > MAX_AUTOMATION_SCENARIO_BYTES {
        bail!(
            "persisted automation scenario exceeds the {} byte limit",
            MAX_AUTOMATION_SCENARIO_BYTES
        );
    }
    Ok(bytes)
}

fn parse_viewport_tool(value: &OsStr) -> anyhow::Result<ViewportToolKind> {
    match value.to_str() {
        Some("select") => Ok(ViewportToolKind::Select),
        Some("translate") => Ok(ViewportToolKind::Translate),
        Some("rotate") => Ok(ViewportToolKind::Rotate),
        Some("scale") => Ok(ViewportToolKind::Scale),
        Some(value) => bail!(
            "unsupported viewport tool '{value}'; expected select, translate, rotate, or scale"
        ),
        None => bail!("--automation-viewport-tool requires a UTF-8 tool name"),
    }
}

const fn viewport_tool_name(tool: ViewportToolKind) -> &'static str {
    match tool {
        ViewportToolKind::Select => "select",
        ViewportToolKind::Translate => "translate",
        ViewportToolKind::Rotate => "rotate",
        ViewportToolKind::Scale => "scale",
    }
}
