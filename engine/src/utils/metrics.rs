#[derive(Debug)]
pub struct Metrics {
	accum_time: f32,
	frame_count: u32,

	// Smoothed values
	pub smoothed_fps: f32,
	pub smoothed_frame_ms: f32,

	// EWMA factor (0.0–1.0)
	smoothing: f32,
}

impl Metrics {
	pub fn new(smoothing: f32) -> Self {
		Self {
			accum_time: 0.0,
			frame_count: 0,
			smoothed_fps: 0.0,
			smoothed_frame_ms: 0.0,
			smoothing,
		}
	}

	pub fn update(&mut self, dt: f32) {
		// Clamp dt to avoid huge spikes from hitches
		let dt = dt.min(0.25);

		self.accum_time += dt;
		self.frame_count += 1;

		// Only update EWMA once per second
		if self.accum_time >= 1.0 {
			let current_fps = self.frame_count as f32 / self.accum_time;
			let current_ms = self.accum_time / self.frame_count as f32 * 1000.0;

			// Initialize on first measurement
			if self.smoothed_fps == 0.0 {
				self.smoothed_fps = current_fps;
				self.smoothed_frame_ms = current_ms;
			} else {
				self.smoothed_fps += (current_fps - self.smoothed_fps) * self.smoothing;
				self.smoothed_frame_ms += (current_ms - self.smoothed_frame_ms) * self.smoothing;
			}

			tracing::info!(
                "FPS: {:.1}, Avg Frame Time: {:.2} ms, Smoothed FPS: {:.1}, Smoothed Frame Time: {:.2} ms",
                current_fps,
                current_ms,
                self.smoothed_fps,
                self.smoothed_frame_ms
            );

			// Reset rolling counters
			self.accum_time = 0.0;
			self.frame_count = 0;
		}
	}
}