//! File: domain/ui/ui_math/src/lib.rs
//! Crate: ui_math

pub mod axis;
pub mod insets;
pub mod point;
pub mod rect;
pub mod size;
pub mod vector;

pub use axis::{Axis, AxisDirection};
pub use insets::UiInsets;
pub use point::UiPoint;
pub use rect::UiRect;
pub use size::UiSize;
pub use vector::UiVector;