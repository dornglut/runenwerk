use engine::prelude::Component;
use serde::{Deserialize, Serialize};

// src/domain/gameplay/components/actor.rs
#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn ratio(self) -> f32 {
        if self.max <= f32::EPSILON {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub enum Faction {
    Hunters,
    CavernBeasts,
    Neutral,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct HitFlashState {
    pub remaining_seconds: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct DamageFeedbackState {
    pub last_damage_taken: f32,
    pub last_damage_dealt: f32,
}
