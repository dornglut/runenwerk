use std::env;
use std::ffi::{OsStr, OsString};

use anyhow::{Context, bail};
use editor_shell::ViewportToolKind;
use engine::automation::{
    AutomationExecutionMode, AutomationSession, AutomationSessionId, AutomationStepResult,
    InputSourceId,
};
use runenwerk_editor::automation::{
    EditorAutomationAdapter, EditorAutomationCommand, EditorAutomationObservation,
    EditorAutomationQuery, resolve_unique_scene_viewport_target,
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

fn try_run_automation(
    args: impl IntoIterator<Item = OsString>,
) -> Option<anyhow::Result<()>> {
    let mut args = args.into_iter();
    let first = args.next()?;
    if first != "--automation-viewport-tool" {
        return None;
    }

    Some(run_viewport_tool_automation(args))
}

fn run_viewport_tool_automation(
    mut args: impl Iterator<Item = OsString>,
) -> anyhow::Result<()> {
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
