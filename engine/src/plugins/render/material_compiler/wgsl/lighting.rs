//! Private material-preview/scene direct-light realization; no public renderer API.
pub(super) const DIRECT_LIGHT_WGSL: &str = r#"
// Product-local GGX/Schlick fixed directional light. Linear RGB, perceptual
// roughness and metallic [0,1]. No IBL, transparency policy or normal mapping.
fn rw_safe_direction(value: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    let squared = dot(value, value);
    if squared < 0.00000001 { return fallback; }
    return value * inverseSqrt(squared);
}
fn rw_direct_material_light(
    material: MaterialEvalOutput,
    surface_normal: vec3<f32>,
    toward_camera: vec3<f32>,
    toward_light: vec3<f32>,
) -> vec3<f32> {
    let n = rw_safe_direction(surface_normal, vec3<f32>(0.0, 0.0, 1.0));
    let v = rw_safe_direction(toward_camera, n);
    let l = rw_safe_direction(toward_light, n);
    let h = rw_safe_direction(v + l, n);
    let nl = max(dot(n, l), 0.0);
    let nv = max(dot(n, v), 0.001);
    let nh = max(dot(n, h), 0.0);
    let vh = clamp(dot(v, h), 0.0, 1.0);
    let roughness = clamp(material.roughness, 0.0, 1.0);
    let metallic = clamp(material.metallic, 0.0, 1.0);
    let albedo = clamp(material.base_color.rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    // Clamp width and every potentially singular microfacet denominator.
    let alpha = max(roughness * roughness, 0.045 * 0.045);
    let a2 = alpha * alpha;
    let denominator = nh * nh * (a2 - 1.0) + 1.0;
    let distribution = a2 / max(3.14159265 * denominator * denominator, 0.00001);
    let k = (roughness + 1.0) * (roughness + 1.0) * 0.125;
    let geometry_light = nl / max(nl * (1.0 - k) + k, 0.00001);
    let geometry_view = nv / max(nv * (1.0 - k) + k, 0.00001);
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);
    let fresnel = f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - vh, 5.0);
    let diffuse = (vec3<f32>(1.0) - fresnel) * (1.0 - metallic) * albedo / 3.14159265;
    let specular = fresnel * (distribution * geometry_light * geometry_view)
        / max(4.0 * nl * nv, 0.0001);
    let ambient_floor = albedo * (1.0 - metallic) * 0.06;
    return clamp((diffuse + specular) * nl * 2.4 + ambient_floor + material.emissive,
        vec3<f32>(0.0), vec3<f32>(1.0));
}
"#;
