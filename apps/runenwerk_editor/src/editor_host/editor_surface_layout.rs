use ui_math::UiRect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HostEditorSurfaceLayout {
	pub bounds: UiRect,
	pub toolbar_bounds: UiRect,
	pub outliner_bounds: UiRect,
	pub viewport_bounds: UiRect,
	pub inspector_bounds: UiRect,
}

impl HostEditorSurfaceLayout {
	pub fn new(bounds: UiRect) -> Self {
		let toolbar_height: f32 = 48.0;
		let gap: f32 = 8.0;
		let sidebar_width: f32 = 280.0;
		let inspector_width: f32 = 320.0;

		let body_y = bounds.y + toolbar_height + gap;
		let body_height = (bounds.height - toolbar_height - gap).max(0.0);

		let toolbar_bounds = UiRect::new(bounds.x, bounds.y, bounds.width, toolbar_height);

		let outliner_bounds = UiRect::new(
			bounds.x,
			body_y,
			sidebar_width.min(bounds.width),
			body_height,
		);

		let inspector_x = (bounds.x + bounds.width - inspector_width).max(bounds.x);
		let inspector_bounds = UiRect::new(
			inspector_x,
			body_y,
			inspector_width.min(bounds.width),
			body_height,
		);

		let viewport_x = outliner_bounds.x + outliner_bounds.width + gap;
		let viewport_right = inspector_bounds.x - gap;
		let viewport_width = (viewport_right - viewport_x).max(0.0);

		let viewport_bounds = UiRect::new(viewport_x, body_y, viewport_width, body_height);

		Self {
			bounds,
			toolbar_bounds,
			outliner_bounds,
			viewport_bounds,
			inspector_bounds,
		}
	}

	pub fn shell_bounds(&self) -> UiRect {
		self.bounds
	}

	pub fn contains_viewport_point(
		&self,
		x: f32,
		y: f32,
	) -> bool {
		self.viewport_bounds.contains(ui_math::UiPoint::new(x, y))
	}

	pub fn contains_shell_point(
		&self,
		x: f32,
		y: f32,
	) -> bool {
		self.bounds.contains(ui_math::UiPoint::new(x, y))
	}
}