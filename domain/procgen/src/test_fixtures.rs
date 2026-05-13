use graph::{
    CyclePolicy, EdgeDefinition, EdgeId, GraphDefinition, GraphId, NodeDefinition, NodeId,
    PortDefinition, PortDirection, PortId, PortTypeId,
};
use product::ProductIdentity;
use spatial::{ChunkCoord3, ChunkId, RegionCoord3, RegionId, WorldId};
use world_ops::{QuantizedAabb, QuantizedVec3, WorldRevision};

use crate::{
    ProcgenDocument, ProcgenDocumentId, ProcgenGeneratorId, ProcgenInputProduct,
    ProcgenLoweringPolicy, ProcgenNodeKind, ProcgenNodeParameters, ProcgenOutputKind,
    ProcgenOutputProduct, ProcgenReservation, ProcgenReservationId, ProcgenScope,
    ProcgenWriteTarget,
    catalog::{
        FIELD_PRODUCT_OUTPUT_NODE, HEIGHT_NOISE_NODE, MATERIAL_RULE_NODE, WORLD_OPS_OUTPUT_NODE,
    },
};

pub(crate) fn bounds() -> QuantizedAabb {
    QuantizedAabb {
        min: QuantizedVec3 { x: 0, y: 0, z: 0 },
        max: QuantizedVec3 {
            x: 16,
            y: 16,
            z: 16,
        },
    }
}

pub(crate) fn valid_graph() -> GraphDefinition {
    let scalar = PortTypeId::new(1);
    GraphDefinition::new(
        GraphId::new(7),
        "terrain_material",
        CyclePolicy::RejectDirectedCycles,
        [
            NodeDefinition::new(
                NodeId::new(1),
                HEIGHT_NOISE_NODE,
                [PortDefinition::new(
                    PortId::new(1),
                    "height",
                    PortDirection::Output,
                    scalar,
                )],
            ),
            NodeDefinition::new(
                NodeId::new(2),
                MATERIAL_RULE_NODE,
                [
                    PortDefinition::new(PortId::new(2), "height", PortDirection::Input, scalar),
                    PortDefinition::new(PortId::new(3), "material", PortDirection::Output, scalar),
                ],
            ),
            NodeDefinition::new(
                NodeId::new(3),
                WORLD_OPS_OUTPUT_NODE,
                [PortDefinition::new(
                    PortId::new(4),
                    "material",
                    PortDirection::Input,
                    scalar,
                )],
            ),
            NodeDefinition::new(
                NodeId::new(4),
                FIELD_PRODUCT_OUTPUT_NODE,
                [PortDefinition::new(
                    PortId::new(5),
                    "material",
                    PortDirection::Input,
                    scalar,
                )],
            ),
        ],
        [
            EdgeDefinition::new(EdgeId::new(1), PortId::new(1), PortId::new(2)),
            EdgeDefinition::new(EdgeId::new(2), PortId::new(3), PortId::new(4)),
            EdgeDefinition::new(EdgeId::new(3), PortId::new(3), PortId::new(5)),
        ],
    )
}

pub(crate) fn valid_document() -> ProcgenDocument {
    let world_id = WorldId(1);
    let density_target = ProcgenWriteTarget::density("density-main", bounds());
    let material_target = ProcgenWriteTarget::material_channel("material-main", bounds(), 2);
    let mut document = ProcgenDocument::new(
        ProcgenDocumentId::new(101),
        "first terrain material",
        valid_graph(),
        ProcgenScope::new(
            world_id,
            [ChunkId::new(world_id, ChunkCoord3 { x: 0, y: 0, z: 0 })],
            [RegionId::new(world_id, RegionCoord3 { x: 0, y: 0, z: 0 })],
        ),
    )
    .with_schema_version("procgen.schema.v1")
    .with_generator(ProcgenGeneratorId::new(33), "terrain-material.v1")
    .with_world_seed("world-seed:alpha")
    .with_source_revision("source-rev-1")
    .with_authored_overlay_generation(9)
    .with_lowering_policy(ProcgenLoweringPolicy::new(
        "lowering.v1",
        16,
        WorldRevision(4),
    ))
    .with_input_product(ProcgenInputProduct::new(ProductIdentity::new(77), 12))
    .with_node_parameter(
        ProcgenNodeParameters::new(NodeId::new(1), ProcgenNodeKind::HeightNoise, "height")
            .with_seed_salt("height-a")
            .with_weight(5),
    )
    .with_node_parameter(
        ProcgenNodeParameters::new(NodeId::new(2), ProcgenNodeKind::MaterialRule, "material")
            .with_seed_salt("material-a")
            .with_material_channel(2),
    )
    .with_node_parameter(ProcgenNodeParameters::new(
        NodeId::new(3),
        ProcgenNodeKind::WorldOpsOutput,
        "world ops",
    ))
    .with_node_parameter(ProcgenNodeParameters::new(
        NodeId::new(4),
        ProcgenNodeKind::FieldProductOutput,
        "field product",
    ))
    .with_write_target(density_target.clone())
    .with_write_target(material_target.clone())
    .with_output_product(ProcgenOutputProduct::new(
        ProductIdentity::new(8001),
        ProcgenOutputKind::WorldOpsWindow,
        "operation window",
    ))
    .with_output_product(ProcgenOutputProduct::new(
        ProductIdentity::new(8002),
        ProcgenOutputKind::FieldProductCandidate,
        "field product candidate",
    ))
    .with_reservation(ProcgenReservation::from_target(
        ProcgenReservationId::new(7001),
        &density_target,
    ))
    .with_reservation(ProcgenReservation::from_target(
        ProcgenReservationId::new(7002),
        &material_target,
    ));
    document.refresh_cache_lineage();
    document
}
