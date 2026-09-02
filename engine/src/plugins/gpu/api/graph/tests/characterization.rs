use super::super::{
    GpuPreparedWorkGraph, GpuWorkFragment,
    composition::{
        bind_imports, collect_output_bindings, topological_fragment_order,
        validate_boundary_access_intents,
    },
    coverage::{canonical_storage_resource, storage_identity},
    hazards::{
        DependencyEdges, add_explicit_orders, infer_cross_fragment_hazards, infer_fragment_hazards,
        topological_node_order,
    },
    identity::GpuPreparedWorkNodeId,
    initial_content::derive_prepared_initial_content,
    initialization::{simulate_prepared_initialization, validate_fragment_initialization},
};
use crate::plugins::gpu::{GpuResourceLabel, GpuResourceRef, GpuWorkResourceId};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    hint::black_box,
    num::NonZeroU64,
    path::PathBuf,
    time::{Duration, Instant},
};

#[path = "../../../../../../tests/gpu_reaction_diffusion_native/workload.rs"]
mod retained;

const REPORT_SCHEMA_VERSION: u32 = 1;
const WARMUP_SAMPLES: usize = 2;
const MEASURED_SAMPLES: usize = 5;

#[derive(Debug, Clone)]
struct TimingSummary {
    samples_us: Vec<f64>,
    min_us: f64,
    median_us: f64,
    max_us: f64,
}

impl TimingSummary {
    fn to_json(&self) -> Value {
        json!({
            "samples_us": self.samples_us,
            "min_us": self.min_us,
            "median_us": self.median_us,
            "max_us": self.max_us,
        })
    }
}

#[derive(Debug)]
struct EnvelopeEvidence {
    name: &'static str,
    node_count: usize,
    resource_count: usize,
    access_count: usize,
    dependency_count: usize,
    readback_count: usize,
    timings: BTreeMap<&'static str, TimingSummary>,
}

impl EnvelopeEvidence {
    fn median(&self, phase: &'static str) -> f64 {
        self.timings.get(phase).unwrap().median_us
    }

    fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "structure": {
                "node_count": self.node_count,
                "resource_count": self.resource_count,
                "access_count": self.access_count,
                "dependency_count": self.dependency_count,
                "readback_count": self.readback_count,
            },
            "timings": self.timings.iter().map(|(phase, summary)| {
                ((*phase).to_owned(), summary.to_json())
            }).collect::<BTreeMap<_, _>>(),
        })
    }
}

fn label(value: &str) -> GpuResourceLabel {
    GpuResourceLabel::new(value).unwrap()
}

fn micros(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000_000.0
}

fn summarize(mut samples_us: Vec<f64>) -> TimingSummary {
    assert_eq!(samples_us.len(), MEASURED_SAMPLES);
    assert!(
        samples_us
            .iter()
            .all(|sample| sample.is_finite() && *sample >= 0.0)
    );
    let mut ordered = samples_us.clone();
    ordered.sort_by(f64::total_cmp);
    TimingSummary {
        min_us: ordered[0],
        median_us: ordered[ordered.len() / 2],
        max_us: *ordered.last().unwrap(),
        samples_us: std::mem::take(&mut samples_us),
    }
}

fn measure_with_setup<S, R>(
    mut setup: impl FnMut() -> S,
    mut operation: impl FnMut(S) -> R,
) -> TimingSummary {
    for _ in 0..WARMUP_SAMPLES {
        black_box(operation(setup()));
    }
    let mut samples = Vec::with_capacity(MEASURED_SAMPLES);
    for _ in 0..MEASURED_SAMPLES {
        let state = setup();
        let started = Instant::now();
        let outcome = operation(state);
        let elapsed = started.elapsed();
        black_box(&outcome);
        samples.push(micros(elapsed));
        drop(outcome);
    }
    summarize(samples)
}

fn prepared_node_locations(
    fragments: &[GpuWorkFragment],
) -> BTreeMap<GpuPreparedWorkNodeId, (usize, usize)> {
    let mut locations = BTreeMap::new();
    for (fragment_index, fragment) in fragments.iter().enumerate() {
        let ordinal = u32::try_from(fragment_index).unwrap();
        for (node_index, node) in fragment.nodes().iter().enumerate() {
            let local = NonZeroU64::new(node.id().diagnostic_local()).unwrap();
            assert!(
                locations
                    .insert(
                        GpuPreparedWorkNodeId::new(ordinal, local),
                        (fragment_index, node_index),
                    )
                    .is_none()
            );
        }
    }
    locations
}

fn normalized_storage_resources(
    fragments: &[GpuWorkFragment],
) -> BTreeMap<GpuWorkResourceId, GpuResourceRef> {
    let mut storage = BTreeMap::new();
    for fragment in fragments {
        for resource in fragment.resources() {
            let canonical = canonical_storage_resource(resource);
            let identity = storage_identity(&canonical);
            storage.entry(identity).or_insert(canonical);
        }
    }
    storage
}

fn measure_envelope(
    sources: &retained::ProgramSources,
    envelope: retained::Envelope,
) -> EnvelopeEvidence {
    let (graph_label, fragment, readback_ids) = retained::offscreen_work(sources, envelope);
    let graph_resource_label = label(&graph_label);
    let fragments = vec![fragment];
    let node_count = fragments[0].nodes().len();
    let resource_count = fragments[0].resources().len();
    let access_count = fragments[0]
        .nodes()
        .iter()
        .map(|node| node.accesses().len())
        .sum::<usize>();

    let prepared =
        GpuPreparedWorkGraph::prepare(graph_resource_label.clone(), fragments.clone()).unwrap();
    assert_eq!(prepared.nodes().len(), node_count);
    assert_eq!(prepared.topological_order().len(), node_count);
    assert_eq!(
        readback_ids.len(),
        usize::try_from(envelope.frames).unwrap()
    );

    let node_locations = prepared_node_locations(&fragments);
    let storage_resources = normalized_storage_resources(&fragments);
    let output_bindings = collect_output_bindings(&graph_label, &fragments).unwrap();
    let (import_bindings, relations) =
        bind_imports(&graph_label, &fragments, &output_bindings).unwrap();
    validate_boundary_access_intents(&graph_label, &fragments).unwrap();
    let fragment_order =
        topological_fragment_order(&graph_label, &fragments, &import_bindings).unwrap();
    let initial_content = derive_prepared_initial_content(&graph_label, &fragments).unwrap();

    let mut dependency_edges = DependencyEdges::new();
    infer_fragment_hazards(&graph_label, &fragments, &mut dependency_edges).unwrap();
    infer_cross_fragment_hazards(&graph_label, &fragments, &relations, &mut dependency_edges)
        .unwrap();
    add_explicit_orders(&graph_label, &fragments, &mut dependency_edges).unwrap();
    assert_eq!(dependency_edges.len(), prepared.dependencies().len());
    let topological_order =
        topological_node_order(&graph_label, &fragments, &node_locations, &dependency_edges)
            .unwrap();
    assert_eq!(topological_order, prepared.topological_order());

    validate_fragment_initialization(
        &graph_label,
        &fragments,
        &fragment_order,
        &import_bindings,
        &initial_content,
    )
    .unwrap();
    let (initialization, _) = simulate_prepared_initialization(
        &graph_label,
        &fragments,
        &storage_resources,
        &node_locations,
        &topological_order,
        &initial_content,
    )
    .unwrap();
    assert_eq!(initialization, prepared.initialization());

    let mut timings = BTreeMap::new();
    timings.insert(
        "authoring",
        measure_with_setup(|| (), |_| retained::offscreen_work(sources, envelope)),
    );
    timings.insert(
        "canonical_prepare",
        measure_with_setup(
            || (graph_resource_label.clone(), fragments[0].clone()),
            |(graph_label, fragment)| {
                GpuPreparedWorkGraph::prepare(graph_label, [fragment]).unwrap()
            },
        ),
    );
    timings.insert(
        "composition_and_fragment_order",
        measure_with_setup(
            || (),
            |_| {
                let outputs = collect_output_bindings(&graph_label, black_box(&fragments)).unwrap();
                let (imports, relations) = bind_imports(&graph_label, &fragments, &outputs).unwrap();
                validate_boundary_access_intents(&graph_label, &fragments).unwrap();
                let order = topological_fragment_order(&graph_label, &fragments, &imports).unwrap();
                (outputs, imports, relations, order)
            },
        ),
    );
    timings.insert(
        "dependency_derivation",
        measure_with_setup(DependencyEdges::new, |mut edges| {
            infer_fragment_hazards(&graph_label, black_box(&fragments), &mut edges).unwrap();
            infer_cross_fragment_hazards(&graph_label, &fragments, &relations, &mut edges).unwrap();
            add_explicit_orders(&graph_label, &fragments, &mut edges).unwrap();
            edges
        }),
    );
    timings.insert(
        "node_topological_order",
        measure_with_setup(
            || (),
            |_| {
                topological_node_order(
                    &graph_label,
                    black_box(&fragments),
                    &node_locations,
                    &dependency_edges,
                )
                .unwrap()
            },
        ),
    );
    timings.insert(
        "prepared_initial_content_derivation",
        measure_with_setup(
            || (),
            |_| derive_prepared_initial_content(&graph_label, black_box(&fragments)).unwrap(),
        ),
    );
    timings.insert(
        "fragment_initialization_validation",
        measure_with_setup(
            || (),
            |_| {
                validate_fragment_initialization(
                    &graph_label,
                    black_box(&fragments),
                    &fragment_order,
                    &import_bindings,
                    &initial_content,
                )
                .unwrap()
            },
        ),
    );
    timings.insert(
        "prepared_initialization_simulation",
        measure_with_setup(
            || (),
            |_| {
                simulate_prepared_initialization(
                    &graph_label,
                    black_box(&fragments),
                    &storage_resources,
                    &node_locations,
                    &topological_order,
                    &initial_content,
                )
                .unwrap()
            },
        ),
    );

    EnvelopeEvidence {
        name: envelope.name,
        node_count,
        resource_count,
        access_count,
        dependency_count: prepared.dependencies().len(),
        readback_count: readback_ids.len(),
        timings,
    }
}

fn ratio(larger: &EnvelopeEvidence, smaller: &EnvelopeEvidence, phase: &'static str) -> f64 {
    let denominator = smaller.median(phase);
    if denominator == 0.0 {
        0.0
    } else {
        larger.median(phase) / denominator
    }
}

fn artifact_path() -> PathBuf {
    std::env::var_os("RUNEN_GPU_PROOF_ARTIFACT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/runengpu-proof-artifacts"))
        .join("graph-preparation-phases")
        .join("report.json")
}

#[test]
#[ignore = "retained #393 graph-preparation phase characterization; executed by RunenGPU Conformance CI"]
fn graph_preparation_phase_characterization_retains_report() {
    let revision = std::env::var("RUNEN_GPU_PROOF_REVISION")
        .expect("retained #393 characterization must declare the exact repository revision");
    assert!(!revision.trim().is_empty());
    let _surface_work_remains_compiled = retained::surface_work;

    let sources = retained::admitted_sources();
    let evidence = retained::ENVELOPES
        .into_iter()
        .map(|envelope| measure_envelope(&sources, envelope))
        .collect::<Vec<_>>();
    assert_eq!(evidence.len(), 2);
    let smaller = &evidence[0];
    let larger = &evidence[1];

    let phases = [
        "authoring",
        "canonical_prepare",
        "composition_and_fragment_order",
        "dependency_derivation",
        "node_topological_order",
        "prepared_initial_content_derivation",
        "fragment_initialization_validation",
        "prepared_initialization_simulation",
    ];
    let scaling = phases
        .into_iter()
        .map(|phase| (phase.to_owned(), ratio(larger, smaller, phase)))
        .collect::<BTreeMap<_, _>>();

    let report = json!({
        "schema_version": REPORT_SCHEMA_VERSION,
        "authority": "dornglut/runenwerk#393",
        "subject": "RunenGPU retained reaction-diffusion host-cost phase characterization",
        "repository_revision": revision,
        "runner": {
            "os": std::env::var("RUNNER_OS").ok(),
            "arch": std::env::var("RUNNER_ARCH").ok(),
        },
        "measurement_policy": {
            "cargo_test_profile": "release",
            "warmup_samples_per_phase": WARMUP_SAMPLES,
            "measured_samples_per_phase": MEASURED_SAMPLES,
            "hosted_ci_timing_is_characterization_only": true,
            "performance_pass_fail_threshold": null,
            "setup_for_each_phase_is_excluded_from_its_timed_interval": true,
            "result_destruction_is_excluded_from_each_timed_interval": true,
            "phase_samples_are_independent_and_not_an_additive_timing_budget": true,
        },
        "measurement_boundary": {
            "canonical_prepare": "GpuPreparedWorkGraph::prepare on the retained G6-I01 authored fragment",
            "subphases": "exact existing graph-internal semantic functions invoked from cfg(test) evidence; no copied preparation implementation and no public timing API",
            "authoring": "retained G6-I01 offscreen_work construction with admitted sources reused across samples",
            "unisolated_inside_canonical_prepare": [
                "resource registration and work-node identity/resource validation",
                "graph-wide capability requirement merging",
                "dependency/output/diagnostic result materialization"
            ],
            "cross_fragment_note": "the retained G6-I01 workload is one fragment; cross-fragment hazard traversal has no candidate pairs here"
        },
        "source_hypotheses_under_test": {
            "authoring_transaction_snapshot": "ordinary lexical operation authoring clones current builder transaction state before commit; source observation only, not a correction authorization",
            "initialization_work": "canonical preparation performs both fragment initialization validation and prepared topological initialization simulation; both are accepted semantic work unless evidence selects a later correction"
        },
        "larger_over_smaller_median_ratio": scaling,
        "envelopes": evidence.iter().map(EnvelopeEvidence::to_json).collect::<Vec<_>>(),
    });

    let path = artifact_path();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    assert!(path.metadata().unwrap().len() > 0);
}
