use runen_spatial::{ChunkId, GridPartitionConfig, WorldId, WorldPosition};
use world_sdf::{
    CollisionHit, CollisionQueryService, CollisionSweepOutcome, SdfChunkStore, SphereSweep,
};

const MAX_FIXED_STEP_SECONDS: f32 = 1.0;
const MAX_CHARACTER_RADIUS: f32 = 1024.0;
const MAX_GRAVITY_ACCELERATION: f32 = 10_000.0;
const MAX_SUPPORT_PROBE_DISTANCE: f32 = 1024.0;
const MAX_LINEAR_SPEED: f32 = 10_000.0;
const MAX_SWEEP_CHUNKS: u128 = 4096;
const MAX_SLIDE_ITERATIONS: u8 = 16;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CharacterPhysicalState {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub grounded: bool,
    pub support_normal: Option<[f32; 3]>,
}

impl Default for CharacterPhysicalState {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            grounded: false,
            support_normal: None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CharacterMotionConfig {
    pub radius: f32,
    pub gravity_acceleration: f32,
    pub up: [f32; 3],
    pub max_walkable_slope_radians: f32,
    pub contact_skin: f32,
    pub support_probe_distance: f32,
    pub max_slide_iterations: u8,
}

impl Default for CharacterMotionConfig {
    fn default() -> Self {
        Self {
            radius: 0.25,
            gravity_acceleration: 9.81,
            up: [0.0, 1.0, 0.0],
            max_walkable_slope_radians: 50.0_f32.to_radians(),
            contact_skin: 0.01,
            support_probe_distance: 0.08,
            max_slide_iterations: 4,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CharacterMotionInput {
    pub state: CharacterPhysicalState,
    pub desired_planar_velocity: [f32; 3],
    pub fixed_step_seconds: f32,
    pub config: CharacterMotionConfig,
    pub world_id: WorldId,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CharacterContact {
    pub hit_position: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterMotionDiagnostics {
    pub requested_displacement: [f32; 3],
    pub admitted_displacement: [f32; 3],
    pub contacts: Vec<CharacterContact>,
    pub slide_iterations: u8,
    pub grounded: bool,
    pub support_normal: Option<[f32; 3]>,
    pub rejected_steep_support: bool,
    pub iteration_limit_reached: bool,
}

impl CharacterMotionDiagnostics {
    fn new(requested_displacement: [f32; 3]) -> Self {
        Self {
            requested_displacement,
            admitted_displacement: [0.0; 3],
            contacts: Vec::new(),
            slide_iterations: 0,
            grounded: false,
            support_normal: None,
            rejected_steep_support: false,
            iteration_limit_reached: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CharacterMotionOutcome {
    Completed {
        state: CharacterPhysicalState,
        diagnostics: CharacterMotionDiagnostics,
    },
    MissingPayload {
        chunk_id: ChunkId,
        diagnostics: CharacterMotionDiagnostics,
    },
    InitialOverlap {
        hit: CollisionHit,
        diagnostics: CharacterMotionDiagnostics,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CharacterMotionError {
    InvalidFixedStep,
    InvalidPhysicalState,
    InvalidDesiredVelocity,
    InvalidRadius,
    InvalidGravity,
    InvalidUpVector,
    InvalidSlopeLimit,
    InvalidContactSkin,
    InvalidSupportProbe,
    InvalidSlideIterationLimit,
    SpatialQueryFailed,
    SweepTooBroad,
}

pub fn evaluate_character_motion(
    input: CharacterMotionInput,
    partition: &GridPartitionConfig,
    store: &SdfChunkStore,
    query: &CollisionQueryService,
) -> Result<CharacterMotionOutcome, CharacterMotionError> {
    validate_input(&input)?;

    let up = normalize3(input.config.up).ok_or(CharacterMotionError::InvalidUpVector)?;
    let prior_vertical_speed = dot3(input.state.velocity, up);
    let desired_planar_velocity = project_onto_plane(input.desired_planar_velocity, up);
    let vertical_speed =
        prior_vertical_speed - input.config.gravity_acceleration * input.fixed_step_seconds;
    let mut velocity = add3(desired_planar_velocity, scale3(up, vertical_speed));
    let requested_displacement = scale3(velocity, input.fixed_step_seconds);
    let mut diagnostics = CharacterMotionDiagnostics::new(requested_displacement);

    match sweep(
        query,
        partition,
        store,
        input.world_id,
        input.state.position,
        input.state.position,
        input.config.radius,
    )? {
        CollisionSweepOutcome::MissingPayload { chunk_id } => {
            return Ok(CharacterMotionOutcome::MissingPayload {
                chunk_id,
                diagnostics,
            });
        }
        CollisionSweepOutcome::Hit(hit) => {
            return Ok(CharacterMotionOutcome::InitialOverlap { hit, diagnostics });
        }
        CollisionSweepOutcome::Clear => {}
    }

    let initial_position = input.state.position;
    let mut position = initial_position;
    let mut remaining = requested_displacement;

    for iteration in 0..input.config.max_slide_iterations {
        if length3(remaining) <= f32::EPSILON {
            break;
        }

        let target = add3(position, remaining);
        match sweep(
            query,
            partition,
            store,
            input.world_id,
            position,
            target,
            input.config.radius,
        )? {
            CollisionSweepOutcome::MissingPayload { chunk_id } => {
                diagnostics.admitted_displacement = sub3(position, initial_position);
                return Ok(CharacterMotionOutcome::MissingPayload {
                    chunk_id,
                    diagnostics,
                });
            }
            CollisionSweepOutcome::Clear => {
                position = target;
                remaining = [0.0; 3];
                break;
            }
            CollisionSweepOutcome::Hit(hit) => {
                let normal =
                    normalize3(hit.normal).ok_or(CharacterMotionError::SpatialQueryFailed)?;
                let to_hit = sub3(hit.hit_position, position);
                let direction =
                    normalize3(remaining).ok_or(CharacterMotionError::SpatialQueryFailed)?;
                let safe_distance = (length3(to_hit) - input.config.contact_skin)
                    .max(0.0)
                    .min(length3(remaining));
                position = add3(position, scale3(direction, safe_distance));

                diagnostics.contacts.push(CharacterContact {
                    hit_position: hit.hit_position,
                    normal,
                });
                diagnostics.slide_iterations = iteration.saturating_add(1);

                let to_target = sub3(target, position);
                remaining = project_onto_plane(to_target, normal);
                velocity = clip_into_surface(velocity, normal);
            }
        }
    }

    if length3(remaining) > f32::EPSILON {
        diagnostics.iteration_limit_reached = true;
    }

    let support_target = sub3(position, scale3(up, input.config.support_probe_distance));
    match sweep(
        query,
        partition,
        store,
        input.world_id,
        position,
        support_target,
        input.config.radius,
    )? {
        CollisionSweepOutcome::MissingPayload { chunk_id } => {
            diagnostics.admitted_displacement = sub3(position, initial_position);
            return Ok(CharacterMotionOutcome::MissingPayload {
                chunk_id,
                diagnostics,
            });
        }
        CollisionSweepOutcome::Clear => {}
        CollisionSweepOutcome::Hit(hit) => {
            let normal = normalize3(hit.normal).ok_or(CharacterMotionError::SpatialQueryFailed)?;
            let minimum_walkable_dot = input.config.max_walkable_slope_radians.cos();
            if dot3(normal, up) >= minimum_walkable_dot {
                diagnostics.grounded = true;
                diagnostics.support_normal = Some(normal);
                velocity = clip_into_surface(velocity, normal);
            } else {
                diagnostics.rejected_steep_support = true;
            }
        }
    }

    diagnostics.admitted_displacement = sub3(position, initial_position);
    let state = CharacterPhysicalState {
        position,
        velocity,
        grounded: diagnostics.grounded,
        support_normal: diagnostics.support_normal,
    };
    Ok(CharacterMotionOutcome::Completed { state, diagnostics })
}

fn validate_input(input: &CharacterMotionInput) -> Result<(), CharacterMotionError> {
    if !input.fixed_step_seconds.is_finite()
        || input.fixed_step_seconds <= 0.0
        || input.fixed_step_seconds > MAX_FIXED_STEP_SECONDS
    {
        return Err(CharacterMotionError::InvalidFixedStep);
    }
    if !finite3(input.state.position)
        || !finite3(input.state.velocity)
        || !length3(input.state.velocity).is_finite()
        || length3(input.state.velocity) > MAX_LINEAR_SPEED
        || input
            .state
            .support_normal
            .is_some_and(|normal| !finite3(normal) || !length3(normal).is_finite())
    {
        return Err(CharacterMotionError::InvalidPhysicalState);
    }
    if !finite3(input.desired_planar_velocity)
        || !length3(input.desired_planar_velocity).is_finite()
        || length3(input.desired_planar_velocity) > MAX_LINEAR_SPEED
    {
        return Err(CharacterMotionError::InvalidDesiredVelocity);
    }
    if !input.config.radius.is_finite()
        || input.config.radius <= 0.0
        || input.config.radius > MAX_CHARACTER_RADIUS
    {
        return Err(CharacterMotionError::InvalidRadius);
    }
    if !input.config.gravity_acceleration.is_finite()
        || input.config.gravity_acceleration < 0.0
        || input.config.gravity_acceleration > MAX_GRAVITY_ACCELERATION
    {
        return Err(CharacterMotionError::InvalidGravity);
    }
    let up_length = length3(input.config.up);
    if !finite3(input.config.up) || !up_length.is_finite() || up_length <= f32::EPSILON {
        return Err(CharacterMotionError::InvalidUpVector);
    }
    if !input.config.max_walkable_slope_radians.is_finite()
        || !(0.0..std::f32::consts::FRAC_PI_2).contains(&input.config.max_walkable_slope_radians)
    {
        return Err(CharacterMotionError::InvalidSlopeLimit);
    }
    if !input.config.contact_skin.is_finite()
        || input.config.contact_skin < 0.0
        || input.config.contact_skin >= input.config.radius
    {
        return Err(CharacterMotionError::InvalidContactSkin);
    }
    if !input.config.support_probe_distance.is_finite()
        || input.config.support_probe_distance <= 0.0
        || input.config.support_probe_distance > MAX_SUPPORT_PROBE_DISTANCE
    {
        return Err(CharacterMotionError::InvalidSupportProbe);
    }
    if input.config.max_slide_iterations == 0
        || input.config.max_slide_iterations > MAX_SLIDE_ITERATIONS
    {
        return Err(CharacterMotionError::InvalidSlideIterationLimit);
    }
    Ok(())
}

fn sweep(
    query: &CollisionQueryService,
    partition: &GridPartitionConfig,
    store: &SdfChunkStore,
    world_id: WorldId,
    start: [f32; 3],
    end: [f32; 3],
    radius: f32,
) -> Result<CollisionSweepOutcome, CharacterMotionError> {
    validate_sweep_chunk_bound(partition, world_id, start, end, radius)?;
    query
        .sweep_sphere_authoritative(
            partition,
            store,
            world_id,
            SphereSweep { start, end, radius },
        )
        .map_err(|_| CharacterMotionError::SpatialQueryFailed)
}

fn validate_sweep_chunk_bound(
    partition: &GridPartitionConfig,
    world_id: WorldId,
    start: [f32; 3],
    end: [f32; 3],
    radius: f32,
) -> Result<(), CharacterMotionError> {
    let extent = f64::from(radius.max(0.0));
    let min = [
        f64::from(start[0].min(end[0])) - extent,
        f64::from(start[1].min(end[1])) - extent,
        f64::from(start[2].min(end[2])) - extent,
    ];
    let max = [
        f64::from(start[0].max(end[0])) + extent,
        f64::from(start[1].max(end[1])) + extent,
        f64::from(start[2].max(end[2])) + extent,
    ];

    let min_coord = partition
        .chunk_coord_from_world_position(
            WorldPosition::try_new(world_id, min)
                .map_err(|_| CharacterMotionError::SpatialQueryFailed)?,
        )
        .map_err(|_| CharacterMotionError::SpatialQueryFailed)?;
    let max_coord = partition
        .chunk_coord_from_world_position(
            WorldPosition::try_new(world_id, max)
                .map_err(|_| CharacterMotionError::SpatialQueryFailed)?,
        )
        .map_err(|_| CharacterMotionError::SpatialQueryFailed)?;

    let span = |minimum: i64, maximum: i64| -> Result<u128, CharacterMotionError> {
        let width = i128::from(maximum)
            .checked_sub(i128::from(minimum))
            .and_then(|delta| delta.checked_add(1))
            .filter(|width| *width > 0)
            .ok_or(CharacterMotionError::SweepTooBroad)?;
        u128::try_from(width).map_err(|_| CharacterMotionError::SweepTooBroad)
    };

    let x = span(min_coord.x, max_coord.x)?;
    let y = span(min_coord.y, max_coord.y)?;
    let z = span(min_coord.z, max_coord.z)?;
    let total = x
        .checked_mul(y)
        .and_then(|xy| xy.checked_mul(z))
        .ok_or(CharacterMotionError::SweepTooBroad)?;
    if total > MAX_SWEEP_CHUNKS {
        return Err(CharacterMotionError::SweepTooBroad);
    }
    Ok(())
}

fn finite3(value: [f32; 3]) -> bool {
    value.into_iter().all(f32::is_finite)
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(value: [f32; 3], scale: f32) -> [f32; 3] {
    [value[0] * scale, value[1] * scale, value[2] * scale]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length3(value: [f32; 3]) -> f32 {
    dot3(value, value).sqrt()
}

fn normalize3(value: [f32; 3]) -> Option<[f32; 3]> {
    let length = length3(value);
    (length > f32::EPSILON).then(|| scale3(value, 1.0 / length))
}

fn project_onto_plane(value: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    sub3(value, scale3(normal, dot3(value, normal)))
}

fn clip_into_surface(value: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    let into_surface = dot3(value, normal);
    if into_surface < 0.0 {
        sub3(value, scale3(normal, into_surface))
    } else {
        value
    }
}
