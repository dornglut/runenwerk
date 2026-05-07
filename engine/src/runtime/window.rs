#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowCursorIcon {
    Default,
    ColResize,
    RowResize,
    NwseResize,
    NeswResize,
    Grab,
    Grabbing,
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct WindowState {
    pub title: String,
    pub size_px: (u32, u32),
    pub scale_factor: f64,
    pub close_requested: bool,
    pub redraw_requested: bool,
    pub cursor_icon: WindowCursorIcon,
    headless: bool,
}

impl WindowState {
    pub fn windowed(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            size_px: (1280, 720),
            scale_factor: 1.0,
            close_requested: false,
            redraw_requested: true,
            cursor_icon: WindowCursorIcon::Default,
            headless: false,
        }
    }

    pub fn headless(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            size_px: (1280, 720),
            scale_factor: 1.0,
            close_requested: false,
            redraw_requested: false,
            cursor_icon: WindowCursorIcon::Default,
            headless: true,
        }
    }

    pub fn request_close(&mut self) {
        self.close_requested = true;
    }

    pub fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.request_redraw();
    }

    pub fn set_cursor_icon(&mut self, cursor_icon: WindowCursorIcon) {
        if self.cursor_icon != cursor_icon {
            self.cursor_icon = cursor_icon;
            self.request_redraw();
        }
    }

    pub fn is_headless(&self) -> bool {
        self.headless
    }

    pub(crate) fn set_headless(&mut self, headless: bool) {
        self.headless = headless;
    }
}
