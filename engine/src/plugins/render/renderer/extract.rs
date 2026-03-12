use super::*;

impl Renderer {
    pub(super) fn extract_rect_instances(draw_list: &UiDrawList) -> Vec<RectInstanceRaw> {
        let mut instances = Vec::new();
        for cmd in &draw_list.commands {
            if let UiDrawCmd::Rect {
                x,
                y,
                w,
                h,
                color,
                radius,
            } = cmd
            {
                instances.push(RectInstanceRaw {
                    rect: [*x, *y, *w, *h],
                    color: *color,
                    radius: *radius,
                    _pad: [0.0; 3],
                });
            }
        }
        instances
    }
}
