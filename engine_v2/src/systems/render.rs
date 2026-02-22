use crate::runtime::EngineData;
use crate::runtime::{AgentHealth, AgentPosition, AgentTeam};
use crate::render::{MAX_WORLD_RENDER_AGENTS, WorldRenderAgent};
use wgpu::SurfaceError;

pub fn world_render_extract_system(data: &mut EngineData) -> anyhow::Result<()> {
    let frame = &mut data.world_render;
    frame.agents.clear();
    frame.model_proxies.clear();
    frame.world_paused = data.scene.world.paused;

    let bounds = &data.scene.world_runtime.ctx.gameplay_config.bounds;
    frame.world_bounds = [bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y];

    if !data.scene.world.visible {
        return Ok(());
    }

    let world = &data.scene.world_runtime.ctx.world;
    let entities: Vec<_> = world.entities_with::<AgentPosition>().collect();
    for entity in entities.into_iter().take(MAX_WORLD_RENDER_AGENTS) {
        let Some(position) = world.get_component::<AgentPosition>(entity).copied() else {
            continue;
        };
        let health = world
            .get_component::<AgentHealth>(entity)
            .copied()
            .unwrap_or(AgentHealth { current: 1, max: 1 });
        let team = world
            .get_component::<AgentTeam>(entity)
            .copied()
            .unwrap_or(AgentTeam::Enemy);
        let health_ratio = if health.max > 0 {
            (health.current as f32 / health.max as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let team_id = match team {
            AgentTeam::Player => 0,
            AgentTeam::Enemy => 1,
        };
        frame.agents.push(WorldRenderAgent {
            x: position.x,
            y: position.y,
            radius: 0.95,
            health_ratio,
            team: team_id,
        });
    }

    Ok(())
}

pub fn ui_render_submit_system(data: &mut EngineData) -> anyhow::Result<()> {
    let shader_reload_messages = data.gfx.poll_shader_hot_reload();
    if !shader_reload_messages.is_empty() {
        for msg in shader_reload_messages {
            data.scene
                .overlay_runtime
                .ui
                .log_lines
                .push(format!("[world] shader {msg}"));
        }
        clamp_lines(
            &mut data.scene.overlay_runtime.ui.log_lines,
            data.scene.overlay_runtime.ui.max_lines,
        );
        data.scene.overlay_runtime.ui.log_scroll_lines_from_bottom = 0;
    }
    let model_reload_messages = data.gfx.poll_model_hot_reload();
    if !model_reload_messages.is_empty() {
        for msg in model_reload_messages {
            data.scene
                .overlay_runtime
                .ui
                .log_lines
                .push(format!("[world] model {msg}"));
        }
        clamp_lines(
            &mut data.scene.overlay_runtime.ui.log_lines,
            data.scene.overlay_runtime.ui.max_lines,
        );
        data.scene.overlay_runtime.ui.log_scroll_lines_from_bottom = 0;
    }

    match data
        .gfx
        .render(&data.world_render, &data.scene.overlay_runtime.ui.draw_list)
    {
        Ok(()) => Ok(()),
        Err(SurfaceError::Lost | SurfaceError::Outdated) => {
            let (w, h) = data.scene.overlay_runtime.ui.screen_size;
            data.gfx.resize(w as u32, h as u32);
            Ok(())
        }
        Err(SurfaceError::Timeout) => Ok(()),
        Err(SurfaceError::OutOfMemory) => anyhow::bail!("surface out of memory"),
        Err(SurfaceError::Other) => Ok(()),
    }
}

fn clamp_lines(lines: &mut Vec<String>, max_lines: usize) {
    if max_lines == 0 {
        lines.clear();
        return;
    }
    let overflow = lines.len().saturating_sub(max_lines);
    if overflow > 0 {
        lines.drain(..overflow);
    }
}
