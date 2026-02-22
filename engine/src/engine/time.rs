use std::time::Instant;

pub struct Time {
	pub accumulator: f64,
	pub last_frame: Instant,
	pub last_dt: f32,
}

impl Time {
	pub fn new() -> Self {
		Self {
			accumulator: 0.0,
			last_frame: Instant::now(),
			last_dt: 1.0 / 60.0,
		}
	}

	pub fn tick(&mut self) -> f32 {
		let now = Instant::now();
		let dt = (now - self.last_frame).as_secs_f32().min(0.25);
		self.last_frame = now;
		self.last_dt = dt;
		dt
	}

	/// Returns the time delta of the last frame without updating the timer
	pub fn delta_seconds(&self) -> f32 {
		self.last_dt
	}
}