use crate::ui::{UiDrawCmd, UiDrawList};
use bytemuck::{Pod, Zeroable};
use image::GenericImageView;
use rusttype::{Font, Scale, point};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use wgpu::util::DeviceExt;
use wgpu::*;

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

#[derive(Debug, Default)]
pub struct FileFontProvider;

impl FileFontProvider {
    fn msdf_paths() -> [(&'static str, &'static str); 2] {
        [
            (
                "assets/fonts/console_msdf.png",
                "assets/fonts/console_msdf.json",
            ),
            (
                "engine_v2/assets/console_msdf.png",
                "engine_v2/assets/console_msdf.json",
            ),
        ]
    }

    fn ttf_paths() -> [&'static str; 2] {
        [
            "assets/fonts/JetBrainsMono-Regular.ttf",
            "engine_v2/assets/JetBrainsMono-Regular.ttf",
        ]
    }

    fn load_msdf_assets() -> Option<LoadedFontAtlas> {
        for (png_path, json_path) in Self::msdf_paths() {
            if !Path::new(png_path).exists() || !Path::new(json_path).exists() {
                continue;
            }

            let png = image::open(png_path).ok()?;
            let (img_w, img_h) = png.dimensions();
            let rgba = png.to_rgba8().into_raw();

            let json_text = fs::read_to_string(json_path).ok()?;
            let parsed: MsdfAtlasJson = serde_json::from_str(&json_text).ok()?;

            let atlas_w = parsed.atlas.width.unwrap_or(img_w).max(1);
            let atlas_h = parsed.atlas.height.unwrap_or(img_h).max(1);
            let y_origin_bottom = parsed
                .atlas
                .y_origin
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case("bottom"))
                .unwrap_or(false);

            let base_size = parsed.atlas.size.unwrap_or(32.0).max(1.0);
            let line_height = parsed
                .metrics
                .as_ref()
                .and_then(|m| m.line_height)
                .unwrap_or(base_size)
                * base_size;
            let ascent = parsed
                .metrics
                .as_ref()
                .and_then(|m| m.ascender)
                .unwrap_or(0.8)
                * base_size;

            let mut glyphs = HashMap::new();
            for glyph in parsed.glyphs {
                let Some(ch) = char::from_u32(glyph.unicode) else {
                    continue;
                };

                let advance_px = glyph.advance * base_size;
                let (size_px, bearing_px, uv_min, uv_max) =
                    if let (Some(plane), Some(atlas)) = (glyph.plane_bounds, glyph.atlas_bounds) {
                        let uv_left = atlas.left / atlas_w as f32;
                        let uv_right = atlas.right / atlas_w as f32;

                        let (uv_top, uv_bottom) = if y_origin_bottom {
                            (
                                1.0 - (atlas.top / atlas_h as f32),
                                1.0 - (atlas.bottom / atlas_h as f32),
                            )
                        } else {
                            (atlas.top / atlas_h as f32, atlas.bottom / atlas_h as f32)
                        };

                        (
                            [
                                (plane.right - plane.left) * base_size,
                                (plane.top - plane.bottom) * base_size,
                            ],
                            [plane.left * base_size, plane.top * base_size],
                            [uv_left, uv_top],
                            [uv_right, uv_bottom],
                        )
                    } else {
                        ([0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0])
                    };

                glyphs.insert(
                    ch,
                    GlyphMetrics {
                        uv_min,
                        uv_max,
                        size_px,
                        bearing_px,
                        advance_px,
                    },
                );
            }

            if glyphs.is_empty() {
                continue;
            }

            return Some(LoadedFontAtlas {
                width: atlas_w,
                height: atlas_h,
                rgba8_pixels: rgba,
                glyphs,
                base_size,
                line_height: line_height.max(1.0),
                ascent,
                sampling: TextSampling::Msdf,
            });
        }

        None
    }

    fn load_ttf_bytes() -> Option<Vec<u8>> {
        for path in Self::ttf_paths() {
            if Path::new(path).exists() {
                if let Ok(bytes) = fs::read(path) {
                    return Some(bytes);
                }
            }
        }
        None
    }

    fn placeholder() -> LoadedFontAtlas {
        let mut glyphs = HashMap::new();
        for ch in 32u8..=126u8 {
            glyphs.insert(
                ch as char,
                GlyphMetrics {
                    uv_min: [0.0, 0.0],
                    uv_max: [1.0, 1.0],
                    size_px: [8.0, 14.0],
                    bearing_px: [0.0, 11.0],
                    advance_px: 8.0,
                },
            );
        }

        LoadedFontAtlas {
            width: 1,
            height: 1,
            rgba8_pixels: vec![255, 255, 255, 255],
            glyphs,
            base_size: 18.0,
            line_height: 22.0,
            ascent: 16.0,
            sampling: TextSampling::Alpha,
        }
    }
}

impl FontAtlasProvider for FileFontProvider {
    fn load_default_font(&self) -> LoadedFontAtlas {
        if let Some(msdf) = Self::load_msdf_assets() {
            return msdf;
        }

        let Some(ttf_bytes) = Self::load_ttf_bytes() else {
            return Self::placeholder();
        };

        build_bitmap_font_atlas(&ttf_bytes).unwrap_or_else(Self::placeholder)
    }
}

#[derive(Debug, Deserialize)]
struct MsdfAtlasJson {
    atlas: MsdfAtlasInfo,
    #[serde(default)]
    metrics: Option<MsdfMetrics>,
    glyphs: Vec<MsdfGlyph>,
}

#[derive(Debug, Deserialize)]
struct MsdfAtlasInfo {
    #[serde(default)]
    size: Option<f32>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(rename = "yOrigin", default)]
    y_origin: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MsdfMetrics {
    #[serde(rename = "lineHeight", default)]
    line_height: Option<f32>,
    #[serde(default)]
    ascender: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct MsdfGlyph {
    unicode: u32,
    advance: f32,
    #[serde(rename = "planeBounds", default)]
    plane_bounds: Option<MsdfBounds>,
    #[serde(rename = "atlasBounds", default)]
    atlas_bounds: Option<MsdfBounds>,
}

#[derive(Debug, Deserialize)]
struct MsdfBounds {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GlyphInstanceRaw {
    rect: [f32; 4],
    uv: [f32; 4],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct ScreenUniformRaw {
    size: [f32; 2],
    _pad: [f32; 2],
}

#[derive(Debug)]
pub struct TextRenderer {
    pipeline: RenderPipeline,
    screen_buffer: Buffer,
    screen_bind_group: BindGroup,
    atlas_bind_group: BindGroup,
    glyphs: HashMap<char, GlyphMetrics>,
    base_size: f32,
    line_height: f32,
    ascent: f32,
}

impl TextRenderer {
    pub fn new(
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        provider: &dyn FontAtlasProvider,
    ) -> Self {
        let loaded = provider.load_default_font();

        let atlas_texture = device.create_texture(&TextureDescriptor {
            label: Some("engine_v2_text_atlas"),
            size: Extent3d {
                width: loaded.width.max(1),
                height: loaded.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let atlas_view = atlas_texture.create_view(&TextureViewDescriptor::default());
        let atlas_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("engine_v2_text_atlas_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let screen_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_v2_text_screen_uniform"),
            size: std::mem::size_of::<ScreenUniformRaw>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let screen_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_v2_text_screen_bgl"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let atlas_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("engine_v2_text_atlas_bgl"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let screen_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("engine_v2_text_screen_bg"),
            layout: &screen_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });
        let atlas_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("engine_v2_text_atlas_bg"),
            layout: &atlas_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&atlas_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_v2_ui_text_shader"),
            source: ShaderSource::Wgsl(
                match loaded.sampling {
                    TextSampling::Msdf => UI_TEXT_SHADER_MSDF,
                    TextSampling::Alpha => UI_TEXT_SHADER_ALPHA,
                }
                .into(),
            ),
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_v2_ui_text_pipeline_layout"),
            bind_group_layouts: &[&screen_bind_group_layout, &atlas_bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_v2_ui_text_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<GlyphInstanceRaw>() as u64,
                    step_mode: VertexStepMode::Instance,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let bytes_per_row = loaded.width.max(1) * 4;
        let rows_per_image = loaded.height.max(1);
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &atlas_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &loaded.rgba8_pixels,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(rows_per_image),
            },
            Extent3d {
                width: loaded.width.max(1),
                height: loaded.height.max(1),
                depth_or_array_layers: 1,
            },
        );

        Self {
            pipeline,
            screen_buffer,
            screen_bind_group,
            atlas_bind_group,
            glyphs: loaded.glyphs,
            base_size: loaded.base_size,
            line_height: loaded.line_height,
            ascent: loaded.ascent,
        }
    }

    pub fn write_screen_uniform(&self, queue: &Queue, surface_width: f32, surface_height: f32) {
        let screen = ScreenUniformRaw {
            size: [surface_width.max(1.0), surface_height.max(1.0)],
            _pad: [0.0; 2],
        };
        queue.write_buffer(&self.screen_buffer, 0, bytemuck::bytes_of(&screen));
    }

    pub fn build_instance_buffer(
        &self,
        device: &Device,
        draw_list: &UiDrawList,
    ) -> Option<(Buffer, u32)> {
        let instances = build_glyph_instances(
            draw_list,
            &self.glyphs,
            self.base_size,
            self.line_height,
            self.ascent,
        );
        if instances.is_empty() {
            return None;
        }

        let buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("engine_v2_ui_text_instances"),
            contents: bytemuck::cast_slice(&instances),
            usage: BufferUsages::VERTEX,
        });
        Some((buffer, instances.len() as u32))
    }

    pub fn encode_draw<'a>(
        &'a self,
        pass: &mut RenderPass<'a>,
        instance_buffer: &'a Buffer,
        count: u32,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.screen_bind_group, &[]);
        pass.set_bind_group(1, &self.atlas_bind_group, &[]);
        pass.set_vertex_buffer(0, instance_buffer.slice(..));
        pass.draw(0..6, 0..count);
    }
}

#[derive(Clone, Copy)]
struct GlyphPackMeta {
    ch: char,
    width: u32,
    height: u32,
    bearing_x: f32,
    bearing_y: f32,
    advance: f32,
    x: u32,
    y: u32,
}

fn build_bitmap_font_atlas(ttf_bytes: &[u8]) -> Option<LoadedFontAtlas> {
    let font = Font::try_from_bytes(ttf_bytes)?;
    let base_size = 18.0_f32;
    let scale = Scale::uniform(base_size);
    let v_metrics = font.v_metrics(scale);
    let ascent = v_metrics.ascent;
    let line_height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap).max(1.0);

    let mut metas = Vec::new();
    for code in 32u8..=126u8 {
        let ch = code as char;
        let glyph = font.glyph(ch).scaled(scale);
        let advance = glyph.h_metrics().advance_width;

        let positioned = glyph.positioned(point(0.0, 0.0));
        let bb = positioned.pixel_bounding_box();
        let (width, height, bearing_x, bearing_y) = if let Some(bb) = bb {
            (
                bb.width().max(0) as u32,
                bb.height().max(0) as u32,
                bb.min.x as f32,
                (-bb.min.y) as f32,
            )
        } else {
            (0, 0, 0.0, 0.0)
        };

        metas.push(GlyphPackMeta {
            ch,
            width,
            height,
            bearing_x,
            bearing_y,
            advance,
            x: 0,
            y: 0,
        });
    }

    let atlas_width = 1024_u32;
    let padding = 1_u32;
    let mut cursor_x = padding;
    let mut cursor_y = padding;
    let mut row_h = 0_u32;

    for meta in &mut metas {
        if meta.width == 0 || meta.height == 0 {
            continue;
        }

        if cursor_x + meta.width + padding > atlas_width {
            cursor_x = padding;
            cursor_y += row_h + padding;
            row_h = 0;
        }

        meta.x = cursor_x;
        meta.y = cursor_y;
        cursor_x += meta.width + padding;
        row_h = row_h.max(meta.height);
    }

    let atlas_height = (cursor_y + row_h + padding).max(1);
    let mut pixels = vec![0u8; (atlas_width * atlas_height * 4) as usize];

    for meta in &metas {
        if meta.width == 0 || meta.height == 0 {
            continue;
        }

        let glyph = font
            .glyph(meta.ch)
            .scaled(scale)
            .positioned(point(-meta.bearing_x, meta.bearing_y));

        glyph.draw(|x, y, v| {
            let px = meta.x + x;
            let py = meta.y + y;
            if px >= atlas_width || py >= atlas_height {
                return;
            }
            let idx = ((py * atlas_width + px) * 4) as usize;
            let d = (v.clamp(0.0, 1.0) * 255.0) as u8;
            pixels[idx] = d;
            pixels[idx + 1] = d;
            pixels[idx + 2] = d;
            pixels[idx + 3] = 255;
        });
    }

    let mut glyphs = HashMap::new();
    for meta in metas {
        if meta.width == 0 || meta.height == 0 {
            glyphs.insert(
                meta.ch,
                GlyphMetrics {
                    uv_min: [0.0, 0.0],
                    uv_max: [0.0, 0.0],
                    size_px: [0.0, 0.0],
                    bearing_px: [meta.bearing_x, meta.bearing_y],
                    advance_px: meta.advance,
                },
            );
            continue;
        }

        glyphs.insert(
            meta.ch,
            GlyphMetrics {
                uv_min: [
                    meta.x as f32 / atlas_width as f32,
                    meta.y as f32 / atlas_height as f32,
                ],
                uv_max: [
                    (meta.x + meta.width) as f32 / atlas_width as f32,
                    (meta.y + meta.height) as f32 / atlas_height as f32,
                ],
                size_px: [meta.width as f32, meta.height as f32],
                bearing_px: [meta.bearing_x, meta.bearing_y],
                advance_px: meta.advance,
            },
        );
    }

    Some(LoadedFontAtlas {
        width: atlas_width,
        height: atlas_height,
        rgba8_pixels: pixels,
        glyphs,
        base_size,
        line_height,
        ascent,
        sampling: TextSampling::Alpha,
    })
}

pub fn build_glyph_instances(
    draw_list: &UiDrawList,
    glyphs: &HashMap<char, GlyphMetrics>,
    base_size: f32,
    line_height: f32,
    ascent: f32,
) -> Vec<GlyphInstanceRaw> {
    let mut instances = Vec::new();
    let fallback = glyphs.get(&' ').copied().unwrap_or(GlyphMetrics {
        uv_min: [0.0, 0.0],
        uv_max: [0.0, 0.0],
        size_px: [0.0, 0.0],
        bearing_px: [0.0, 0.0],
        advance_px: 8.0,
    });

    for cmd in &draw_list.commands {
        let UiDrawCmd::Text {
            x,
            y,
            content,
            color,
            size,
            ..
        } = cmd
        else {
            continue;
        };

        let mut pen_x = *x;
        let scale = if base_size > 0.0 {
            (*size / base_size).max(0.1)
        } else {
            1.0
        };
        let scaled_line_height = line_height * scale;
        let scaled_ascent = ascent * scale;
        let mut baseline_y = *y + scaled_ascent;

        for ch in content.chars() {
            if ch == '\n' {
                pen_x = *x;
                baseline_y += scaled_line_height;
                continue;
            }

            let glyph = glyphs.get(&ch).copied().unwrap_or(fallback);
            let rect_w = glyph.size_px[0] * scale;
            let rect_h = glyph.size_px[1] * scale;

            if rect_w > 0.0 && rect_h > 0.0 {
                let rect_x = pen_x + glyph.bearing_px[0] * scale;
                let rect_y = baseline_y - glyph.bearing_px[1] * scale;

                instances.push(GlyphInstanceRaw {
                    rect: [rect_x, rect_y, rect_w, rect_h],
                    uv: [
                        glyph.uv_min[0],
                        glyph.uv_min[1],
                        glyph.uv_max[0],
                        glyph.uv_max[1],
                    ],
                    color: *color,
                });
            }

            pen_x += glyph.advance_px * scale;
        }
    }

    instances
}

#[cfg(test)]
mod tests {
    use super::{GlyphMetrics, build_glyph_instances};
    use crate::ui::{UiDrawCmd, UiDrawList};
    use std::collections::HashMap;

    #[test]
    fn build_glyph_instances_advances_horizontally() {
        let mut glyphs = HashMap::new();
        glyphs.insert(
            'a',
            GlyphMetrics {
                uv_min: [0.0, 0.0],
                uv_max: [0.5, 0.5],
                size_px: [8.0, 10.0],
                bearing_px: [0.0, 8.0],
                advance_px: 9.0,
            },
        );
        glyphs.insert(
            'b',
            GlyphMetrics {
                uv_min: [0.5, 0.0],
                uv_max: [1.0, 0.5],
                size_px: [8.0, 10.0],
                bearing_px: [0.0, 8.0],
                advance_px: 9.0,
            },
        );
        glyphs.insert(
            ' ',
            GlyphMetrics {
                uv_min: [0.0, 0.5],
                uv_max: [0.5, 1.0],
                size_px: [4.0, 10.0],
                bearing_px: [0.0, 8.0],
                advance_px: 5.0,
            },
        );

        let draw = UiDrawList {
            commands: vec![UiDrawCmd::Text {
                x: 10.0,
                y: 20.0,
                content: "ab".to_string(),
                color: [1.0, 1.0, 1.0, 1.0],
                size: 14.0,
                clip: None,
            }],
        };

        let instances = build_glyph_instances(&draw, &glyphs, 14.0, 16.0, 12.0);
        assert_eq!(instances.len(), 2);
        assert!((instances[0].rect[0] - 10.0).abs() < 0.001);
        assert!((instances[1].rect[0] - 19.0).abs() < 0.001);
    }
}
