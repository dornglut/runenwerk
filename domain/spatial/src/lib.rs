pub mod frames;
pub mod ids;
pub mod positions;

pub mod grid;
pub mod clipmap;
pub mod hash;
pub mod ring;

pub use frames::{build_camera_relative_frame, CameraRelativeFrame, WorldFrame};
pub use ids::WorldId;
pub use positions::{WorldLocalPosition, WorldPosition};

pub use grid::{
	ChunkCoord3,
	ChunkId,
	GridLevel,
	GridPartitionConfig,
	HierarchicalChunkId,
	HierarchicalGridConfig,
	RegionCoord3,
	RegionId,
};

pub use clipmap::{
	ClipmapCellId,
	ClipmapConfig,
	ClipmapCoord3,
	ClipmapLevel,
	ClipmapWindow,
	coord_from_world_local_position as clipmap_coord_from_world_local_position,
	window_for_center as clipmap_window_for_center,
};

pub use ring::{slot_for_coord as ring_slot_for_coord, RingBufferConfig, RingSlot3};