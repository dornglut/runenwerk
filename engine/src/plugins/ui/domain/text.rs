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

include!("text_internal/shaders_and_types.rs");

include!("text_internal/font_provider.rs");

include!("text_internal/renderer.rs");

include!("text_internal/atlas_builder.rs");

include!("text_internal/glyph_instances.rs");

include!("text_internal/tests.rs");
