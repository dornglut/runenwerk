use glam::{Vec2, Vec3};

use crate::{Aabb2, Aabb3, LineSegment2, LineSegment3, Sphere};

pub fn closest_point_on_aabb2(aabb: &Aabb2, point: Vec2) -> Vec2 {
    point.clamp(aabb.min, aabb.max)
}

pub fn closest_point_on_aabb3(aabb: &Aabb3, point: Vec3) -> Vec3 {
    point.clamp(aabb.min, aabb.max)
}

pub fn closest_point_on_segment2(segment: &LineSegment2, point: Vec2) -> Vec2 {
    let ab = segment.direction();
    let len_sq = ab.length_squared();
    if len_sq <= f32::EPSILON {
        return segment.a;
    }

    let t = (point - segment.a).dot(ab) / len_sq;
    segment.point_at(t.clamp(0.0, 1.0))
}

pub fn closest_point_on_segment3(segment: &LineSegment3, point: Vec3) -> Vec3 {
    let ab = segment.direction();
    let len_sq = ab.length_squared();
    if len_sq <= f32::EPSILON {
        return segment.a;
    }

    let t = (point - segment.a).dot(ab) / len_sq;
    segment.point_at(t.clamp(0.0, 1.0))
}

pub fn closest_point_on_sphere(sphere: &Sphere, point: Vec3) -> Vec3 {
    let radius = sphere.radius.max(0.0);
    let to_point = point - sphere.center;
    let len_sq = to_point.length_squared();

    if len_sq <= f32::EPSILON || radius <= f32::EPSILON {
        return sphere.center;
    }

    sphere.center + to_point / len_sq.sqrt() * radius
}
