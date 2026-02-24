use anyhow::{Result, bail};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PipelineKey {
    WorldComputeBasic,
    WorldComputeHighContrast,
    WorldComputeSdf3d,
    WorldComposeFullscreen,
    UiCompositeSdf,
}

impl PipelineKey {
    pub fn label(self) -> &'static str {
        match self {
            Self::WorldComputeBasic => "world_compute_basic",
            Self::WorldComputeHighContrast => "world_compute_high_contrast",
            Self::WorldComputeSdf3d => "world_compute_sdf_3d",
            Self::WorldComposeFullscreen => "world_compose_fullscreen",
            Self::UiCompositeSdf => "ui_composite_sdf",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PassSlot {
    WorldCompute,
    WorldCompose,
    UiComposite,
}

impl PassSlot {
    pub fn label(self) -> &'static str {
        match self {
            Self::WorldCompute => "world_compute",
            Self::WorldCompose => "world_compose",
            Self::UiComposite => "ui_composite",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PipelineSelection {
    pub world_compute: PipelineKey,
    pub world_compose: PipelineKey,
    pub ui_composite: PipelineKey,
}

impl Default for PipelineSelection {
    fn default() -> Self {
        Self {
            world_compute: PipelineKey::WorldComputeBasic,
            world_compose: PipelineKey::WorldComposeFullscreen,
            ui_composite: PipelineKey::UiCompositeSdf,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PipelineRegistry {
    selection: PipelineSelection,
}

impl PipelineRegistry {
    pub fn selection(&self) -> PipelineSelection {
        self.selection
    }

    pub fn key_for(&self, slot: PassSlot) -> PipelineKey {
        match slot {
            PassSlot::WorldCompute => self.selection.world_compute,
            PassSlot::WorldCompose => self.selection.world_compose,
            PassSlot::UiComposite => self.selection.ui_composite,
        }
    }

    pub fn set_pipeline(&mut self, slot: PassSlot, key: PipelineKey) -> Result<()> {
        if !Self::supports(slot, key) {
            bail!(
                "pipeline '{}' is not compatible with pass slot '{}'",
                key.label(),
                slot.label()
            );
        }
        match slot {
            PassSlot::WorldCompute => self.selection.world_compute = key,
            PassSlot::WorldCompose => self.selection.world_compose = key,
            PassSlot::UiComposite => self.selection.ui_composite = key,
        }
        Ok(())
    }

    pub fn supports(slot: PassSlot, key: PipelineKey) -> bool {
        matches!(
            (slot, key),
            (PassSlot::WorldCompute, PipelineKey::WorldComputeBasic)
                | (
                    PassSlot::WorldCompute,
                    PipelineKey::WorldComputeHighContrast
                )
                | (PassSlot::WorldCompute, PipelineKey::WorldComputeSdf3d)
                | (PassSlot::WorldCompose, PipelineKey::WorldComposeFullscreen)
                | (PassSlot::UiComposite, PipelineKey::UiCompositeSdf)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{PassSlot, PipelineKey, PipelineRegistry};

    #[test]
    fn defaults_match_expected_slots() {
        let registry = PipelineRegistry::default();
        assert_eq!(
            registry.key_for(PassSlot::WorldCompute),
            PipelineKey::WorldComputeBasic
        );
        assert_eq!(
            registry.key_for(PassSlot::WorldCompose),
            PipelineKey::WorldComposeFullscreen
        );
        assert_eq!(
            registry.key_for(PassSlot::UiComposite),
            PipelineKey::UiCompositeSdf
        );
    }

    #[test]
    fn reject_incompatible_pipeline() {
        let mut registry = PipelineRegistry::default();
        let err = registry
            .set_pipeline(PassSlot::UiComposite, PipelineKey::WorldComputeBasic)
            .expect_err("invalid pipeline assignment should fail");
        assert!(err.to_string().contains("not compatible"));
    }

    #[test]
    fn world_compute_slot_accepts_multiple_variants() {
        let mut registry = PipelineRegistry::default();
        registry
            .set_pipeline(
                PassSlot::WorldCompute,
                PipelineKey::WorldComputeHighContrast,
            )
            .expect("high contrast compute pipeline should be accepted");
        assert_eq!(
            registry.key_for(PassSlot::WorldCompute),
            PipelineKey::WorldComputeHighContrast
        );
    }
}
