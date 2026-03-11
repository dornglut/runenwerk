use super::geometry_projection::{
    geometry_primitives_from_graph, geometry_primitives_from_layout,
    geometry_primitives_from_topology,
};
use super::*;

// Owner: Cavern Hunt SDF Renderer - World Frame Assembly
pub(crate) fn build_sdf_world_frame_system(
    world: WorldRef,
    layout: Res<CavernLayout>,
    camera: Res<CavernCameraState>,
    mut frame: ResMut<CavernSdfWorldFrame>,
) -> Result<()> {
    let (world_bounds, geometry_primitives) =
        if let Ok(graph) = world.resource::<CavernGeometryGraph>() {
            (
                [
                    graph.bounds.min[0],
                    graph.bounds.min[2],
                    graph.bounds.max[0],
                    graph.bounds.max[2],
                ],
                geometry_primitives_from_graph(&graph),
            )
        } else if let Ok(topology) = world.resource::<CavernTopology>() {
            (
                [
                    topology.world_bounds.min[0],
                    topology.world_bounds.min[2],
                    topology.world_bounds.max[0],
                    topology.world_bounds.max[2],
                ],
                geometry_primitives_from_topology(&topology),
            )
        } else {
            (
                layout.world_bounds,
                geometry_primitives_from_layout(&layout),
            )
        };

    frame.world_bounds = world_bounds;
    frame.camera = camera.clone();
    frame.material_program_headers.clear();
    frame.material_ops.clear();
    frame.material_constants.clear();
    frame.agents.clear();
    frame.geometry_primitives = geometry_primitives;

    if let Ok(quality) = world.resource::<CavernMaterialQualityConfig>() {
        frame.render_mode = quality.render_mode.as_gpu_u32();
        frame.gi_mode = quality.gi.mode.as_gpu_u32();
        frame.gi_quality = quality.gi.quality.as_gpu_u32();
        frame.gi_sample_budget = quality.gi.sample_budget.max(1);
    }

    let query = world.query_state::<(Entity, &Transform2), ()>();
    for (entity, transform) in query.iter(&*world) {
        if is_active_player_entity(&world, entity) {
            let health = world
                .get::<Health>(entity)
                .copied()
                .unwrap_or_else(|| Health::new(1.0));
            let radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(0.45))
                .0;
            let player_palette_slot = world
                .get::<PlayerId>(entity)
                .map(|player_id| player_id.0.saturating_sub(1) % 8)
                .unwrap_or(0);
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius,
                health_ratio: health.ratio(),
                team: player_palette_slot,
                kind: if world.get::<PlayerSpectator>(entity).is_some() {
                    13
                } else if world.get::<PlayerCompanion>(entity).is_some() {
                    12
                } else {
                    0
                },
            });
            continue;
        }

        if let Some(kind) = world.get::<EnemyKind>(entity).copied() {
            let health = world
                .get::<Health>(entity)
                .copied()
                .unwrap_or_else(|| Health::new(1.0));
            let radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(match kind {
                    EnemyKind::Swarmer => ColliderRadius(0.42),
                    EnemyKind::Bruiser => ColliderRadius(0.78),
                    EnemyKind::Spitter => ColliderRadius(0.58),
                    EnemyKind::NestGuardian => ColliderRadius(0.92),
                })
                .0;
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius,
                health_ratio: health.ratio(),
                team: 1,
                kind: match kind {
                    EnemyKind::Swarmer => 1,
                    EnemyKind::Bruiser => 2,
                    EnemyKind::Spitter => 3,
                    EnemyKind::NestGuardian => 4,
                },
            });
            continue;
        }

        if let Some(pickup) = world.get::<Pickup>(entity).copied() {
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius: if world.get::<LootDrop>(entity).is_some() {
                    0.34
                } else {
                    0.48
                },
                health_ratio: 1.0,
                team: 2,
                kind: match pickup.kind {
                    crate::PickupKind::Scrap(_) => 7,
                    crate::PickupKind::WeaponMod(_) => 8,
                    crate::PickupKind::Relic(_) => 9,
                    crate::PickupKind::HealingCharge(_) => 10,
                },
            });
            continue;
        }

        if world.get::<Chest>(entity).is_some() {
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius: 0.55,
                health_ratio: 1.0,
                team: 2,
                kind: 5,
            });
            continue;
        }

        if world.get::<Projectile>(entity).is_some() {
            let team = if world.get::<Player>(entity).is_some() {
                0
            } else {
                world
                    .get::<crate::Faction>(entity)
                    .map(|faction| {
                        if *faction == crate::Faction::Hunters {
                            0
                        } else {
                            1
                        }
                    })
                    .unwrap_or(1)
            };
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius: if let Some(visual) = world.get::<ProjectileVisualState>(entity) {
                    (0.16 + visual.life_elapsed_seconds.min(0.12) * 0.3).max(0.16)
                } else {
                    0.16
                },
                health_ratio: 1.0,
                team,
                kind: 11,
            });
            continue;
        }

        if world.get::<ExtractionZone>(entity).is_some() {
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius: 1.15,
                health_ratio: 1.0,
                team: 3,
                kind: 6,
            });
        }
    }

    if let Ok(runtime) = world.resource::<CavernMaterialRuntimeState>() {
        let payload = runtime.build_gpu_payload(
            MAX_MATERIAL_PROGRAMS,
            MAX_MATERIAL_OPS,
            MAX_MATERIAL_CONSTANTS,
        );
        frame.material_program_headers = payload
            .headers
            .iter()
            .map(|header| crate::CavernSdfMaterialProgramHeader {
                class_id: header.class_id,
                op_offset: header.op_offset,
                op_count: header.op_count,
                const_offset: header.const_offset,
                const_count: header.const_count,
                base_color_slot: header.base_color_slot,
                roughness_slot: header.roughness_slot,
                metallic_slot: header.metallic_slot,
                normal_perturb_slot: header.normal_perturb_slot,
                ao_slot: header.ao_slot,
                emissive_slot: header.emissive_slot,
            })
            .collect();
        frame.material_ops = payload
            .ops
            .iter()
            .map(|op| CavernSdfMaterialOp {
                op: op.op,
                dst: op.dst,
                src_a: op.src_a,
                src_b: op.src_b,
                src_c: op.src_c,
                const_idx: op.const_idx,
                flags: op.flags,
            })
            .collect();
        frame.material_constants = payload.constants;
    }

    Ok(())
}
