//! Built-in static Draw composition definition.

use ui_composition::{
    CapabilityId, CompositionDefinitionId, CompositionDefinitionV1, CompositionRootDefinition,
    CompositionRootId, ContentInstanceRef, ContentOwnerId, ContentProfileId, DefinitionRevision,
    MountedContentRef, MountedUnitDefinition, MountedUnitId, PresentationTargetDefinition,
    PresentationTargetId, RegionDefinition, RegionId, RegionKind, RegionProfileId, SplitAxis,
    SplitFraction, TargetProfileId, UnavailableContentPolicy,
};

use super::{
    DrawingCompositionDiagnosticCode as Code, DrawingCompositionDiagnosticRecord as Record,
    DrawingCompositionDiagnosticStage as Stage, DrawingCompositionDiagnosticSubject as Subject,
    DrawingCompositionRejection,
};

pub const DRAWING_COMPOSITION_ID: CompositionDefinitionId = CompositionDefinitionId::new(1);
pub const DRAWING_PRESENTATION_TARGET_ID: PresentationTargetId = PresentationTargetId::new(1);
pub const DRAWING_COMPOSITION_ROOT_ID: CompositionRootId = CompositionRootId::new(1);

pub const DRAWING_ROOT_REGION_ID: RegionId = RegionId::new(1);
pub const DRAWING_TOP_BAR_REGION_ID: RegionId = RegionId::new(2);
pub const DRAWING_BODY_REGION_ID: RegionId = RegionId::new(3);
pub const DRAWING_TOOL_RAIL_REGION_ID: RegionId = RegionId::new(4);
pub const DRAWING_MAIN_REGION_ID: RegionId = RegionId::new(5);
pub const DRAWING_CANVAS_REGION_ID: RegionId = RegionId::new(6);
pub const DRAWING_SUPPORT_REGION_ID: RegionId = RegionId::new(7);

pub const DRAWING_TOP_BAR_UNIT_ID: MountedUnitId = MountedUnitId::new(1);
pub const DRAWING_TOOL_RAIL_UNIT_ID: MountedUnitId = MountedUnitId::new(2);
pub const DRAWING_CANVAS_UNIT_ID: MountedUnitId = MountedUnitId::new(3);
pub const DRAWING_SUPPORT_UNIT_ID: MountedUnitId = MountedUnitId::new(4);

pub const DRAWING_APP_PROFILE: &str = "runenwerk.draw";
pub const DRAWING_EXTENSION_PROFILE: &str = "runenwerk.draw.layout";
pub const DRAWING_TARGET_PROFILE: &str = "runenwerk.draw.wide";

pub fn builtin_drawing_composition_definition()
-> Result<CompositionDefinitionV1, DrawingCompositionRejection> {
    let target_profile = reference(TargetProfileId::new(DRAWING_TARGET_PROFILE))?;
    let top_fraction = split_fraction(500)?;
    let tool_fraction = split_fraction(438)?;
    let canvas_fraction = split_fraction(7_712)?;

    Ok(CompositionDefinitionV1::new(
        DRAWING_COMPOSITION_ID,
        DefinitionRevision::new(1),
        vec![PresentationTargetDefinition::new(
            DRAWING_PRESENTATION_TARGET_ID,
            target_profile,
        )],
        vec![CompositionRootDefinition::new(
            DRAWING_COMPOSITION_ROOT_ID,
            DRAWING_PRESENTATION_TARGET_ID,
            DRAWING_ROOT_REGION_ID,
            true,
        )],
        vec![
            region(
                DRAWING_ROOT_REGION_ID,
                "runenwerk.draw.root",
                RegionKind::Split {
                    axis: SplitAxis::Vertical,
                    fraction: top_fraction,
                    first: DRAWING_TOP_BAR_REGION_ID,
                    second: DRAWING_BODY_REGION_ID,
                },
            )?,
            region(
                DRAWING_TOP_BAR_REGION_ID,
                "runenwerk.draw.top_bar_region",
                RegionKind::MountPoint {
                    mounted_unit: DRAWING_TOP_BAR_UNIT_ID,
                },
            )?,
            region(
                DRAWING_BODY_REGION_ID,
                "runenwerk.draw.body",
                RegionKind::Split {
                    axis: SplitAxis::Horizontal,
                    fraction: tool_fraction,
                    first: DRAWING_TOOL_RAIL_REGION_ID,
                    second: DRAWING_MAIN_REGION_ID,
                },
            )?,
            region(
                DRAWING_TOOL_RAIL_REGION_ID,
                "runenwerk.draw.tool_rail_region",
                RegionKind::MountPoint {
                    mounted_unit: DRAWING_TOOL_RAIL_UNIT_ID,
                },
            )?,
            region(
                DRAWING_MAIN_REGION_ID,
                "runenwerk.draw.main",
                RegionKind::Split {
                    axis: SplitAxis::Horizontal,
                    fraction: canvas_fraction,
                    first: DRAWING_CANVAS_REGION_ID,
                    second: DRAWING_SUPPORT_REGION_ID,
                },
            )?,
            region(
                DRAWING_CANVAS_REGION_ID,
                "runenwerk.draw.canvas_region",
                RegionKind::MountPoint {
                    mounted_unit: DRAWING_CANVAS_UNIT_ID,
                },
            )?,
            region(
                DRAWING_SUPPORT_REGION_ID,
                "runenwerk.draw.support_region",
                RegionKind::MountPoint {
                    mounted_unit: DRAWING_SUPPORT_UNIT_ID,
                },
            )?,
        ],
        vec![
            mounted_unit(
                DRAWING_TOP_BAR_UNIT_ID,
                "runenwerk.draw.top_bar",
                "runenwerk.draw.primary.top_bar",
            )?,
            mounted_unit(
                DRAWING_TOOL_RAIL_UNIT_ID,
                "runenwerk.draw.tool_rail",
                "runenwerk.draw.primary.tool_rail",
            )?,
            mounted_unit(
                DRAWING_CANVAS_UNIT_ID,
                "runenwerk.draw.canvas",
                "runenwerk.draw.primary.canvas",
            )?,
            mounted_unit(
                DRAWING_SUPPORT_UNIT_ID,
                "runenwerk.draw.support_panel",
                "runenwerk.draw.primary.support_panel",
            )?,
        ],
    ))
}

fn region(
    id: RegionId,
    profile: &str,
    kind: RegionKind,
) -> Result<RegionDefinition, DrawingCompositionRejection> {
    Ok(RegionDefinition::new(
        id,
        Some(reference(RegionProfileId::new(profile))?),
        kind,
    ))
}

fn mounted_unit(
    id: MountedUnitId,
    profile: &str,
    instance: &str,
) -> Result<MountedUnitDefinition, DrawingCompositionRejection> {
    Ok(MountedUnitDefinition::new(
        id,
        MountedContentRef::new(
            reference(ContentOwnerId::new(DRAWING_APP_PROFILE))?,
            reference(ContentProfileId::new(profile))?,
            reference(ContentInstanceRef::new(instance))?,
        ),
        [reference(CapabilityId::new(
            "runenwerk.draw.content_present",
        ))?],
        UnavailableContentPolicy::ShowFallback,
    ))
}

fn split_fraction(value: u16) -> Result<SplitFraction, DrawingCompositionRejection> {
    SplitFraction::try_new(value).map_err(|error| {
        invalid_definition(format!(
            "Use a valid compiled-in Draw split fraction: {error}."
        ))
    })
}

fn reference<T, E: std::fmt::Display>(
    result: Result<T, E>,
) -> Result<T, DrawingCompositionRejection> {
    result.map_err(|error| {
        invalid_definition(format!(
            "Use valid compiled-in namespaced Draw composition references: {error}."
        ))
    })
}

fn invalid_definition(message: String) -> DrawingCompositionRejection {
    DrawingCompositionRejection::single(Record::error(
        Code::DefinitionInvalid,
        Stage::Definition,
        Subject::Layout(DRAWING_COMPOSITION_ID),
        message,
    ))
}
