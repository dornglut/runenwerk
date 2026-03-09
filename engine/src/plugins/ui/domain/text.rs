use super::{UiDrawCmd, UiDrawList};
use bytemuck::{Pod, Zeroable};
use image::GenericImageView;
use rusttype::{Font, Scale, point};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use wgpu::util::DeviceExt;
use wgpu::*;

include!("text/internal/shaders_and_types.rs");

include!("text/internal/font_provider.rs");

include!("text/internal/renderer.rs");

include!("text/internal/atlas_builder.rs");

include!("text/internal/glyph_instances.rs");

include!("text/internal/tests.rs");
