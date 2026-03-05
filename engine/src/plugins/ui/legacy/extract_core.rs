// Owner: Grotto Quest Engine - UI Plugin
pub(crate) fn ui_render_extract_system(
    mut ui_overlay: ResMut<UiOverlayState>,
    mut scene_resource: ResMut<SceneResource>,
) -> anyhow::Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };
    let commands: Vec<UiDrawCmd> = manager
        .overlay_runtime
        .ui
        .batches
        .commands
        .iter()
        .map(|cmd| match cmd {
            UiBatchCmd::Rect {
                x,
                y,
                w,
                h,
                color,
                radius,
            } => UiDrawCmd::Rect {
                x: *x,
                y: *y,
                w: *w,
                h: *h,
                color: *color,
                radius: *radius,
            },
            UiBatchCmd::Text {
                x,
                y,
                content,
                color,
                size,
                clip,
            } => UiDrawCmd::Text {
                x: *x,
                y: *y,
                content: content.clone(),
                color: *color,
                size: *size,
                clip: *clip,
            },
        })
        .collect();

    manager.overlay_runtime.ui.draw_list.commands = commands.clone();
    ui_overlay.draw_list = OverlayDrawList {
        commands: commands
            .into_iter()
            .map(|cmd| match cmd {
                UiDrawCmd::Rect {
                    x,
                    y,
                    w,
                    h,
                    color,
                    radius,
                } => OverlayDrawCmd::Rect {
                    x,
                    y,
                    w,
                    h,
                    color,
                    radius,
                },
                UiDrawCmd::Text {
                    x,
                    y,
                    content,
                    color,
                    size,
                    clip,
                } => OverlayDrawCmd::Text {
                    x,
                    y,
                    content,
                    color,
                    size,
                    clip,
                },
            })
            .collect(),
    };
    Ok(())
}
