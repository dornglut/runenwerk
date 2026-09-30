use super::*;
use engine::plugins::render::inspect::{RenderTextureDiffResult, RenderTextureDiffStatus};
use serde_json::{Value, json};

const COMPARISON_DIFF_ID: &str = "runenwerk.render_lab.rl2.compare.a_vs_b";

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
    gfx: &engine::plugins::render::Gfx,
    target_frame: u64,
    output_size_px: (u32, u32),
    candidate_size_px: (u32, u32),
) -> Result<Vec<Value>> {
    let mut generation_classes = Vec::new();
    let mut frames = Vec::new();
    for observation in history.observations() {
        let frame = observation.key.frame_index;
        let records = gfx.deterministic_temporal_evidence(frame);
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
        let record_json = |record: &engine::plugins::render::deterministic_execution::RenderDeterministicTemporalExecutionEvidence| json!({
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
        });
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

pub(super) fn write_comparison_evidence(
    config: &RenderLabComparisonEvidenceConfig,
    history: &RenderFrameHistoryState,
    debug_report: &RenderDebugFrameReportState,
    gfx: &engine::plugins::render::Gfx,
    target_frame: u64,
    output_size_px: (u32, u32),
    candidate_size_px: (u32, u32),
) -> Result<()> {
    let diffs = debug_report
        .texture_diff_results_for_frame(target_frame)
        .ok_or_else(|| anyhow::anyhow!("comparison target frame has no retained texture diff"))?;
    let diff = compared_diff(diffs, target_frame, output_size_px)?;
    let temporal = temporal_frames(
        history,
        gfx,
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
    let captures = config.capture_selectors().iter().map(|selector| {
        let result = results.iter().find(|result| &result.selector == selector)
            .ok_or_else(|| anyhow::anyhow!("comparison target report lost a capture selector"))?;
        Ok(json!({
            "resource_id": selector.resource_id,
            "frame_index": target_frame,
            "artifact_path": result.artifact_path.as_ref().expect("validated capture path").display().to_string(),
        }))
    }).collect::<Result<Vec<Value>>>()?;
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
        "scenario_id": "rl2-native-comparison",
        "source_git_revision": std::env::var("RUNENWERK_SOURCE_REVISION").ok(),
        "output_size_px": [output_size_px.0, output_size_px.1],
        "reference": {"identity": "native/P100", "internal_size_px": [output_size_px.0, output_size_px.1]},
        "candidate": {"identity": "accepted-static-footprint-temporal", "internal_size_px": [candidate_size_px.0, candidate_size_px.1]},
        "target_frame_index": target_frame,
        "capture_submission_ordinal": temporal.len(),
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
