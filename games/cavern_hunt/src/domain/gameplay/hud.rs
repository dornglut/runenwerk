use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerStatusPanel {
	pub player_id: u32,
	pub label: String,
	pub alive: bool,
	pub is_companion: bool,
	pub scrap: u32,
	pub health_ratio: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunObjectivePanel {
	pub title: String,
	pub detail: String,
}

impl Default for RunObjectivePanel {
	fn default() -> Self {
		Self {
			title: "Explore the caverns".to_string(),
			detail: "Find the Nest Guardian".to_string(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionCountdownPanel {
	pub visible: bool,
	pub seconds_remaining: f32,
}

impl Default for ExtractionCountdownPanel {
	fn default() -> Self {
		Self {
			visible: false,
			seconds_remaining: 0.0,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernHudState {
	pub visible: bool,
	pub local_health: f32,
	pub local_max_health: f32,
	pub dash_cooldown_remaining: f32,
	pub scrap: u32,
	pub elite_defeated: bool,
	pub extraction_active: bool,
	pub objective: RunObjectivePanel,
	pub extraction: ExtractionCountdownPanel,
	pub teammates: Vec<PlayerStatusPanel>,
	pub status_lines: Vec<String>,
}

impl Default for CavernHudState {
	fn default() -> Self {
		Self {
			visible: true,
			local_health: 0.0,
			local_max_health: 0.0,
			dash_cooldown_remaining: 0.0,
			scrap: 0,
			elite_defeated: false,
			extraction_active: false,
			objective: RunObjectivePanel::default(),
			extraction: ExtractionCountdownPanel::default(),
			teammates: Vec::new(),
			status_lines: Vec::new(),
		}
	}
}