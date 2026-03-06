// Owner: Engine UI Text - Shader Sources and Core Types
const UI_TEXT_SHADER_MSDF: &str = r#"
struct GlyphIn {
    @location(0) rect : vec4<f32>,
    @location(1) uv : vec4<f32>,
    @location(2) color : vec4<f32>,
};

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>,
    @location(1) color : vec4<f32>,
};

struct ScreenUniform {
    size : vec2<f32>,
    _pad : vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen : ScreenUniform;

@group(1) @binding(0)
var atlas_tex : texture_2d<f32>;
@group(1) @binding(1)
var atlas_sampler : sampler;

@vertex
fn vs_main(input: GlyphIn, @builtin(vertex_index) vertex_index: u32) -> VsOut {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );
    let p = corners[vertex_index];

    let pixel = vec2<f32>(
        input.rect.x + p.x * input.rect.z,
        input.rect.y + p.y * input.rect.w
    );
    let x_ndc = (pixel.x / screen.size.x) * 2.0 - 1.0;
    let y_ndc = 1.0 - (pixel.y / screen.size.y) * 2.0;

    let uv = vec2<f32>(
        mix(input.uv.x, input.uv.z, p.x),
        mix(input.uv.y, input.uv.w, p.y)
    );

    var out: VsOut;
    out.clip_position = vec4<f32>(x_ndc, y_ndc, 0.0, 1.0);
    out.uv = uv;
    out.color = input.color;
    return out;
}

fn median3(v: vec3<f32>) -> f32 {
    return max(min(v.x, v.y), min(max(v.x, v.y), v.z));
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let sample_rgb = textureSample(atlas_tex, atlas_sampler, input.uv).rgb;
    let distance = median3(sample_rgb);
    let edge = 0.5;
    let w = max(fwidth(distance), 0.0001);
    let alpha = smoothstep(edge - w, edge + w, distance);
    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
"#;

const UI_TEXT_SHADER_ALPHA: &str = r#"
struct GlyphIn {
    @location(0) rect : vec4<f32>,
    @location(1) uv : vec4<f32>,
    @location(2) color : vec4<f32>,
};

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>,
    @location(1) color : vec4<f32>,
};

struct ScreenUniform {
    size : vec2<f32>,
    _pad : vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen : ScreenUniform;

@group(1) @binding(0)
var atlas_tex : texture_2d<f32>;
@group(1) @binding(1)
var atlas_sampler : sampler;

@vertex
fn vs_main(input: GlyphIn, @builtin(vertex_index) vertex_index: u32) -> VsOut {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );
    let p = corners[vertex_index];

    let pixel = vec2<f32>(
        input.rect.x + p.x * input.rect.z,
        input.rect.y + p.y * input.rect.w
    );
    let x_ndc = (pixel.x / screen.size.x) * 2.0 - 1.0;
    let y_ndc = 1.0 - (pixel.y / screen.size.y) * 2.0;

    let uv = vec2<f32>(
        mix(input.uv.x, input.uv.z, p.x),
        mix(input.uv.y, input.uv.w, p.y)
    );

    var out: VsOut;
    out.clip_position = vec4<f32>(x_ndc, y_ndc, 0.0, 1.0);
    out.uv = uv;
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let alpha = textureSample(atlas_tex, atlas_sampler, input.uv).r;
    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSampling {
    Msdf,
    Alpha,
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub size_px: [f32; 2],
    pub bearing_px: [f32; 2],
    pub advance_px: f32,
}

#[derive(Debug)]
pub struct LoadedFontAtlas {
    pub width: u32,
    pub height: u32,
    pub rgba8_pixels: Vec<u8>,
    pub glyphs: HashMap<char, GlyphMetrics>,
    pub base_size: f32,
    pub line_height: f32,
    pub ascent: f32,
    pub sampling: TextSampling,
}

pub trait FontAtlasProvider {
    fn load_default_font(&self) -> LoadedFontAtlas;
}
