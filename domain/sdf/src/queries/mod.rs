pub mod classify;
pub mod closest_point;
pub mod project;
pub mod raymarch;
pub mod sweep;

pub use classify::PointClassification;
pub use closest_point::{ClosestPointHit, closest_point_on_surface};
pub use project::{ProjectHit, ProjectSettings, project_point_to_surface};
pub use raymarch::{RayHit, raymarch_first_hit};
pub use sweep::{SweepHit, SweepSettings, sweep_sphere};
