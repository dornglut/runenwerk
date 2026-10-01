//! Bounded private camera-history delivery evidence. An explicitly requested qualification
//! run reads only the shader's fixed 32-cell diagnostic tail, after the exact producer
//! submission completes. The follow-up submission gates that producer until readback ends.

use super::*;
use runen_gpu::GpuBufferRange;
use std::path::PathBuf;

pub(super) const OUTPUT_FILE: &str = "camera-motion-cells.json";

#[derive(Debug, Clone)]
pub(super) struct CameraDiagnosticSnapshot {
    pub(super) history: GpuBufferHandle,
    pub(super) extent: (u32, u32),
    pub(super) phase: u32,
    pub(super) history_age: u32,
    pub(super) prior_same_pose_completed_frames: u32,
    pub(super) directory: PathBuf,
    pub(super) source_revision: String,
    pub(super) current_only_control: bool,
}

impl CameraDiagnosticSnapshot {
    pub(super) fn requested_directory() -> Option<PathBuf> {
        std::env::var_os("RUNENWERK_CAMERA_HISTORY_DIAGNOSTICS_DIR").map(PathBuf::from)
    }
}

#[derive(Debug)]
struct AcceptedSnapshot {
    snapshot: CameraDiagnosticSnapshot,
    frame_index: u64,
    submission: GpuSubmission,
}

#[derive(Debug)]
struct DiagnosticReadback {
    snapshot: CameraDiagnosticSnapshot,
    frame_index: u64,
    submission: GpuSubmission,
    readback: GpuReadbackId,
}

#[derive(Debug, Default)]
pub(super) struct CameraDiagnostics {
    prepared: BTreeMap<u64, CameraDiagnosticSnapshot>,
    accepted: BTreeMap<u64, AcceptedSnapshot>,
    readbacks: BTreeMap<u64, DiagnosticReadback>,
}

impl CameraDiagnostics {
    pub(super) fn prepare(&mut self, scope: u64, snapshot: CameraDiagnosticSnapshot) {
        self.prepared.insert(scope, snapshot);
    }

    pub(super) fn accept(&mut self, scope: u64, frame_index: u64, submission: &GpuSubmission) {
        if let Some(snapshot) = self.prepared.remove(&scope) {
            self.accepted.insert(
                scope,
                AcceptedSnapshot {
                    snapshot,
                    frame_index,
                    submission: submission.clone(),
                },
            );
        }
    }

    pub(super) fn progress(
        &mut self,
        context: &GpuContext,
        producers: &mut BTreeMap<u64, GpuSubmission>,
    ) -> anyhow::Result<()> {
        let mut completed = Vec::new();
        for (scope, readback) in &self.readbacks {
            match readback.submission.status() {
                GpuSubmissionStatus::Failed(failure) => {
                    anyhow::bail!("camera diagnostic submission failed: {failure:?}")
                }
                GpuSubmissionStatus::Accepted => continue,
                GpuSubmissionStatus::Completed => {}
            }
            match readback
                .submission
                .readback(readback.readback)
                .ok_or_else(|| {
                    anyhow::anyhow!("camera diagnostic readback lost its submission correlation")
                })?
                .status()
            {
                GpuReadbackStatus::Pending => continue,
                GpuReadbackStatus::Failed(failure) => {
                    anyhow::bail!("camera diagnostic readback failed: {failure:?}")
                }
                GpuReadbackStatus::Ready(bytes) => {
                    write_artifact(context, readback, bytes.as_bytes())?;
                    completed.push(*scope);
                }
            }
        }
        for scope in completed {
            self.readbacks.remove(&scope);
        }
        let mut ready = Vec::new();
        for (scope, snapshot) in &self.accepted {
            match snapshot.submission.status() {
                GpuSubmissionStatus::Accepted => {}
                GpuSubmissionStatus::Completed => ready.push(*scope),
                GpuSubmissionStatus::Failed(failure) => {
                    anyhow::bail!("camera diagnostic producer failed: {failure:?}")
                }
            }
        }
        for scope in ready {
            let accepted = self
                .accepted
                .remove(&scope)
                .expect("collected camera diagnostic scope");
            let offset = u64::from(accepted.snapshot.extent.0)
                * u64::from(accepted.snapshot.extent.1)
                * CAMERA_HISTORY_WORDS_PER_SAMPLE
                * WORD_BYTES;
            let range = GpuBufferRange::new(
                &accepted.snapshot.history,
                offset,
                CAMERA_DIAGNOSTIC_WORDS * WORD_BYTES,
            )?;
            let region = GpuBufferRegion::new(&accepted.snapshot.history, range)?;
            let operation = GpuReadbackOperation::ordinary(region.into())?;
            let readback = operation.id();
            let work = GpuWorkFragment::build("camera history cell diagnostics", |work| {
                work.operation("read exact completed first-motion cells", operation)?;
                Ok(())
            })?;
            let submission =
                pollster::block_on(context.submit_work("camera history cell diagnostics", [work]))?;
            // The diagnostic read owns the retained history until its exact readback completes.
            producers.insert(scope, submission.clone());
            self.readbacks.insert(
                scope,
                DiagnosticReadback {
                    snapshot: accepted.snapshot,
                    frame_index: accepted.frame_index,
                    submission,
                    readback,
                },
            );
        }
        Ok(())
    }
}

fn outcome(value: u32) -> &'static str {
    match value {
        0 => "accepted",
        1 => "current_miss",
        2 => "behind_previous_camera",
        3 => "out_of_bounds",
        4 => "previous_miss",
        5 => "depth_mismatch",
        6 => "invalid_history_count",
        7 => "nonfinite_history_sample",
        8 => "outside_current_requested_footprint",
        9 => "invalid_current_visibility_query",
        10 => "visible_point_does_not_match_anchor",
        11 => "revalidated_radiance_does_not_match",
        12 => "current_undefined",
        13 => "invalid_current_geometry",
        _ => "unknown",
    }
}

fn write_artifact(
    context: &GpuContext,
    readback: &DiagnosticReadback,
    bytes: &[u8],
) -> anyhow::Result<()> {
    let (chunks, remainder) = bytes.as_chunks::<4>();
    anyhow::ensure!(
        remainder.is_empty() && chunks.len() == CAMERA_DIAGNOSTIC_WORDS as usize,
        "camera diagnostic tail has an unexpected byte length"
    );
    let words = chunks
        .iter()
        .map(|word| u32::from_ne_bytes(*word))
        .collect::<Vec<_>>();
    let width = readback.snapshot.extent.0;
    let cell = |index: u32| (index != u32::MAX).then_some([index % width, index / width]);
    let scalar = |word: u32| {
        let value = f32::from_bits(word);
        value.is_finite().then_some(value)
    };
    let point = |words: &[u32]| words.iter().map(|word| scalar(*word)).collect::<Vec<_>>();
    let cells = words.as_chunks::<32>().0.iter().map(|w| {
        serde_json::json!({
            "current_cell": cell(w[0]), "previous_cell": cell(w[1]),
            "current_phase": w[2], "previous_phase": w[3],
            "current_defined": w[8] < 12, "current_hit": w[4] != 0, "current_depth": (w[4] != 0).then(|| scalar(w[5])).flatten(),
            "current_only_radiance": (w[8] < 12).then(|| scalar(w[6])).flatten(), "resulting_radiance": scalar(w[7]),
            "outcome": outcome(w[8]), "projected_depth": (w[8] == 0 || (5..=11).contains(&w[8])).then(|| scalar(w[9])).flatten(),
            "depth_tolerance_passed": w[8] == 0 || (6..=11).contains(&w[8]),
            "validated_history_sample_count": u32::from(w[8] == 0),
            "result_sample_count": (w[8] < 12).then_some(1 + u32::from(w[8] == 0 && !readback.snapshot.current_only_control)),
            "depth_tolerance": (w[8] == 0 || (5..=11).contains(&w[8])).then(|| scalar(w[10])).flatten(), "current_point": (w[4] != 0 && w[8] < 12).then(|| point(&w[11..14])),
            "previous_hit": (w[1] != u32::MAX).then_some(w[14] != 0), "previous_depth": (w[1] != u32::MAX && w[14] != 0).then(|| scalar(w[15])).flatten(),
            "retained_aggregate_radiance": (w[1] != u32::MAX).then(|| scalar(w[16])).flatten(), "retained_aggregate_count": (w[1] != u32::MAX).then_some(w[17]),
            "coherent_sample_radiance": scalar(w[18]),
            "coherent_sample_count": u32::from(w[14] != 0 && f32::from_bits(w[18]).is_finite()),
            "previous_anchor": (w[1] != u32::MAX && w[14] != 0).then(|| point(&w[19..22])), "revalidated_visible_point": (w[8] == 0 || (10..=11).contains(&w[8])).then(|| point(&w[22..25])),
            "revalidated_radiance": (w[8] == 0 || w[8] == 11).then(|| scalar(w[25])).flatten(),
            "phase_nearest_control": { "previous_cell": cell(w[26]), "outcome": outcome(w[27]),
                "projected_depth": scalar(w[28]), "depth_tolerance": scalar(w[29]),
                "coherent_sample_radiance": (w[26] != u32::MAX).then(|| scalar(w[30])).flatten(), "resulting_radiance": scalar(w[31]) },
            "raw_words": w,
        })
    }).collect::<Vec<_>>();
    let facts = context.adapter_facts();
    let artifact = serde_json::json!({
        "schema_version": 1, "source_git_revision": readback.snapshot.source_revision,
        "frame_index": readback.frame_index, "requested_size_px": [readback.snapshot.extent.0, readback.snapshot.extent.1],
        "phase": readback.snapshot.phase, "history_age": readback.snapshot.history_age,
        "prior_same_pose_completed_frames": readback.snapshot.prior_same_pose_completed_frames,
        "camera_reprojection_revision": CAMERA_REPROJECTION_REVISION,
        "qualification_control": if readback.snapshot.current_only_control { "current_only_first_motion" } else { "canonical" },
        "backend": format!("{:?}", facts.backend()), "adapter": facts.diagnostic_name(),
        "software_status": format!("{:?}", facts.software()), "cells": cells,
    });
    std::fs::create_dir_all(&readback.snapshot.directory)?;
    let path = readback.snapshot.directory.join(OUTPUT_FILE);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    serde_json::to_writer_pretty(file, &artifact)?;
    tracing::info!(path = %path.display(), frame_index = readback.frame_index, "retained first-motion camera cell diagnostics");
    Ok(())
}
