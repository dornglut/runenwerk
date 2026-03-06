// Owner: Cavern Hunt SDF Renderer - World Frame and Geometry Projection
pub(crate) fn setup_render_resources(world: &mut World) -> Result<()> {
    let mut frame_bindings = world
        .resource_mut::<RenderFrameResourceBindings>()
        .map_err(|_| anyhow!("RenderPlugin must be installed before Cavern Hunt client plugin"))?;
    if !frame_bindings.contains_resource::<CavernSdfWorldFrame>() {
        frame_bindings.register_resource::<CavernSdfWorldFrame>();
    }
    drop(frame_bindings);

    let spec = build_feature_graph_spec()?;
    world
        .resource_mut::<RenderGraphRegistryResource>()?
        .register_feature_graph(spec);

    let shared = Arc::new(Mutex::new(CavernGpuSharedState::default()));
    let mut executors = world.resource_mut::<RenderPassExecutorRegistryResource>()?;
    executors.register_custom(
        COMPUTE_EXECUTOR_ID,
        Arc::new(CavernComputeExecutor::new(Arc::clone(&shared))),
    );
    executors.register_custom(
        COMPOSE_EXECUTOR_ID,
        Arc::new(CavernComposeExecutor::new(shared)),
    );
    Ok(())
}

pub(crate) fn update_camera_and_hud_system(
    world: WorldRef,
    input: Res<InputState>,
    _time: Res<Time>,
    mut camera: ResMut<CavernCameraState>,
) -> Result<()> {
    let local_player_ref = world.resource::<LocalPlayerRef>()?;
    let local_player = local_player_ref.entity.and_then(|entity| {
        world.get::<Transform2>(entity).copied().map(|transform| {
            let health = world
                .get::<Health>(entity)
                .copied()
                .unwrap_or_else(|| Health::new(1.0));
            (transform, health)
        })
    });
    let Some((transform, _health)) = local_player else {
        return Ok(());
    };

    let zoom_delta = input.scroll_delta * 1.5;
    if zoom_delta.abs() > f32::EPSILON {
        camera.distance =
            (camera.distance - zoom_delta).clamp(camera.distance_min, camera.distance_max);
    }

    camera.target = [transform.x, 1.55, transform.y];

    Ok(())
}

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

    for (entity, transform) in world.query::<(Entity, &Transform2)>().iter() {
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
                radius: radius,
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
                    crate::domain::PickupKind::Scrap(_) => 7,
                    crate::domain::PickupKind::WeaponMod(_) => 8,
                    crate::domain::PickupKind::Relic(_) => 9,
                    crate::domain::PickupKind::HealingCharge(_) => 10,
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
                    .get::<crate::domain::Faction>(entity)
                    .map(|faction| {
                        if *faction == crate::domain::Faction::Hunters {
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
            .map(|header| crate::domain::CavernSdfMaterialProgramHeader {
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

const SHAPE_SPHERE: u32 = 0;
const SHAPE_ELLIPSOID: u32 = 1;
const SHAPE_CAPSULE: u32 = 2;
const SHAPE_BOX: u32 = 3;
const SHAPE_ROUNDED_BOX: u32 = 4;
const SHAPE_CYLINDER: u32 = 5;

const OP_ADD_SOLID: u32 = 0;
const OP_SUBTRACT_VOID: u32 = 1;
const OP_MASK_WALKABLE: u32 = 2;
const OP_BLOCKER: u32 = 3;
const OP_HAZARD: u32 = 4;

fn material_class_from_geometry(material: GeometryMaterial) -> u32 {
    match material {
        GeometryMaterial::Rock | GeometryMaterial::CavernVoid => crate::domain::MATERIAL_CLASS_ROCK,
        GeometryMaterial::Barrier => crate::domain::MATERIAL_CLASS_BARRIER,
        GeometryMaterial::Hazard => crate::domain::MATERIAL_CLASS_HAZARD,
        GeometryMaterial::Marker => crate::domain::MATERIAL_CLASS_MARKER,
    }
}

fn op_kind(op: GeometryOp) -> u32 {
    match op {
        GeometryOp::AddSolid => OP_ADD_SOLID,
        GeometryOp::SubtractVoid => OP_SUBTRACT_VOID,
        GeometryOp::MaskWalkable => OP_MASK_WALKABLE,
        GeometryOp::Blocker => OP_BLOCKER,
        GeometryOp::HazardVolume => OP_HAZARD,
    }
}

fn geometry_primitives_from_graph(graph: &CavernGeometryGraph) -> Vec<CavernSdfGeometryPrimitive> {
    let mut out = Vec::with_capacity(graph.primitives.len());
    for primitive in graph
        .primitives
        .iter()
        .filter(|primitive| primitive.enabled)
    {
        append_shape_primitive(
            &mut out,
            primitive.op,
            &primitive.shape,
            material_class_from_geometry(primitive.material),
            primitive.id.0 as u32,
        );
    }
    out
}

fn geometry_primitives_from_topology(topology: &CavernTopology) -> Vec<CavernSdfGeometryPrimitive> {
    let mut out = Vec::with_capacity(topology.rooms.len() + topology.connections.len() + 1);
    let center = [
        (topology.world_bounds.min[0] + topology.world_bounds.max[0]) * 0.5,
        (topology.world_bounds.min[1] + topology.world_bounds.max[1]) * 0.5,
        (topology.world_bounds.min[2] + topology.world_bounds.max[2]) * 0.5,
    ];
    let half_extents = [
        (topology.world_bounds.max[0] - topology.world_bounds.min[0]) * 0.5,
        (topology.world_bounds.max[1] - topology.world_bounds.min[1]) * 0.5,
        (topology.world_bounds.max[2] - topology.world_bounds.min[2]) * 0.5,
    ];
    out.push(CavernSdfGeometryPrimitive {
        shape_kind: SHAPE_BOX,
        op_kind: OP_ADD_SOLID,
        material_class: crate::domain::MATERIAL_CLASS_ROCK,
        material_instance: 0,
        p0: [center[0], center[1], center[2], 0.0],
        p1: [half_extents[0], half_extents[1], half_extents[2], 0.0],
        p2: [0.0; 4],
    });
    for room in &topology.rooms {
        out.push(CavernSdfGeometryPrimitive {
            shape_kind: SHAPE_CYLINDER,
            op_kind: OP_SUBTRACT_VOID,
            material_class: crate::domain::MATERIAL_CLASS_ROCK,
            material_instance: room.id.0 as u32,
            p0: [
                room.center[0],
                room.center[1],
                room.center[2],
                room.radii[0].max(room.radii[2]),
            ],
            p1: [room.radii[1], 0.0, 0.0, 0.0],
            p2: [0.0; 4],
        });
    }
    for connection in &topology.connections {
        out.push(CavernSdfGeometryPrimitive {
            shape_kind: SHAPE_CAPSULE,
            op_kind: OP_SUBTRACT_VOID,
            material_class: crate::domain::MATERIAL_CLASS_ROCK,
            material_instance: 0,
            p0: [
                connection.start[0],
                connection.start[1],
                connection.start[2],
                connection.radius,
            ],
            p1: [connection.end[0], connection.end[1], connection.end[2], 0.0],
            p2: [0.0; 4],
        });
    }
    out
}

fn geometry_primitives_from_layout(layout: &CavernLayout) -> Vec<CavernSdfGeometryPrimitive> {
    let mut out = Vec::with_capacity(layout.rooms.len() + layout.connections.len() + 1);
    let center = [
        (layout.world_bounds[0] + layout.world_bounds[2]) * 0.5,
        2.2,
        (layout.world_bounds[1] + layout.world_bounds[3]) * 0.5,
    ];
    let half_extents = [
        (layout.world_bounds[2] - layout.world_bounds[0]) * 0.5,
        5.8,
        (layout.world_bounds[3] - layout.world_bounds[1]) * 0.5,
    ];
    out.push(CavernSdfGeometryPrimitive {
        shape_kind: SHAPE_BOX,
        op_kind: OP_ADD_SOLID,
        material_class: crate::domain::MATERIAL_CLASS_ROCK,
        material_instance: 0,
        p0: [center[0], center[1], center[2], 0.0],
        p1: [half_extents[0], half_extents[1], half_extents[2], 0.0],
        p2: [0.0; 4],
    });
    for room in &layout.rooms {
        out.push(CavernSdfGeometryPrimitive {
            shape_kind: SHAPE_CYLINDER,
            op_kind: OP_SUBTRACT_VOID,
            material_class: crate::domain::MATERIAL_CLASS_ROCK,
            material_instance: room.id.0 as u32,
            p0: [
                room.center[0],
                2.4,
                room.center[1],
                room.radii[0].max(room.radii[1]),
            ],
            p1: [2.2, 0.0, 0.0, 0.0],
            p2: [0.0; 4],
        });
    }
    for tunnel in &layout.connections {
        out.push(CavernSdfGeometryPrimitive {
            shape_kind: SHAPE_CAPSULE,
            op_kind: OP_SUBTRACT_VOID,
            material_class: crate::domain::MATERIAL_CLASS_ROCK,
            material_instance: 0,
            p0: [tunnel.start[0], 2.2, tunnel.start[1], tunnel.radius],
            p1: [tunnel.end[0], 2.2, tunnel.end[1], 0.0],
            p2: [0.0; 4],
        });
    }
    out
}

fn append_shape_primitive(
    out: &mut Vec<CavernSdfGeometryPrimitive>,
    op: GeometryOp,
    shape: &GeometryPrimitiveShape3,
    material_class: u32,
    material_instance: u32,
) {
    let op_kind = op_kind(op);
    match shape {
        GeometryPrimitiveShape3::Sphere { center, radius } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_SPHERE,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], *radius],
                p1: [0.0; 4],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::Ellipsoid { center, radii } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_ELLIPSOID,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], 0.0],
                p1: [radii[0], radii[1], radii[2], 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::Capsule { start, end, radius } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_CAPSULE,
                op_kind,
                material_class,
                material_instance,
                p0: [start[0], start[1], start[2], *radius],
                p1: [end[0], end[1], end[2], 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::Box {
            center,
            half_extents,
        } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_BOX,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], 0.0],
                p1: [half_extents[0], half_extents[1], half_extents[2], 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::RoundedBox {
            center,
            half_extents,
            radius,
        } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_ROUNDED_BOX,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], *radius],
                p1: [half_extents[0], half_extents[1], half_extents[2], 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::Cylinder {
            center,
            radius,
            half_height,
        } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_CYLINDER,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], *radius],
                p1: [*half_height, 0.0, 0.0, 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::TunnelSplineCapsuleChain { points, radius } => {
            for segment in points.windows(2) {
                out.push(CavernSdfGeometryPrimitive {
                    shape_kind: SHAPE_CAPSULE,
                    op_kind,
                    material_class,
                    material_instance,
                    p0: [segment[0][0], segment[0][1], segment[0][2], *radius],
                    p1: [segment[1][0], segment[1][1], segment[1][2], 0.0],
                    p2: [0.0; 4],
                });
            }
        }
    }
}
