use crate::*;
use engine_sim::{
    DeterminismLevel, SimulationHash, SimulationProfile, SimulationSeed, SimulationSessionId,
    SimulationTick,
};

#[test]
fn recorder_retains_checkpoint_ring() {
    let header = ReplayHeader {
        format_version: ReplayHeader::FORMAT_VERSION,
        profile: SimulationProfile::DedicatedAuthority,
        determinism: DeterminismLevel::Validated,
        session_id: SimulationSessionId(1),
        seed: SimulationSeed(7),
        tick_rate_hz: 60,
        codec_id: "test".to_string(),
        codec_version: 1,
    };
    let mut recorder = ReplayRecorder::<u32, u8>::new(
        header,
        CheckpointPolicy {
            interval_ticks: 5,
            retained_checkpoints: 2,
            hash_every_tick: true,
        },
        ReplayStoragePolicy::default(),
    );
    for tick in 0..3 {
        recorder.record_checkpoint(ReplayCheckpoint {
            meta: ReplayCheckpointMeta {
                tick: SimulationTick(tick),
                hash: SimulationHash([tick as u8; 32]),
            },
            snapshot: tick as u32,
        });
    }
    assert_eq!(recorder.checkpoint_count(), 2);
}

#[test]
fn archive_round_trips_with_compression() {
    let archive = ReplayArchive {
        header: ReplayHeader {
            format_version: ReplayHeader::FORMAT_VERSION,
            profile: SimulationProfile::DedicatedAuthority,
            determinism: DeterminismLevel::Validated,
            session_id: SimulationSessionId(1),
            seed: SimulationSeed(3),
            tick_rate_hz: 60,
            codec_id: "scene".to_string(),
            codec_version: 1,
        },
        checkpoints: vec![ReplayCheckpoint {
            meta: ReplayCheckpointMeta {
                tick: SimulationTick(0),
                hash: SimulationHash([1; 32]),
            },
            snapshot: vec![1u8, 2, 3],
        }],
        journal: vec![ReplayJournalFrame {
            tick: SimulationTick(1),
            commands: vec![9u8],
            post_hash: Some(SimulationHash([2; 32])),
        }],
    };

    let bytes = archive.encode_compressed().expect("archive should encode");
    let decoded =
        ReplayArchive::<Vec<u8>, u8>::decode_compressed(&bytes).expect("archive should decode");
    assert_eq!(decoded, archive);
}
