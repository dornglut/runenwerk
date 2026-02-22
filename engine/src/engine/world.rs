use crate::engine::entities::*;

pub struct World {
	pub camera: Camera,
}

impl World {
	pub fn new() -> Self {

		Self {
			camera: Default::default(),
		}
	}
}