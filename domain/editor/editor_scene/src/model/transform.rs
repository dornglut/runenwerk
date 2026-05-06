//! File: domain/editor/editor_scene/src/model/transform.rs
//! Purpose: Engine-neutral scene transform authoring contracts.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl SceneVec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub const fn one() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneQuat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl SceneQuat {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub const fn identity() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneTransform {
    pub translation: SceneVec3,
    pub rotation: SceneQuat,
    pub scale: SceneVec3,
}

impl SceneTransform {
    pub const fn new(translation: SceneVec3, rotation: SceneQuat, scale: SceneVec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub const fn identity() -> Self {
        Self::new(SceneVec3::zero(), SceneQuat::identity(), SceneVec3::one())
    }
}

impl Default for SceneTransform {
    fn default() -> Self {
        Self::identity()
    }
}
