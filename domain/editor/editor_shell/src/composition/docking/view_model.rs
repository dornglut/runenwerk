use ui_adaptive_composition::DockZone;
use ui_composition::{MountedUnitId, PresentationTargetId, RegionId};

use super::RegionCompassAccessibility;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionCompassSessionState {
    Idle,
    Armed,
    ActiveNoDestination,
    ActiveDestination,
    DetachFocused,
    CommitPending,
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionCompassTargetState {
    Candidate,
    Focused,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionCompassTargetViewModel {
    pub zone: DockZone,
    pub state: RegionCompassTargetState,
    pub short_label: &'static str,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RegionCompassViewModel {
    pub session: RegionCompassSessionState,
    pub target: PresentationTargetId,
    pub region: RegionId,
    pub unit: MountedUnitId,
    pub ordinal: usize,
    pub targets: Vec<RegionCompassTargetViewModel>,
    pub detach_label: String,
    pub accessibility: RegionCompassAccessibility,
}

impl RegionCompassViewModel {
    pub fn active(
        target: PresentationTargetId,
        region: RegionId,
        unit: MountedUnitId,
        focused: DockZone,
        unit_label: &str,
        destination_label: &str,
        accessibility: RegionCompassAccessibility,
    ) -> Self {
        Self::active_with_invalid_zones(
            target,
            region,
            unit,
            focused,
            unit_label,
            destination_label,
            accessibility,
            &[],
        )
    }

    pub const fn with_ordinal(mut self, ordinal: usize) -> Self {
        self.ordinal = ordinal;
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn active_with_invalid_zones(
        target: PresentationTargetId,
        region: RegionId,
        unit: MountedUnitId,
        focused: DockZone,
        unit_label: &str,
        destination_label: &str,
        accessibility: RegionCompassAccessibility,
        invalid_zones: &[DockZone],
    ) -> Self {
        let order = [
            DockZone::Center,
            DockZone::Left,
            DockZone::Right,
            DockZone::Top,
            DockZone::Bottom,
        ];
        let targets = order
            .into_iter()
            .map(|zone| RegionCompassTargetViewModel {
                zone,
                state: if invalid_zones.contains(&zone) {
                    RegionCompassTargetState::Invalid
                } else if zone == focused {
                    RegionCompassTargetState::Focused
                } else {
                    RegionCompassTargetState::Candidate
                },
                short_label: zone_short_label(zone),
                label: format!(
                    "Dock {unit_label} {} in {destination_label}",
                    zone_label(zone)
                ),
            })
            .collect();
        Self {
            session: RegionCompassSessionState::ActiveDestination,
            target,
            region,
            unit,
            ordinal: 0,
            targets,
            detach_label: format!("Move {unit_label} to new window"),
            accessibility,
        }
    }

    pub fn focus_zone(&mut self, focused: DockZone) -> bool {
        let selectable = self.targets.iter().any(|target| {
            target.zone == focused && target.state != RegionCompassTargetState::Invalid
        });
        if !selectable {
            return false;
        }
        self.session = RegionCompassSessionState::ActiveDestination;
        for target in &mut self.targets {
            if target.state != RegionCompassTargetState::Invalid {
                target.state = if target.zone == focused {
                    RegionCompassTargetState::Focused
                } else {
                    RegionCompassTargetState::Candidate
                };
            }
        }
        true
    }

    pub fn focus_detach(&mut self) {
        self.session = RegionCompassSessionState::DetachFocused;
        for target in &mut self.targets {
            if target.state == RegionCompassTargetState::Focused {
                target.state = RegionCompassTargetState::Candidate;
            }
        }
    }

    pub fn focused_zone(&self) -> Option<DockZone> {
        self.targets
            .iter()
            .find(|target| target.state == RegionCompassTargetState::Focused)
            .map(|target| target.zone)
    }

    pub fn is_accessible(&self) -> bool {
        self.accessibility.validate()
            && !self.detach_label.trim().is_empty()
            && self.targets.len() == 5
            && self
                .targets
                .iter()
                .all(|target| !target.label.trim().is_empty())
            && match self.session {
                RegionCompassSessionState::ActiveDestination => {
                    self.targets
                        .iter()
                        .filter(|target| target.state == RegionCompassTargetState::Focused)
                        .count()
                        == 1
                }
                RegionCompassSessionState::DetachFocused => self.focused_zone().is_none(),
                _ => true,
            }
    }
}

const fn zone_short_label(zone: DockZone) -> &'static str {
    match zone {
        DockZone::Center => "Tab",
        DockZone::Left => "Left",
        DockZone::Right => "Right",
        DockZone::Top => "Top",
        DockZone::Bottom => "Bottom",
    }
}

const fn zone_label(zone: DockZone) -> &'static str {
    match zone {
        DockZone::Center => "center",
        DockZone::Left => "left",
        DockZone::Right => "right",
        DockZone::Top => "top",
        DockZone::Bottom => "bottom",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_compass_has_one_focused_preview_and_five_stably_ordered_labels() {
        let model = RegionCompassViewModel::active(
            PresentationTargetId::new(1),
            RegionId::new(2),
            MountedUnitId::new(3),
            DockZone::Left,
            "Scene Viewport",
            "Inspector region",
            RegionCompassAccessibility::default(),
        );

        assert!(model.is_accessible());
        assert_eq!(model.targets[0].zone, DockZone::Center);
        assert_eq!(model.targets[1].state, RegionCompassTargetState::Focused);
        assert_eq!(model.accessibility.minimum_hit_size, 44.0);
    }
}
