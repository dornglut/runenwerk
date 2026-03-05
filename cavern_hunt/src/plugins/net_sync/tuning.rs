#[cfg(test)]
use anyhow::Result;
#[cfg(test)]
use engine::prelude::World;

#[cfg(test)]
use crate::domain::{InterpolationConfig, ReplicationBudgetConfig, ReplicationCadenceConfig};

#[cfg(test)]
const ENV_NET_TUNING_PRESET: &str = "CAVERN_NET_TUNING_PRESET";

#[cfg(test)]
mod overrides;
#[cfg(test)]
mod preset;

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn configure_replication_tuning_from_env(world: &mut World) -> Result<()> {
    let mut budget = world
        .resource::<ReplicationBudgetConfig>()
        .copied()
        .unwrap_or_default();
    let mut cadence = world
        .resource::<ReplicationCadenceConfig>()
        .copied()
        .unwrap_or_default();
    let mut interpolation = world
        .resource::<InterpolationConfig>()
        .copied()
        .unwrap_or_default();
    let mut diagnostics = Vec::new();
    let preset = std::env::var(ENV_NET_TUNING_PRESET).ok();

    apply_replication_tuning_preset(
        &mut budget,
        &mut cadence,
        preset.as_deref(),
        &mut diagnostics,
    );
    apply_replication_tuning_overrides_from_reader(
        &mut budget,
        &mut cadence,
        |key| std::env::var(key).ok(),
        &mut diagnostics,
    );
    apply_interpolation_overrides_from_reader(
        &mut interpolation,
        |key| std::env::var(key).ok(),
        &mut diagnostics,
    );

    world.insert_resource(budget);
    world.insert_resource(cadence);
    world.insert_resource(interpolation);

    for diagnostic in diagnostics {
        tracing::warn!(diagnostic = %diagnostic, "cavern net tuning diagnostic");
    }
    tracing::info!(
        preset = preset.unwrap_or_else(|| "default".to_string()),
        ?budget,
        ?cadence,
        ?interpolation,
        "cavern net replication tuning ready"
    );
    Ok(())
}

#[cfg(test)]
pub(super) fn apply_replication_tuning_preset(
    budget: &mut ReplicationBudgetConfig,
    cadence: &mut ReplicationCadenceConfig,
    preset: Option<&str>,
    diagnostics: &mut Vec<String>,
) {
    preset::apply_replication_tuning_preset(budget, cadence, preset, diagnostics);
}

#[cfg(test)]
pub(super) fn apply_replication_tuning_overrides_from_reader<F>(
    budget: &mut ReplicationBudgetConfig,
    cadence: &mut ReplicationCadenceConfig,
    read_var: F,
    diagnostics: &mut Vec<String>,
) where
    F: Fn(&str) -> Option<String>,
{
    overrides::apply_replication_tuning_overrides_from_reader(
        budget,
        cadence,
        read_var,
        diagnostics,
    );
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn apply_interpolation_overrides_from_reader<F>(
    interpolation: &mut InterpolationConfig,
    read_var: F,
    diagnostics: &mut Vec<String>,
) where
    F: Fn(&str) -> Option<String>,
{
    overrides::apply_interpolation_overrides_from_reader(interpolation, read_var, diagnostics);
}
