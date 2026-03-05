use crate::domain::{InterpolationConfig, ReplicationBudgetConfig, ReplicationCadenceConfig};

pub(super) fn apply_replication_tuning_overrides_from_reader<F>(
    budget: &mut ReplicationBudgetConfig,
    cadence: &mut ReplicationCadenceConfig,
    read_var: F,
    diagnostics: &mut Vec<String>,
) where
    F: Fn(&str) -> Option<String>,
{
    macro_rules! override_usize {
        ($field:expr, $name:literal, $min:expr, $max:expr) => {
            if let Some(parsed) = parse_env_usize(read_var($name), $name, $min, $max, diagnostics) {
                $field = parsed;
            }
        };
    }

    macro_rules! override_u64 {
        ($field:expr, $name:literal, $min:expr, $max:expr) => {
            if let Some(parsed) = parse_env_u64(read_var($name), $name, $min, $max, diagnostics) {
                $field = parsed;
            }
        };
    }

    override_usize!(
        budget.enemy_ops_per_patch_level0,
        "CAVERN_NET_BUDGET_ENEMY_L0",
        0,
        4096
    );
    override_usize!(
        budget.enemy_ops_per_patch_level1,
        "CAVERN_NET_BUDGET_ENEMY_L1",
        0,
        4096
    );
    override_usize!(
        budget.enemy_ops_per_patch_level2,
        "CAVERN_NET_BUDGET_ENEMY_L2",
        0,
        4096
    );
    override_usize!(
        budget.projectile_ops_per_patch_level0,
        "CAVERN_NET_BUDGET_PROJECTILE_L0",
        0,
        4096
    );
    override_usize!(
        budget.projectile_ops_per_patch_level1,
        "CAVERN_NET_BUDGET_PROJECTILE_L1",
        0,
        4096
    );
    override_usize!(
        budget.projectile_ops_per_patch_level2,
        "CAVERN_NET_BUDGET_PROJECTILE_L2",
        0,
        4096
    );
    override_usize!(
        budget.pickup_ops_per_patch_level0,
        "CAVERN_NET_BUDGET_PICKUP_L0",
        0,
        2048
    );
    override_usize!(
        budget.pickup_ops_per_patch_level1,
        "CAVERN_NET_BUDGET_PICKUP_L1",
        0,
        2048
    );
    override_usize!(
        budget.pickup_ops_per_patch_level2,
        "CAVERN_NET_BUDGET_PICKUP_L2",
        0,
        2048
    );
    override_usize!(
        budget.extraction_ops_per_patch_level0,
        "CAVERN_NET_BUDGET_EXTRACTION_L0",
        0,
        512
    );
    override_usize!(
        budget.extraction_ops_per_patch_level1,
        "CAVERN_NET_BUDGET_EXTRACTION_L1",
        0,
        512
    );
    override_usize!(
        budget.extraction_ops_per_patch_level2,
        "CAVERN_NET_BUDGET_EXTRACTION_L2",
        0,
        512
    );

    override_u64!(
        cadence.enemy_patch_interval_level0,
        "CAVERN_NET_CADENCE_ENEMY_L0",
        0,
        120
    );
    override_u64!(
        cadence.enemy_patch_interval_level1,
        "CAVERN_NET_CADENCE_ENEMY_L1",
        0,
        120
    );
    override_u64!(
        cadence.enemy_patch_interval_level2,
        "CAVERN_NET_CADENCE_ENEMY_L2",
        0,
        120
    );
    override_u64!(
        cadence.projectile_patch_interval_level0,
        "CAVERN_NET_CADENCE_PROJECTILE_L0",
        0,
        120
    );
    override_u64!(
        cadence.projectile_patch_interval_level1,
        "CAVERN_NET_CADENCE_PROJECTILE_L1",
        0,
        120
    );
    override_u64!(
        cadence.projectile_patch_interval_level2,
        "CAVERN_NET_CADENCE_PROJECTILE_L2",
        0,
        120
    );
    override_u64!(
        cadence.pickup_patch_interval_level0,
        "CAVERN_NET_CADENCE_PICKUP_L0",
        0,
        120
    );
    override_u64!(
        cadence.pickup_patch_interval_level1,
        "CAVERN_NET_CADENCE_PICKUP_L1",
        0,
        120
    );
    override_u64!(
        cadence.pickup_patch_interval_level2,
        "CAVERN_NET_CADENCE_PICKUP_L2",
        0,
        120
    );
    override_u64!(
        cadence.extraction_patch_interval_level0,
        "CAVERN_NET_CADENCE_EXTRACTION_L0",
        0,
        120
    );
    override_u64!(
        cadence.extraction_patch_interval_level1,
        "CAVERN_NET_CADENCE_EXTRACTION_L1",
        0,
        120
    );
    override_u64!(
        cadence.extraction_patch_interval_level2,
        "CAVERN_NET_CADENCE_EXTRACTION_L2",
        0,
        120
    );
    override_u64!(
        cadence.patch_emit_interval_level0,
        "CAVERN_NET_CADENCE_PATCH_EMIT_L0",
        1,
        120
    );
    override_u64!(
        cadence.patch_emit_interval_level1,
        "CAVERN_NET_CADENCE_PATCH_EMIT_L1",
        1,
        120
    );
    override_u64!(
        cadence.patch_emit_interval_level2,
        "CAVERN_NET_CADENCE_PATCH_EMIT_L2",
        1,
        120
    );
}

#[allow(dead_code)]
pub(super) fn apply_interpolation_overrides_from_reader<F>(
    interpolation: &mut InterpolationConfig,
    read_var: F,
    diagnostics: &mut Vec<String>,
) where
    F: Fn(&str) -> Option<String>,
{
    macro_rules! override_f32 {
        ($field:expr, $name:literal, $min:expr, $max:expr) => {
            if let Some(parsed) = parse_env_f32(read_var($name), $name, $min, $max, diagnostics) {
                $field = parsed;
            }
        };
    }

    override_f32!(
        interpolation.min_delay_ms,
        "CAVERN_NET_INTERP_MIN_DELAY_MS",
        5.0,
        500.0
    );
    override_f32!(
        interpolation.max_delay_ms,
        "CAVERN_NET_INTERP_MAX_DELAY_MS",
        5.0,
        800.0
    );
    override_f32!(
        interpolation.small_error_distance,
        "CAVERN_NET_INTERP_SMALL_ERROR",
        0.01,
        5.0
    );
    override_f32!(
        interpolation.medium_error_distance,
        "CAVERN_NET_INTERP_MEDIUM_ERROR",
        0.01,
        8.0
    );
    override_f32!(
        interpolation.large_error_distance,
        "CAVERN_NET_INTERP_LARGE_ERROR",
        0.01,
        16.0
    );
    override_f32!(
        interpolation.hard_snap_distance,
        "CAVERN_NET_INTERP_HARD_SNAP",
        0.01,
        64.0
    );
    if interpolation.medium_error_distance < interpolation.small_error_distance {
        diagnostics.push(
            "CAVERN_NET_INTERP_MEDIUM_ERROR < CAVERN_NET_INTERP_SMALL_ERROR; adjusting medium to small".to_string(),
        );
        interpolation.medium_error_distance = interpolation.small_error_distance;
    }
    if interpolation.large_error_distance < interpolation.medium_error_distance {
        diagnostics.push(
            "CAVERN_NET_INTERP_LARGE_ERROR < CAVERN_NET_INTERP_MEDIUM_ERROR; adjusting large to medium".to_string(),
        );
        interpolation.large_error_distance = interpolation.medium_error_distance;
    }
    if interpolation.hard_snap_distance < interpolation.large_error_distance {
        diagnostics.push(
            "CAVERN_NET_INTERP_HARD_SNAP < CAVERN_NET_INTERP_LARGE_ERROR; adjusting hard snap to large".to_string(),
        );
        interpolation.hard_snap_distance = interpolation.large_error_distance;
    }
}

fn parse_env_usize(
    value: Option<String>,
    key: &str,
    min: usize,
    max: usize,
    diagnostics: &mut Vec<String>,
) -> Option<usize> {
    let value = value?;
    let trimmed = value.trim();
    match trimmed.parse::<usize>() {
        Ok(parsed) => {
            if parsed < min || parsed > max {
                diagnostics.push(format!(
                    "{}={} is out of range [{}..={}], clamping",
                    key, parsed, min, max
                ));
            }
            Some(parsed.clamp(min, max))
        }
        Err(_) => {
            diagnostics.push(format!(
                "{}='{}' is not a valid integer, ignoring",
                key, trimmed
            ));
            None
        }
    }
}

fn parse_env_u64(
    value: Option<String>,
    key: &str,
    min: u64,
    max: u64,
    diagnostics: &mut Vec<String>,
) -> Option<u64> {
    let value = value?;
    let trimmed = value.trim();
    match trimmed.parse::<u64>() {
        Ok(parsed) => {
            if parsed < min || parsed > max {
                diagnostics.push(format!(
                    "{}={} is out of range [{}..={}], clamping",
                    key, parsed, min, max
                ));
            }
            Some(parsed.clamp(min, max))
        }
        Err(_) => {
            diagnostics.push(format!(
                "{}='{}' is not a valid integer, ignoring",
                key, trimmed
            ));
            None
        }
    }
}

#[allow(dead_code)]
fn parse_env_f32(
    value: Option<String>,
    key: &str,
    min: f32,
    max: f32,
    diagnostics: &mut Vec<String>,
) -> Option<f32> {
    let value = value?;
    let trimmed = value.trim();
    match trimmed.parse::<f32>() {
        Ok(parsed) => {
            if parsed < min || parsed > max {
                diagnostics.push(format!(
                    "{}={} is out of range [{:.3}..={:.3}], clamping",
                    key, parsed, min, max
                ));
            }
            Some(parsed.clamp(min, max))
        }
        Err(_) => {
            diagnostics.push(format!(
                "{}='{}' is not a valid float, ignoring",
                key, trimmed
            ));
            None
        }
    }
}
