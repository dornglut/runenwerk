use crate::closest_point::closest_point_on_aabb3;
use crate::{Aabb2, Aabb3, Frustum, LineSegment3, Plane, Ray2, Ray3, Sphere, Triangle3};

const AXIS_EPSILON: f32 = 1e-8;

/// Returns the first non-negative hit distance for a 2D ray against an AABB.
///
/// Returns `Some(0.0)` when the ray origin is inside the bounds.
pub fn ray_aabb2_first_hit(ray: &Ray2, aabb: &Aabb2) -> Option<f32> {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    let origin = [ray.origin.x, ray.origin.y];
    let direction = [ray.direction.x, ray.direction.y];
    let min = [aabb.min.x, aabb.min.y];
    let max = [aabb.max.x, aabb.max.y];

    for axis in 0..2 {
        let o = origin[axis];
        let d = direction[axis];
        let mn = min[axis];
        let mx = max[axis];

        if d.abs() <= AXIS_EPSILON {
            if o < mn || o > mx {
                return None;
            }
            continue;
        }

        let inv_d = 1.0 / d;
        let t0 = (mn - o) * inv_d;
        let t1 = (mx - o) * inv_d;
        let near = t0.min(t1);
        let far = t0.max(t1);
        t_min = t_min.max(near);
        t_max = t_max.min(far);

        if t_min > t_max {
            return None;
        }
    }

    if t_max < 0.0 {
        None
    } else {
        Some(t_min.max(0.0))
    }
}

/// Returns the first non-negative hit distance for a 3D ray against an AABB.
///
/// Returns `Some(0.0)` when the ray origin is inside the bounds.
pub fn ray_aabb3_first_hit(ray: &Ray3, aabb: &Aabb3) -> Option<f32> {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    let origin = [ray.origin.x, ray.origin.y, ray.origin.z];
    let direction = [ray.direction.x, ray.direction.y, ray.direction.z];
    let min = [aabb.min.x, aabb.min.y, aabb.min.z];
    let max = [aabb.max.x, aabb.max.y, aabb.max.z];

    for axis in 0..3 {
        let o = origin[axis];
        let d = direction[axis];
        let mn = min[axis];
        let mx = max[axis];

        if d.abs() <= AXIS_EPSILON {
            if o < mn || o > mx {
                return None;
            }
            continue;
        }

        let inv_d = 1.0 / d;
        let t0 = (mn - o) * inv_d;
        let t1 = (mx - o) * inv_d;
        let near = t0.min(t1);
        let far = t0.max(t1);
        t_min = t_min.max(near);
        t_max = t_max.min(far);

        if t_min > t_max {
            return None;
        }
    }

    if t_max < 0.0 {
        None
    } else {
        Some(t_min.max(0.0))
    }
}

/// Returns first non-negative hit distance for a 3D ray against a sphere.
///
/// Returns `Some(0.0)` when the ray origin is inside the sphere.
pub fn ray_sphere_first_hit(ray: &Ray3, sphere: &Sphere) -> Option<f32> {
    let radius = sphere.radius.max(0.0);
    let m = ray.origin - sphere.center;
    let a = ray.direction.length_squared();
    if a <= AXIS_EPSILON {
        return None;
    }

    let c = m.length_squared() - radius * radius;
    if c <= 0.0 {
        return Some(0.0);
    }

    let b = m.dot(ray.direction);
    let discriminant = b * b - a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_d = discriminant.sqrt();
    let t0 = (-b - sqrt_d) / a;
    let t1 = (-b + sqrt_d) / a;

    if t0 >= 0.0 {
        Some(t0)
    } else if t1 >= 0.0 {
        Some(t1)
    } else {
        None
    }
}

/// Inclusive overlap test: touching the AABB counts as intersection.
pub fn sphere_aabb3_intersects(sphere: &Sphere, aabb: &Aabb3) -> bool {
    let radius = sphere.radius.max(0.0);
    let closest = closest_point_on_aabb3(aabb, sphere.center);
    closest.distance_squared(sphere.center) <= radius * radius
}

pub fn frustum_aabb3_intersects(frustum: &Frustum, aabb: &Aabb3) -> bool {
    frustum.intersects_aabb(aabb)
}

/// Returns first non-negative hit distance for a 3D ray against a plane.
pub fn ray_plane_first_hit(ray: &Ray3, plane: &Plane) -> Option<f32> {
    let denominator = plane.normal.dot(ray.direction);
    if denominator.abs() <= AXIS_EPSILON {
        return None;
    }

    let t = -(plane.normal.dot(ray.origin) + plane.distance) / denominator;
    if t >= 0.0 { Some(t) } else { None }
}

/// Inclusive overlap test for a line segment against an AABB.
pub fn segment_aabb3_intersects(segment: &LineSegment3, aabb: &Aabb3) -> bool {
    let direction = segment.direction();
    if direction.length_squared() <= AXIS_EPSILON {
        return aabb.contains_point(segment.a);
    }

    let mut t_min: f32 = 0.0;
    let mut t_max: f32 = 1.0;

    let origin = [segment.a.x, segment.a.y, segment.a.z];
    let delta = [direction.x, direction.y, direction.z];
    let min = [aabb.min.x, aabb.min.y, aabb.min.z];
    let max = [aabb.max.x, aabb.max.y, aabb.max.z];

    for axis in 0..3 {
        let o = origin[axis];
        let d = delta[axis];
        let mn = min[axis];
        let mx = max[axis];

        if d.abs() <= AXIS_EPSILON {
            if o < mn || o > mx {
                return false;
            }
            continue;
        }

        let inv_d = 1.0 / d;
        let t0 = (mn - o) * inv_d;
        let t1 = (mx - o) * inv_d;
        let near = t0.min(t1);
        let far = t0.max(t1);

        t_min = t_min.max(near);
        t_max = t_max.min(far);

        if t_min > t_max {
            return false;
        }
    }

    true
}

/// Returns first non-negative hit distance for a 3D ray against a triangle.
pub fn ray_triangle3_first_hit(ray: &Ray3, triangle: &Triangle3) -> Option<f32> {
    let edge1 = triangle.b - triangle.a;
    let edge2 = triangle.c - triangle.a;
    let p = ray.direction.cross(edge2);
    let det = edge1.dot(p);

    if det.abs() <= AXIS_EPSILON {
        return None;
    }

    let inv_det = 1.0 / det;
    let tvec = ray.origin - triangle.a;
    let u = tvec.dot(p) * inv_det;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = tvec.cross(edge1);
    let v = ray.direction.dot(q) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = edge2.dot(q) * inv_det;
    if t >= 0.0 { Some(t) } else { None }
}
