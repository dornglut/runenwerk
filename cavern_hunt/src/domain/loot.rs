use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponModKind {
    DamageUp,
    FireRateUp,
    PierceOne,
    ProjectileSpeedUp,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelicKind {
    MaxHealthUp,
    DashCooldownDown,
    CritChanceUp,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum PickupKind {
    Scrap(u32),
    WeaponMod(WeaponModKind),
    Relic(RelicKind),
    HealingCharge(u32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnemyDropTable {
    pub guaranteed_scrap: u32,
    pub weapon_mod_chance: f32,
    pub healing_charge_chance: f32,
    pub relic_chance: f32,
}

impl EnemyDropTable {
    pub fn swarmer() -> Self {
        Self {
            guaranteed_scrap: 1,
            weapon_mod_chance: 0.08,
            healing_charge_chance: 0.05,
            relic_chance: 0.0,
        }
    }

    pub fn bruiser() -> Self {
        Self {
            guaranteed_scrap: 2,
            weapon_mod_chance: 0.15,
            healing_charge_chance: 0.08,
            relic_chance: 0.0,
        }
    }

    pub fn spitter() -> Self {
        Self {
            guaranteed_scrap: 2,
            weapon_mod_chance: 0.16,
            healing_charge_chance: 0.08,
            relic_chance: 0.0,
        }
    }

    pub fn elite() -> Self {
        Self {
            guaranteed_scrap: 8,
            weapon_mod_chance: 0.35,
            healing_charge_chance: 0.20,
            relic_chance: 1.0,
        }
    }
}
