use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_rust_sources(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|error| panic!("cannot read {}: {error}", root.display()))
    {
        let path = entry.expect("source entry should be readable").path();
        if path.is_dir() {
            collect_rust_sources(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
}

fn compact(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            !line.starts_with("//") && !line.starts_with("* ") && !line.starts_with("*/")
        })
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn read(manifest: &Path, relative: &str) -> String {
    fs::read_to_string(manifest.join(relative))
        .unwrap_or_else(|error| panic!("cannot read {relative}: {error}"))
}

#[test]
fn g5b_completion_registration_stays_in_one_serialized_queue_interval() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let backend_root = manifest.join("src/plugins/gpu/backend/wgpu");
    let execution_path = "src/plugins/gpu/backend/wgpu/execution.rs";

    let mut paths = Vec::new();
    collect_rust_sources(&backend_root, &mut paths);
    let direct_submit_paths = paths
        .into_iter()
        .filter_map(|path| {
            let source =
                compact(&fs::read_to_string(&path).expect("WGPU source should be readable"));
            source.contains(".queue.submit(").then(|| {
                path.strip_prefix(&manifest)
                    .expect("WGPU source stays in engine")
                    .to_string_lossy()
                    .into_owned()
            })
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        direct_submit_paths,
        BTreeSet::from([execution_path.to_owned()]),
        "direct private WGPU queue submission must remain owned by the G5B executor; the residual renderer receives only a gated queue loan"
    );

    let execution = compact(&read(&manifest, execution_path));
    let gate = execution
        .find("let_attribution_gate=backend.error_attribution_gate.acquire();")
        .expect("G5B physical submission must acquire the shared backend-operation gate");
    let submit = execution
        .find("backend.queue.submit([encoder.finish()]);")
        .expect("G5B physical submission must submit the encoded command buffer");
    let attach = execution
        .find("execution.attach_staging(submission,&encoded)?;")
        .expect("accepted staging must be published before callbacks can observe it");
    let callbacks = execution
        .find("register_callbacks(execution,backend,submission,&encoded);")
        .expect(
            "submission and readback callbacks must be registered before the gate interval ends",
        );
    assert!(
        gate < submit && submit < attach && attach < callbacks,
        "one shared gate interval must own submit -> staging publication -> callback registration"
    );

    let current_host = compact(&read(
        &manifest,
        "src/plugins/gpu/backend/wgpu/current_host.rs",
    ));
    assert!(
        current_host
            .contains("_error_attribution_gate:self.backend.error_attribution_gate.acquire()"),
        "the residual renderer queue loan must serialize against G5B through the same gate"
    );
}

#[test]
fn g5b_rejection_categories_and_submission_identity_preserve_acceptance_order() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let execution = compact(&read(
        &manifest,
        "src/plugins/gpu/backend/wgpu/execution.rs",
    ));

    let submit_start = execution
        .find("pubfnsubmit_prepared(")
        .expect("public prepared submission entrypoint must remain explicit");
    let submit_end = execution[submit_start..]
        .find("pubfnprogress(")
        .map(|offset| submit_start + offset)
        .expect("submission entrypoint must end before progress");
    let submit = &execution[submit_start..submit_end];
    let foreign = submit
        .find("prepared.affinity.context()!=expected_affinity.context()")
        .expect("foreign-context rejection must be explicit");
    let stale = submit
        .find("prepared.affinity.generation()!=expected_affinity.generation()")
        .expect("stale-generation rejection must be explicit");
    let owner = submit
        .find(".ptr_eq(&Arc::downgrade(&self.backend.execution))")
        .expect("same-affinity prepared ownership must still be checked");
    assert!(
        foreign < stale && stale < owner,
        "context and generation classification must precede private execution-owner identity"
    );

    let accept_start = execution
        .find("fnaccept_prepared(")
        .expect("private acceptance owner must remain explicit");
    let accept_end = execution[accept_start..]
        .find("fnattach_staging(")
        .map(|offset| accept_start + offset)
        .expect("acceptance must end before staging attachment");
    let accept = &execution[accept_start..accept_end];
    let metadata = accept
        .find("ifreadbacks.len()!=plan.readback_ids.len()")
        .expect("readback metadata completeness must be validated pre-acceptance");
    let remove_prepared = accept
        .find("inner.prepared.remove(&prepared.ticket)")
        .expect("irreversible acceptance must remove retryable prepared authority");
    let allocate_id = accept
        .find("allocate_nonzero(&self.next_submission)")
        .expect("submission identity must be allocated only at irreversible acceptance");
    assert!(
        metadata < remove_prepared && remove_prepared < allocate_id,
        "all rollback-capable metadata validation must precede prepared removal and submission-ID publication"
    );
}
