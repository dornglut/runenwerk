use engine::SimulationTick;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct CavernAimState {
	pub world_point: [f32; 2],
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernControlState {
	pub movement: [f32; 2],
	pub aim_world: [f32; 2],
	pub fire_pressed: bool,
	pub dash_pressed: bool,
	pub interact_pressed: bool,
	pub source_tick: SimulationTick,
}

impl Default for CavernControlState {
	fn default() -> Self {
		Self {
			movement: [0.0, 0.0],
			aim_world: [0.0, 0.0],
			fire_pressed: false,
			dash_pressed: false,
			interact_pressed: false,
			source_tick: SimulationTick::default(),
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernPredictedFrame {
	pub tick: SimulationTick,
	pub control: CavernControlState,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CavernPredictionState {
	pub pending_frames: Vec<CavernPredictedFrame>,
	pub corrections_applied: u64,
	pub last_authoritative_tick: SimulationTick,
}