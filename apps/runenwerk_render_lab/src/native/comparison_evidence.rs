use super::*;
use engine::plugins::render::inspect::{RenderTextureDiffResult, RenderTextureDiffStatus};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const COMPARISON_SCENARIO_ID: &str = "rl2-native-comparison";
const COMPARISON_SCENARIO_REVISION: u32 = 1;
const COMPARISON_DIFF_ID: &str = "runenwerk.render_lab.rl2.compare.a_vs_b";

#[derive(Debug, Clone, Default, runen_ecs::Resource)]
pub(super) struct RenderLabComparisonEvidenceState {
    temporal_by_frame: BTreeMap<u64, Vec<runen_render::RenderTemporalExecutionEvidence>>,
}

impl RenderLabComparisonEvidenceState {
    fn observe_temporal_frame(
        &mut self,
        frame_index: u64,
        records: Vec<runen_render::RenderTemporalExecutionEvidence>,
    ) {
        self.temporal_by_frame.insert(frame_index, records);
        while self.temporal_by_frame.len() > RL2_MEASUREMENT_HISTORY_CAPACITY {
            let Some(oldest) = self.temporal_by_frame.keys().next().copied() else {
                break;
            };
            self.temporal_by_frame.remove(&oldest);
        }
    }

    pub(super) fn observe_available(
        &mut self,
        history: &RenderFrameHistoryState,
        gfx: &engine::plugins::render::Gfx,
    ) {
        for observation in history.observations() {
            let frame_index = observation.key.frame_index;
            if self.temporal_by_frame.contains_key(&frame_index) {
                continue;
            }
            let records = gfx.temporal_execution_evidence(frame_index);
            if !records.is_empty() {
                self.observe_temporal_frame(frame_index, records.to_vec());
            }
        }
    }

    fn temporal_frame(
        &self,
        frame_index: u64,
    ) -> Option<&[runen_render::RenderTemporalExecutionEvidence]> {
        self.temporal_by_frame.get(&frame_index).map(Vec::as_slice)
    }
}

fn compared_diff(
    diffs: &[RenderTextureDiffResult],
    target_frame: u64,
    output_size_px: (u32, u32),
) -> Result<&RenderTextureDiffResult> {
    if diffs.len() != 1 {
        bail!("comparison evidence requires exactly one diff on target frame {target_frame}");
    }
    let diff = &diffs[0];
    if diff.diff_id != COMPARISON_DIFF_ID || diff.status != RenderTextureDiffStatus::Compared {
        bail!("comparison evidence requires a Compared {COMPARISON_DIFF_ID} diff");
    }
    if diff
        .left_frame_identity
        .as_ref()
        .map(|identity| identity.frame_index)
        != Some(target_frame)
        || diff
            .right_frame_identity
            .as_ref()
            .map(|identity| identity.frame_index)
            != Some(target_frame)
    {
        bail!("comparison diff inputs do not belong to target frame {target_frame}");
    }
    let expected_pixels = u64::from(output_size_px.0) * u64::from(output_size_px.1);
    if diff
        .metrics
        .as_ref()
        .map(|metrics| metrics.total_pixel_count)
        != Some(expected_pixels)
    {
        bail!("comparison diff metrics do not cover the complete output extent");
    }
    Ok(diff)
}

fn temporal_frames(
    history: &RenderFrameHistoryState,
    evidence_state: &RenderLabComparisonEvidenceState,
    target_frame: u64,
    output_size_px: (u32, u32),
    candidate_size_px: (u32, u32),
) -> Result<Vec<Value>> {
    let mut generation_classes = Vec::new();
    let mut frames = Vec::new();
    for observation in history.observations() {
        let frame = observation.key.frame_index;
        let records = evidence_state.temporal_frame(frame).ok_or_else(|| {
            anyhow::anyhow!("comparison frame {frame} has no retained temporal evidence")
        })?;
        if records.len() != 2 {
            bail!(
                "comparison frame {frame} requires exactly two temporal records, found {}",
                records.len()
            );
        }
        let variant = |extent| -> Result<_> {
            let mut matches = records
                .iter()
                .filter(|record| record.evaluation_extent == extent);
            let record = matches.next().ok_or_else(|| {
                anyhow::anyhow!("comparison frame {frame} has no temporal record for {extent:?}")
            })?;
            if matches.next().is_some() {
                bail!("comparison frame {frame} has duplicate temporal extent {extent:?}");
            }
            Ok(record)
        };
        let reference = variant(output_size_px)?;
        let candidate = variant(candidate_size_px)?;
        if reference.requested_extent != output_size_px
            || candidate.requested_extent != output_size_px
            || reference.sequence_revision != candidate.sequence_revision
            || reference.reconstruction_revision != candidate.reconstruction_revision
            || reference.semantic_input_generations != candidate.semantic_input_generations
        {
            bail!("comparison frame {frame} temporal variants disagree on shared source truth");
        }
        let classes: Vec<u64> = reference
            .semantic_input_generations
            .iter()
            .map(|(id, generation)| {
                let index = if let Some(index) = generation_classes
                    .iter()
                    .position(|(known_id, _)| known_id == id)
                {
                    index
                } else {
                    generation_classes.push((*id, Vec::new()));
                    generation_classes.len() - 1
                };
                let seen = &mut generation_classes[index].1;
                let class = if let Some(class) = seen.iter().position(|known| known == generation) {
                    class
                } else {
                    seen.push(*generation);
                    seen.len() - 1
                };
                u64::try_from(class + 1).expect("bounded comparison generation class fits u64")
            })
            .collect();
        let record_json = |record: &runen_render::RenderTemporalExecutionEvidence| {
            json!({
                "requested_size_px": [record.requested_extent.0, record.requested_extent.1],
                "evaluation_size_px": [record.evaluation_extent.0, record.evaluation_extent.1],
                "semantic_input_generation_classes": classes,
                "sequence_revision": record.sequence_revision,
                "reconstruction_revision": record.reconstruction_revision,
                "phase": record.phase,
                "history_generation": record.history_generation,
                "history_age": record.history_age,
                "history_reset": record.history_reset,
                "camera_reprojection_eligible": record.camera_reprojection_eligible,
            })
        };
        frames.push(json!({
            "frame_index": frame,
            "reference": record_json(reference),
            "candidate": record_json(candidate),
        }));
        if frame == target_frame {
            break;
        }
    }
    if frames
        .last()
        .and_then(|frame| frame["frame_index"].as_u64())
        != Some(target_frame)
    {
        bail!("comparison target frame is absent from retained submitted-frame history");
    }
    Ok(frames)
}

pub(super) struct RenderLabComparisonEvidenceInputs<'a> {
    pub(super) config: &'a RenderLabComparisonEvidenceConfig,
    pub(super) history: &'a RenderFrameHistoryState,
    pub(super) evidence_state: &'a RenderLabComparisonEvidenceState,
    pub(super) debug_report: &'a RenderDebugFrameReportState,
    pub(super) gfx: &'a engine::plugins::render::Gfx,
    pub(super) target_frame: u64,
    pub(super) output_size_px: (u32, u32),
    pub(super) candidate_size_px: (u32, u32),
}

pub(super) fn write_comparison_evidence(
    inputs: RenderLabComparisonEvidenceInputs<'_>,
) -> Result<()> {
    let RenderLabComparisonEvidenceInputs {
        config,
        history,
        evidence_state,
        debug_report,
        gfx,
        target_frame,
        output_size_px,
        candidate_size_px,
    } = inputs;
    let diffs = debug_report
        .texture_diff_results_for_frame(target_frame)
        .ok_or_else(|| anyhow::anyhow!("comparison target frame has no retained texture diff"))?;
    let diff = compared_diff(diffs, target_frame, output_size_px)?;
    let temporal = temporal_frames(
        history,
        evidence_state,
        target_frame,
        output_size_px,
        candidate_size_px,
    )?;
    if temporal.len()
        != config
            .submitted_frame_limit()
            .expect("bounded comparison has frame limit")
    {
        bail!("comparison temporal evidence does not cover every submitted frame");
    }
    let results = debug_report
        .capture_results_for_frame(target_frame)
        .expect("validated target capture results");
    let capture_manifest_path = debug_report
        .capture_artifact_manifest_for_frame(target_frame)
        .ok_or_else(|| {
            anyhow::anyhow!("comparison target frame has no retained capture manifest")
        })?;
    let captures = config
        .capture_selectors()
        .iter()
        .map(|selector| {
            let result = results
                .iter()
                .find(|result| &result.selector == selector)
                .ok_or_else(|| {
                    anyhow::anyhow!("comparison target report lost a capture selector")
                })?;
            let artifact_path = result
                .artifact_path
                .as_ref()
                .expect("validated capture path");
            let artifact_bytes = fs::read(artifact_path).with_context(|| {
                format!(
                    "read comparison capture artifact {} for hashing",
                    artifact_path.display()
                )
            })?;
            Ok(json!({
                "flow_id": selector.flow_id.as_deref(),
                "pass_id": selector.pass_id.as_deref(),
                "resource_id": selector.resource_id,
                "frame_index": target_frame,
                "artifact_path": artifact_path.display().to_string(),
                "artifact_blake3": format!("blake3:{}", blake3::hash(&artifact_bytes).to_hex()),
            }))
        })
        .collect::<Result<Vec<Value>>>()?;
    let metrics = diff.metrics.expect("validated comparison metrics");
    let mismatch_samples: Vec<Value> = diff
        .mismatch_samples
        .iter()
        .map(|sample| {
            json!({
                "coordinate": [sample.coordinate.x, sample.coordinate.y],
                "left_rgba8": sample.left_rgba8,
                "right_rgba8": sample.right_rgba8,
                "max_channel_delta": sample.max_channel_delta,
            })
        })
        .collect();
    let evidence = json!({
        "schema_version": 1,
        "scenario_id": COMPARISON_SCENARIO_ID,
        "scenario_revision": COMPARISON_SCENARIO_REVISION,
        "source_git_revision": std::env::var("RUNENWERK_SOURCE_REVISION").ok(),
        "output_size_px": [output_size_px.0, output_size_px.1],
        "candidate_size_px": [candidate_size_px.0, candidate_size_px.1],
        "reference": {"identity": "native/P100", "internal_size_px": [output_size_px.0, output_size_px.1]},
        "candidate": {"identity": "accepted-static-footprint-temporal", "internal_size_px": [candidate_size_px.0, candidate_size_px.1]},
        "target_frame_index": target_frame,
        "capture_submission_ordinal": temporal.len(),
        "capture_manifest_path": capture_manifest_path.display().to_string(),
        "captures": captures,
        "diff": {
            "id": diff.diff_id,
            "status": "compared",
            "left_frame_index": target_frame,
            "right_frame_index": target_frame,
            "metrics": {
                "total_pixel_count": metrics.total_pixel_count,
                "changed_pixel_count": metrics.changed_pixel_count,
                "changed_pixel_ratio": metrics.changed_pixel_ratio,
                "max_delta": metrics.max_delta,
                "mean_delta": metrics.mean_delta,
            },
            "mismatch_samples": mismatch_samples,
        },
        "temporal_reconstruction": temporal,
        "gpu": temporal_quality::temporal_quality_gpu_evidence(gfx.adapter_facts()),
    });
    let root = config
        .output_root()
        .expect("active comparison has output root");
    fs::create_dir_all(root)
        .with_context(|| format!("create comparison evidence directory {}", root.display()))?;
    fs::write(
        root.join("comparison-evidence.json"),
        serde_json::to_vec_pretty(&evidence)?,
    )
    .context("write Render Lab comparison evidence")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comparison_temporal_evidence_state_retains_the_full_product_bound() {
        let mut state = RenderLabComparisonEvidenceState::default();
        for frame_index in 0..=RL2_MEASUREMENT_HISTORY_CAPACITY as u64 {
            state.observe_temporal_frame(frame_index, Vec::new());
        }

        assert!(state.temporal_frame(0).is_none());
        assert!(
            state
                .temporal_frame(RL2_MEASUREMENT_HISTORY_CAPACITY as u64)
                .is_some()
        );
        assert_eq!(
            state.temporal_by_frame.len(),
            RL2_MEASUREMENT_HISTORY_CAPACITY
        );
    }
}
