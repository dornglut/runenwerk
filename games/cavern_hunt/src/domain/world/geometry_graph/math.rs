pub(super) fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(super) fn length3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

pub(super) fn sd_capsule3(point: [f32; 3], start: [f32; 3], end: [f32; 3], radius: f32) -> f32 {
    let pa = sub3(point, start);
    let ba = sub3(end, start);
    let denom = dot3(ba, ba).max(0.0001);
    let h = (dot3(pa, ba) / denom).clamp(0.0, 1.0);
    let closest = [
        start[0] + ba[0] * h,
        start[1] + ba[1] * h,
        start[2] + ba[2] * h,
    ];
    length3(sub3(point, closest)) - radius
}

pub(super) fn sd_box3(point: [f32; 3], center: [f32; 3], half_extents: [f32; 3]) -> f32 {
    let q = [
        (point[0] - center[0]).abs() - half_extents[0],
        (point[1] - center[1]).abs() - half_extents[1],
        (point[2] - center[2]).abs() - half_extents[2],
    ];
    let outside = [q[0].max(0.0), q[1].max(0.0), q[2].max(0.0)];
    let outside_len = length3(outside);
    let inside = q[0].max(q[1]).max(q[2]).min(0.0);
    outside_len + inside
}
