use super::{
    GameCommand, apply_game_command, clamp_scrollback_lines, flush_paused_logs, parse_command_line,
};
use engine::plugins::render::domain::{PassSlot, PipelineKey};
use engine::plugins::scene::domain::{OverlayCommandInput, OverlaySubmitMessage};
use engine::plugins::scene::{
    pop_overlay as pop_overlay_scene, push_overlay_by_id, set_world_by_id, set_world_paused,
    switch_scene_by_id, toggle_world_pause,
};
use engine::plugins::ui::domain::UiDirty;
use engine::runtime::EngineData;

fn drain_submit_lines(data: &mut EngineData) -> Vec<OverlaySubmitMessage> {
    std::mem::take(&mut data.scene.channels.overlay_submit)
}

fn drain_command_inputs(data: &mut EngineData) -> Vec<OverlayCommandInput> {
    std::mem::take(&mut data.scene.channels.overlay_command_inputs)
}

#[derive(Debug, Clone)]
enum SceneOp {
    SetScene(String),
    SetWorldById(String),
    PushOverlayById(String),
    PopOverlay,
    SetWorldPaused(bool),
    ToggleWorldPause,
    SetLogsPaused(bool),
    ToggleLogsPaused,
    FlushPausedLogs,
    SetEditorStatus(String),
}

#[derive(Debug, Clone, Copy)]
enum GfxOp {
    SetPipeline { slot: PassSlot, key: PipelineKey },
    ForceShaderReload,
    SetShaderWatch(bool),
    ShaderStatus,
    ModelStatus,
    ForceModelReload,
    SetModelWatch(bool),
}

#[derive(Debug, Clone)]
struct CommandOutcome {
    command_for_feedback: GameCommand,
    messages: Vec<String>,
    scene_ops: Vec<SceneOp>,
    gfx_ops: Vec<GfxOp>,
}

impl CommandOutcome {
    fn from_command(command: GameCommand) -> Self {
        Self {
            command_for_feedback: command,
            messages: Vec::new(),
            scene_ops: Vec::new(),
            gfx_ops: Vec::new(),
        }
    }
}

pub fn game_command_apply_system(data: &mut EngineData) -> anyhow::Result<()> {
    let lines = drain_submit_lines(data);
    if lines.is_empty() {
        return Ok(());
    }

    data.scene
        .channels
        .overlay_command_inputs
        .extend(lines.into_iter().map(|submit| match submit {
            OverlaySubmitMessage::Line(line) => OverlayCommandInput::Line(line),
        }));

    Ok(())
}

pub fn game_command_execute_system(data: &mut EngineData) -> anyhow::Result<()> {
    let inputs = drain_command_inputs(data);
    if inputs.is_empty() {
        return Ok(());
    }

    for input in inputs {
        let OverlayCommandInput::Line(line) = input;
        let Some(command) = parse_command_line(&line) else {
            continue;
        };

        let mut outcome = plan_command(command);
        apply_gfx_ops(data, &mut outcome);
        apply_scene_ops(data, outcome.scene_ops);

        data.scene
            .overlay_runtime
            .ui
            .lines
            .extend(outcome.messages.into_iter());
        apply_game_command(
            &mut data.scene.overlay_runtime.ui.lines,
            outcome.command_for_feedback,
        );
        clamp_scrollback_lines(
            &mut data.scene.overlay_runtime.ui.lines,
            data.scene.overlay_runtime.ui.max_lines,
        );
    }

    data.scene.overlay_runtime.ui.scroll_lines_from_bottom = 0;
    let scroll_entity = data.scene.overlay_runtime.ui.scrollback;
    if let Some(dirty) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiDirty>(scroll_entity)
    {
        dirty.text = true;
    }

    Ok(())
}

fn plan_command(command: GameCommand) -> CommandOutcome {
    let mut outcome = CommandOutcome::from_command(command.clone());
    match command {
        GameCommand::SetWorld(scene_label) => {
            outcome.scene_ops.push(SceneOp::SetWorldById(scene_label));
        }
        GameCommand::SetScene(scene_id) => {
            outcome.scene_ops.push(SceneOp::SetScene(scene_id));
        }
        GameCommand::PushOverlay(scene_label) => {
            outcome
                .scene_ops
                .push(SceneOp::PushOverlayById(scene_label));
        }
        GameCommand::PopOverlay => {
            outcome.scene_ops.push(SceneOp::PopOverlay);
        }
        GameCommand::PauseLogs => {
            outcome.scene_ops.push(SceneOp::SetLogsPaused(true));
        }
        GameCommand::ResumeLogs => {
            outcome.scene_ops.push(SceneOp::SetLogsPaused(false));
            outcome.scene_ops.push(SceneOp::FlushPausedLogs);
        }
        GameCommand::ToggleLogs => {
            outcome.scene_ops.push(SceneOp::ToggleLogsPaused);
        }
        GameCommand::FreezeTime => {
            outcome.scene_ops.push(SceneOp::SetWorldPaused(true));
        }
        GameCommand::ResumeTime => {
            outcome.scene_ops.push(SceneOp::SetWorldPaused(false));
        }
        GameCommand::ToggleTime => {
            outcome.scene_ops.push(SceneOp::ToggleWorldPause);
        }
        GameCommand::SetPipeline { slot, key } => {
            outcome.gfx_ops.push(GfxOp::SetPipeline { slot, key });
        }
        GameCommand::ReloadShaders => {
            outcome.gfx_ops.push(GfxOp::ForceShaderReload);
        }
        GameCommand::ShaderWatch(enabled) => {
            outcome.gfx_ops.push(GfxOp::SetShaderWatch(enabled));
        }
        GameCommand::ShaderStatus => {
            outcome.gfx_ops.push(GfxOp::ShaderStatus);
        }
        GameCommand::Models | GameCommand::ModelStatus => {
            outcome.gfx_ops.push(GfxOp::ModelStatus);
        }
        GameCommand::ReloadModels => {
            outcome.gfx_ops.push(GfxOp::ForceModelReload);
        }
        GameCommand::ModelWatch(enabled) => {
            outcome.gfx_ops.push(GfxOp::SetModelWatch(enabled));
        }
        _ => {}
    }

    outcome
}

fn apply_gfx_ops(data: &mut EngineData, outcome: &mut CommandOutcome) {
    for op in std::mem::take(&mut outcome.gfx_ops) {
        match op {
            GfxOp::SetPipeline { slot, key } => {
                if let Err(err) = data.gfx.set_pipeline_for_slot(slot, key) {
                    outcome.command_for_feedback =
                        GameCommand::Invalid(format!("set_pipeline failed: {err}"));
                } else {
                    outcome.scene_ops.push(SceneOp::SetEditorStatus(format!(
                        "editor: pipeline {} -> {}",
                        slot.label(),
                        key.label()
                    )));
                }
            }
            GfxOp::ForceShaderReload => {
                for msg in data.gfx.force_shader_reload() {
                    outcome.messages.push(format!("[world] {msg}"));
                }
            }
            GfxOp::SetShaderWatch(enabled) => {
                data.gfx.set_shader_watch_enabled(enabled);
            }
            GfxOp::ShaderStatus => {
                outcome.messages.extend(data.gfx.shader_status_lines());
            }
            GfxOp::ModelStatus => {
                outcome.messages.extend(data.gfx.model_status_lines());
            }
            GfxOp::ForceModelReload => {
                for msg in data.gfx.force_model_reload() {
                    outcome.messages.push(format!("[world] {msg}"));
                }
            }
            GfxOp::SetModelWatch(enabled) => {
                data.gfx.set_model_watch_enabled(enabled);
            }
        }
    }
}

fn apply_scene_ops(data: &mut EngineData, scene_ops: Vec<SceneOp>) {
    for op in scene_ops {
        match op {
            SceneOp::SetScene(scene_id) => {
                let switched = switch_scene_by_id(data, &scene_id).unwrap_or_else(|err| {
                    data.scene.overlay_runtime.ui.editor.status =
                        format!("editor: failed to switch scene '{scene_id}': {err}");
                    false
                });
                if !switched {
                    data.scene.overlay_runtime.ui.editor.status =
                        format!("editor: unknown scene '{scene_id}'");
                }
            }
            SceneOp::SetWorldById(scene_id) => {
                let switched = set_world_by_id(data, &scene_id).unwrap_or_else(|err| {
                    data.scene.overlay_runtime.ui.editor.status =
                        format!("editor: failed to switch world '{scene_id}': {err}");
                    false
                });
                if !switched {
                    data.scene.overlay_runtime.ui.editor.status =
                        format!("editor: unknown world scene '{scene_id}'");
                }
            }
            SceneOp::PushOverlayById(scene_id) => {
                let switched = push_overlay_by_id(data, &scene_id).unwrap_or_else(|err| {
                    data.scene.overlay_runtime.ui.editor.status =
                        format!("editor: failed to push overlay '{scene_id}': {err}");
                    false
                });
                if !switched {
                    data.scene.overlay_runtime.ui.editor.status =
                        format!("editor: unknown overlay scene '{scene_id}'");
                }
            }
            SceneOp::PopOverlay => {
                pop_overlay_scene(data);
            }
            SceneOp::SetWorldPaused(paused) => {
                set_world_paused(data, paused);
            }
            SceneOp::ToggleWorldPause => {
                toggle_world_pause(data);
            }
            SceneOp::SetLogsPaused(paused) => {
                data.scene.overlay_runtime.ui.logs_paused = paused;
            }
            SceneOp::ToggleLogsPaused => {
                data.scene.overlay_runtime.ui.logs_paused =
                    !data.scene.overlay_runtime.ui.logs_paused;
                if !data.scene.overlay_runtime.ui.logs_paused {
                    flush_paused_logs(&mut data.scene.overlay_runtime.ui);
                }
            }
            SceneOp::FlushPausedLogs => {
                flush_paused_logs(&mut data.scene.overlay_runtime.ui);
            }
            SceneOp::SetEditorStatus(status) => {
                data.scene.overlay_runtime.ui.editor.status = status;
            }
        }
    }
}
