use crate::{ReplicationBudgetConfig, ReplicationCadenceConfig};

const ENV_NET_TUNING_PRESET: &str = "CAVERN_NET_TUNING_PRESET";

pub(super) fn apply_replication_tuning_preset(
    budget: &mut ReplicationBudgetConfig,
    cadence: &mut ReplicationCadenceConfig,
    preset: Option<&str>,
    diagnostics: &mut Vec<String>,
) {
    let Some(raw) = preset else {
        return;
    };
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "two_local" | "2local" | "balanced" => {
            budget.enemy_ops_per_patch_level0 = 160;
            budget.enemy_ops_per_patch_level1 = 96;
            budget.enemy_ops_per_patch_level2 = 48;
            budget.projectile_ops_per_patch_level0 = 320;
            budget.projectile_ops_per_patch_level1 = 176;
            budget.projectile_ops_per_patch_level2 = 88;
            budget.pickup_ops_per_patch_level0 = 64;
            budget.pickup_ops_per_patch_level1 = 40;
            budget.pickup_ops_per_patch_level2 = 20;
            budget.extraction_ops_per_patch_level0 = 16;
            budget.extraction_ops_per_patch_level1 = 10;
            budget.extraction_ops_per_patch_level2 = 6;

            cadence.patch_emit_interval_level0 = 1;
            cadence.patch_emit_interval_level1 = 1;
            cadence.patch_emit_interval_level2 = 2;
            cadence.enemy_patch_interval_level0 = 1;
            cadence.enemy_patch_interval_level1 = 2;
            cadence.enemy_patch_interval_level2 = 3;
            cadence.projectile_patch_interval_level0 = 1;
            cadence.projectile_patch_interval_level1 = 2;
            cadence.projectile_patch_interval_level2 = 2;
            cadence.pickup_patch_interval_level0 = 4;
            cadence.pickup_patch_interval_level1 = 5;
            cadence.pickup_patch_interval_level2 = 8;
            cadence.extraction_patch_interval_level0 = 1;
            cadence.extraction_patch_interval_level1 = 1;
            cadence.extraction_patch_interval_level2 = 1;
        }
        "four_local" | "4local" | "conservative" => {
            budget.enemy_ops_per_patch_level0 = 96;
            budget.enemy_ops_per_patch_level1 = 56;
            budget.enemy_ops_per_patch_level2 = 28;
            budget.projectile_ops_per_patch_level0 = 160;
            budget.projectile_ops_per_patch_level1 = 96;
            budget.projectile_ops_per_patch_level2 = 48;
            budget.pickup_ops_per_patch_level0 = 32;
            budget.pickup_ops_per_patch_level1 = 16;
            budget.pickup_ops_per_patch_level2 = 8;
            budget.extraction_ops_per_patch_level0 = 12;
            budget.extraction_ops_per_patch_level1 = 8;
            budget.extraction_ops_per_patch_level2 = 4;

            cadence.patch_emit_interval_level0 = 2;
            cadence.patch_emit_interval_level1 = 2;
            cadence.patch_emit_interval_level2 = 3;
            cadence.enemy_patch_interval_level0 = 2;
            cadence.enemy_patch_interval_level1 = 3;
            cadence.enemy_patch_interval_level2 = 4;
            cadence.projectile_patch_interval_level0 = 2;
            cadence.projectile_patch_interval_level1 = 3;
            cadence.projectile_patch_interval_level2 = 4;
            cadence.pickup_patch_interval_level0 = 6;
            cadence.pickup_patch_interval_level1 = 8;
            cadence.pickup_patch_interval_level2 = 12;
            cadence.extraction_patch_interval_level0 = 1;
            cadence.extraction_patch_interval_level1 = 2;
            cadence.extraction_patch_interval_level2 = 2;
        }
        "aggressive" | "lan" => {
            budget.enemy_ops_per_patch_level0 = 256;
            budget.enemy_ops_per_patch_level1 = 160;
            budget.enemy_ops_per_patch_level2 = 80;
            budget.projectile_ops_per_patch_level0 = 512;
            budget.projectile_ops_per_patch_level1 = 320;
            budget.projectile_ops_per_patch_level2 = 160;
            budget.pickup_ops_per_patch_level0 = 128;
            budget.pickup_ops_per_patch_level1 = 80;
            budget.pickup_ops_per_patch_level2 = 40;
            budget.extraction_ops_per_patch_level0 = 24;
            budget.extraction_ops_per_patch_level1 = 16;
            budget.extraction_ops_per_patch_level2 = 8;

            cadence.patch_emit_interval_level0 = 1;
            cadence.patch_emit_interval_level1 = 1;
            cadence.patch_emit_interval_level2 = 2;
            cadence.enemy_patch_interval_level0 = 1;
            cadence.enemy_patch_interval_level1 = 1;
            cadence.enemy_patch_interval_level2 = 2;
            cadence.projectile_patch_interval_level0 = 1;
            cadence.projectile_patch_interval_level1 = 1;
            cadence.projectile_patch_interval_level2 = 2;
            cadence.pickup_patch_interval_level0 = 2;
            cadence.pickup_patch_interval_level1 = 3;
            cadence.pickup_patch_interval_level2 = 5;
            cadence.extraction_patch_interval_level0 = 1;
            cadence.extraction_patch_interval_level1 = 1;
            cadence.extraction_patch_interval_level2 = 1;
        }
        _ => diagnostics.push(format!(
            "unknown {} preset '{}' (supported: two_local, four_local, aggressive)",
            ENV_NET_TUNING_PRESET, raw
        )),
    }
}
