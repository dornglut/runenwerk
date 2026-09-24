use glam::Vec3;

use crate::{Aabb3, Frustum, Plane};

const DEFAULT_EPSILON: f32 = 1e-6;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlanePointClassification {
    Front,
    Back,
    OnPlane,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlaneAabbClassification {
    Front,
    Back,
    Intersecting,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FrustumPointClassification {
    Inside,
    Outside,
}

pub fn classify_point_plane(plane: &Plane, point: Vec3) -> PlanePointClassification {
    classify_point_plane_with_epsilon(plane, point, DEFAULT_EPSILON)
}

pub fn classify_point_plane_with_epsilon(
    plane: &Plane,
    point: Vec3,
    epsilon: f32,
) -> PlanePointClassification {
    let signed_distance = plane.signed_distance(point);
    if signed_distance > epsilon {
        PlanePointClassification::Front
    } else if signed_distance < -epsilon {
        PlanePointClassification::Back
    } else {
        PlanePointClassification::OnPlane
    }
}

pub fn classify_aabb_plane(plane: &Plane, aabb: &Aabb3) -> PlaneAabbClassification {
    classify_aabb_plane_with_epsilon(plane, aabb, DEFAULT_EPSILON)
}

pub fn classify_aabb_plane_with_epsilon(
    plane: &Plane,
    aabb: &Aabb3,
    epsilon: f32,
) -> PlaneAabbClassification {
    let max_vertex = Vec3::new(
        if plane.normal.x >= 0.0 {
            aabb.max.x
        } else {
            aabb.min.x
        },
        if plane.normal.y >= 0.0 {
            aabb.max.y
        } else {
            aabb.min.y
        },
        if plane.normal.z >= 0.0 {
            aabb.max.z
        } else {
            aabb.min.z
        },
    );
    let min_vertex = Vec3::new(
        if plane.normal.x >= 0.0 {
            aabb.min.x
        } else {
            aabb.max.x
        },
        if plane.normal.y >= 0.0 {
            aabb.min.y
        } else {
            aabb.max.y
        },
        if plane.normal.z >= 0.0 {
            aabb.min.z
        } else {
            aabb.max.z
        },
    );

    let max_distance = plane.signed_distance(max_vertex);
    let min_distance = plane.signed_distance(min_vertex);

    if min_distance > epsilon {
        PlaneAabbClassification::Front
    } else if max_distance < -epsilon {
        PlaneAabbClassification::Back
    } else {
        PlaneAabbClassification::Intersecting
    }
}

pub fn classify_point_frustum(frustum: &Frustum, point: Vec3) -> FrustumPointClassification {
    if frustum.contains_point(point) {
        FrustumPointClassification::Inside
    } else {
        FrustumPointClassification::Outside
    }
}
