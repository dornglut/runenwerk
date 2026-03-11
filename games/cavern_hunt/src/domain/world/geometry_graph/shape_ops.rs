use super::math::{length3, sd_box3, sd_capsule3, sub3};
use super::{GeometryBounds3, GeometryPrimitiveShape3};

impl GeometryPrimitiveShape3 {
    pub fn bounds(&self) -> GeometryBounds3 {
        match self {
            GeometryPrimitiveShape3::Sphere { center, radius } => {
                GeometryBounds3::from_center_half_extents(*center, [*radius, *radius, *radius])
            }
            GeometryPrimitiveShape3::Ellipsoid { center, radii } => {
                GeometryBounds3::from_center_half_extents(*center, *radii)
            }
            GeometryPrimitiveShape3::Capsule { start, end, radius } => GeometryBounds3 {
                min: [
                    start[0].min(end[0]) - *radius,
                    start[1].min(end[1]) - *radius,
                    start[2].min(end[2]) - *radius,
                ],
                max: [
                    start[0].max(end[0]) + *radius,
                    start[1].max(end[1]) + *radius,
                    start[2].max(end[2]) + *radius,
                ],
            },
            GeometryPrimitiveShape3::Box {
                center,
                half_extents,
            } => GeometryBounds3::from_center_half_extents(*center, *half_extents),
            GeometryPrimitiveShape3::RoundedBox {
                center,
                half_extents,
                radius,
            } => GeometryBounds3::from_center_half_extents(
                *center,
                [
                    half_extents[0] + *radius,
                    half_extents[1] + *radius,
                    half_extents[2] + *radius,
                ],
            ),
            GeometryPrimitiveShape3::Cylinder {
                center,
                radius,
                half_height,
            } => {
                GeometryBounds3::from_center_half_extents(*center, [*radius, *half_height, *radius])
            }
            GeometryPrimitiveShape3::TunnelSplineCapsuleChain { points, radius } => {
                let mut bounds = GeometryBounds3::from_center_half_extents(
                    points[0],
                    [*radius, *radius, *radius],
                );
                for point in points.iter().skip(1) {
                    bounds = bounds.union(&GeometryBounds3::from_center_half_extents(
                        *point,
                        [*radius, *radius, *radius],
                    ));
                }
                bounds
            }
        }
    }

    pub fn signed_distance(&self, point: [f32; 3]) -> f32 {
        match self {
            GeometryPrimitiveShape3::Sphere { center, radius } => {
                length3(sub3(point, *center)) - *radius
            }
            GeometryPrimitiveShape3::Ellipsoid { center, radii } => {
                let q = [
                    (point[0] - center[0]) / radii[0].max(0.001),
                    (point[1] - center[1]) / radii[1].max(0.001),
                    (point[2] - center[2]) / radii[2].max(0.001),
                ];
                (length3(q) - 1.0) * radii[0].min(radii[1]).min(radii[2])
            }
            GeometryPrimitiveShape3::Capsule { start, end, radius } => {
                sd_capsule3(point, *start, *end, *radius)
            }
            GeometryPrimitiveShape3::Box {
                center,
                half_extents,
            } => sd_box3(point, *center, *half_extents),
            GeometryPrimitiveShape3::RoundedBox {
                center,
                half_extents,
                radius,
            } => sd_box3(point, *center, *half_extents) - *radius,
            GeometryPrimitiveShape3::Cylinder {
                center,
                radius,
                half_height,
            } => {
                let px = point[0] - center[0];
                let py = (point[1] - center[1]).abs() - *half_height;
                let pz = point[2] - center[2];
                let radial = (px * px + pz * pz).sqrt() - *radius;
                let ax = radial.max(0.0);
                let ay = py.max(0.0);
                radial.max(py).min(0.0) + (ax * ax + ay * ay).sqrt()
            }
            GeometryPrimitiveShape3::TunnelSplineCapsuleChain { points, radius } => {
                let mut distance = f32::INFINITY;
                for window in points.windows(2) {
                    distance = distance.min(sd_capsule3(point, window[0], window[1], *radius));
                }
                distance
            }
        }
    }
}
