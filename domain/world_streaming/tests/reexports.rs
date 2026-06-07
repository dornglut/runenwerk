use chunking::{ChunkStreamingConfig, StreamingFocus};
use spatial::{GridPartitionConfig, WorldId};
use world_streaming::{StreamingTick, WorldStreamingConfig, WorldStreamingController};

#[test]
fn wrapper_reexports_payload_neutral_streaming_controller() {
    let config = WorldStreamingConfig::new(
        WorldId(7),
        GridPartitionConfig::default(),
        ChunkStreamingConfig::default(),
    );
    let mut controller = WorldStreamingController::new(config);

    let output = controller.tick(StreamingTick::from_focus(StreamingFocus::new([
        0.0, 0.0, 0.0,
    ])));

    assert_eq!(controller.world_id(), WorldId(7));
    assert!(
        !output.requests.is_empty(),
        "initial focus should produce load requests without owning payloads"
    );
}
