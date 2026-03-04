use crate::domain::resources::CavernMetaProfile;
use anyhow::{Context, Result};
use engine::prelude::World;
use std::path::Path;

pub const META_PROFILE_PATH: &str = "var/cavern_hunt/meta_profile.json";

pub fn load_meta_profile(world: &mut World) -> Result<()> {
    let path = Path::new(META_PROFILE_PATH);
    let profile = match std::fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str::<CavernMetaProfile>(&raw)
            .with_context(|| format!("failed to parse {}", path.display()))?,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => CavernMetaProfile::default(),
        Err(err) => return Err(err).with_context(|| format!("failed to read {}", path.display())),
    };
    world.insert_resource(profile);
    Ok(())
}

pub fn save_meta_profile(profile: &CavernMetaProfile) -> Result<()> {
    let path = Path::new(META_PROFILE_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let raw = serde_json::to_string_pretty(profile).context("failed to serialize meta profile")?;
    std::fs::write(path, raw).with_context(|| format!("failed to write {}", path.display()))
}
