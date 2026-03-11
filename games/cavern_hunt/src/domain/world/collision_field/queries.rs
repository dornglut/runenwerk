use super::sampling::{distance_analytic_from_primitives, length3, normalize3};
use super::{CavernCollisionField, CavernGeometryGraph, PushOutResult3, SweepHit3};

impl CavernCollisionField {
    pub fn distance(&mut self, graph: &CavernGeometryGraph, point: [f32; 3]) -> f32 {
        let key = self.chunk_key_for(point);
        self.ensure_chunk(graph, key);
        self.sample_chunk(key, point)
            .unwrap_or_else(|| self.distance_analytic(graph, point))
    }

    pub fn distance_analytic(&self, graph: &CavernGeometryGraph, point: [f32; 3]) -> f32 {
        distance_analytic_from_primitives(
            &graph
                .primitives
                .iter()
                .filter(|primitive| primitive.enabled)
                .collect::<Vec<_>>(),
            point,
        )
    }

    pub fn normal(&mut self, graph: &CavernGeometryGraph, point: [f32; 3]) -> [f32; 3] {
        let e = 0.05;
        let dx = self.distance(graph, [point[0] + e, point[1], point[2]])
            - self.distance(graph, [point[0] - e, point[1], point[2]]);
        let dy = self.distance(graph, [point[0], point[1] + e, point[2]])
            - self.distance(graph, [point[0], point[1] - e, point[2]]);
        let dz = self.distance(graph, [point[0], point[1], point[2] + e])
            - self.distance(graph, [point[0], point[1], point[2] - e]);
        normalize3([dx, dy, dz])
    }

    pub fn solid_at(&mut self, graph: &CavernGeometryGraph, point: [f32; 3]) -> bool {
        self.distance(graph, point) > 0.0
    }

    pub fn push_out_sphere(
        &mut self,
        graph: &CavernGeometryGraph,
        center: [f32; 3],
        radius: f32,
    ) -> PushOutResult3 {
        let distance = self.distance(graph, center);
        let penetration = distance + radius;
        if penetration <= 0.0 {
            return PushOutResult3 {
                collided: false,
                corrected_center: center,
                normal: [0.0, 1.0, 0.0],
                penetration: 0.0,
            };
        }
        let normal = self.normal(graph, center);
        let corrected_center = [
            center[0] - normal[0] * (penetration + 0.02),
            center[1] - normal[1] * (penetration + 0.02),
            center[2] - normal[2] * (penetration + 0.02),
        ];
        PushOutResult3 {
            collided: true,
            corrected_center,
            normal,
            penetration,
        }
    }

    pub fn push_out_capsule(
        &mut self,
        graph: &CavernGeometryGraph,
        base: [f32; 3],
        height: f32,
        radius: f32,
    ) -> PushOutResult3 {
        let top = [base[0], base[1] + height, base[2]];
        let base_push = self.push_out_sphere(graph, base, radius);
        let top_push = self.push_out_sphere(graph, top, radius);
        if !base_push.collided && !top_push.collided {
            return PushOutResult3 {
                collided: false,
                corrected_center: base,
                normal: [0.0, 1.0, 0.0],
                penetration: 0.0,
            };
        }
        let chosen = if top_push.penetration > base_push.penetration {
            top_push
        } else {
            base_push
        };
        PushOutResult3 {
            collided: true,
            corrected_center: chosen.corrected_center,
            normal: chosen.normal,
            penetration: chosen.penetration,
        }
    }

    pub fn sweep_sphere(
        &mut self,
        graph: &CavernGeometryGraph,
        start: [f32; 3],
        end: [f32; 3],
        radius: f32,
    ) -> SweepHit3 {
        let delta = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
        let length = length3(delta);
        let steps = ((length / 0.18).ceil() as usize).max(1);
        for step in 1..=steps {
            let fraction = step as f32 / steps as f32;
            let point = [
                start[0] + delta[0] * fraction,
                start[1] + delta[1] * fraction,
                start[2] + delta[2] * fraction,
            ];
            if self.distance(graph, point) > -radius {
                return SweepHit3 {
                    hit: true,
                    fraction,
                    point,
                    normal: self.normal(graph, point),
                };
            }
        }
        SweepHit3 {
            hit: false,
            fraction: 1.0,
            point: end,
            normal: [0.0, 1.0, 0.0],
        }
    }

    pub fn sweep_capsule(
        &mut self,
        graph: &CavernGeometryGraph,
        start: [f32; 3],
        end: [f32; 3],
        half_height: f32,
        radius: f32,
    ) -> SweepHit3 {
        let lower = self.sweep_sphere(graph, start, end, radius);
        let upper = self.sweep_sphere(
            graph,
            [start[0], start[1] + half_height * 2.0, start[2]],
            [end[0], end[1] + half_height * 2.0, end[2]],
            radius,
        );
        if upper.hit && upper.fraction < lower.fraction {
            upper
        } else {
            lower
        }
    }

    pub fn find_ground_below(
        &mut self,
        graph: &CavernGeometryGraph,
        origin: [f32; 3],
        max_drop: f32,
    ) -> Option<[f32; 3]> {
        let steps = ((max_drop / 0.1).ceil() as usize).max(1);
        for step in 0..=steps {
            let y = origin[1] - step as f32 * (max_drop / steps as f32);
            let point = [origin[0], y, origin[2]];
            if self.distance(graph, point) <= 0.0 {
                return Some(point);
            }
        }
        None
    }
}
