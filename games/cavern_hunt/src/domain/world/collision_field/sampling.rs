use super::{ChunkBrick3, GeometryOp, GeometryPrimitive3};

pub(super) fn sample_t(index: usize, resolution: usize) -> f32 {
    if resolution <= 1 {
        0.0
    } else {
        index as f32 / (resolution - 1) as f32
    }
}

pub(super) fn inverse_lerp(min: f32, max: f32, value: f32) -> f32 {
    if (max - min).abs() <= f32::EPSILON {
        0.0
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }
}

pub(super) fn sample_brick_trilinear(brick: &ChunkBrick3, local: [f32; 3]) -> f32 {
    let max_x = brick.resolution[0].saturating_sub(1);
    let max_y = brick.resolution[1].saturating_sub(1);
    let max_z = brick.resolution[2].saturating_sub(1);
    let fx = local[0] * max_x as f32;
    let fy = local[1] * max_y as f32;
    let fz = local[2] * max_z as f32;
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let z0 = fz.floor() as usize;
    let x1 = (x0 + 1).min(max_x);
    let y1 = (y0 + 1).min(max_y);
    let z1 = (z0 + 1).min(max_z);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let tz = fz - z0 as f32;
    let c000 = brick_value(brick, x0, y0, z0);
    let c100 = brick_value(brick, x1, y0, z0);
    let c010 = brick_value(brick, x0, y1, z0);
    let c110 = brick_value(brick, x1, y1, z0);
    let c001 = brick_value(brick, x0, y0, z1);
    let c101 = brick_value(brick, x1, y0, z1);
    let c011 = brick_value(brick, x0, y1, z1);
    let c111 = brick_value(brick, x1, y1, z1);

    let c00 = lerp(c000, c100, tx);
    let c10 = lerp(c010, c110, tx);
    let c01 = lerp(c001, c101, tx);
    let c11 = lerp(c011, c111, tx);
    let c0 = lerp(c00, c10, ty);
    let c1 = lerp(c01, c11, ty);
    lerp(c0, c1, tz)
}

fn brick_value(brick: &ChunkBrick3, x: usize, y: usize, z: usize) -> f32 {
    let width = brick.resolution[0];
    let height = brick.resolution[1];
    brick.distances[z * width * height + y * width + x]
}

pub(super) fn distance_analytic_from_primitives(
    primitives: &[&GeometryPrimitive3],
    point: [f32; 3],
) -> f32 {
    let mut walkable = f32::INFINITY;
    for primitive in primitives {
        let sdf = primitive.shape.signed_distance(point);
        match primitive.op {
            GeometryOp::SubtractVoid | GeometryOp::MaskWalkable => {
                walkable = walkable.min(sdf);
            }
            GeometryOp::Blocker => {
                walkable = walkable.max(-sdf);
            }
            GeometryOp::AddSolid | GeometryOp::HazardVolume => {}
        }
    }
    walkable
}

pub(super) fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let length = length3(v);
    if length <= 0.0001 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / length, v[1] / length, v[2] / length]
    }
}

pub(super) fn length3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

pub(super) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
