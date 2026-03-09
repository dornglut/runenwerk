// Owner: Cavern Hunt Net Sync - Typed Run Event Codes
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(super) enum CavernRunEventCodeV2 {
    GeometryEdits,
    Keyframe,
    Patch,
    Chunk,
}

impl CavernRunEventCodeV2 {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::GeometryEdits => "cavern_hunt.geometry.edits.v2",
            Self::Keyframe => "cavern_hunt.keyframe.v2",
            Self::Patch => "cavern_hunt.patch.v2",
            Self::Chunk => "cavern_hunt.run_event.chunk.v2",
        }
    }

    pub(super) fn parse(value: &str) -> Option<Self> {
        match value {
            "cavern_hunt.geometry.edits.v2" => Some(Self::GeometryEdits),
            "cavern_hunt.keyframe.v2" => Some(Self::Keyframe),
            "cavern_hunt.patch.v2" => Some(Self::Patch),
            "cavern_hunt.run_event.chunk.v2" => Some(Self::Chunk),
            _ => None,
        }
    }

    pub(super) const fn event_type_tag(self) -> u64 {
        match self {
            Self::GeometryEdits => 1,
            Self::Keyframe => 2,
            Self::Patch => 3,
            Self::Chunk => 4,
        }
    }
}
