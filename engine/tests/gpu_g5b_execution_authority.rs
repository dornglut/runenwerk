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
fn g5b_mapping_and_completion_are_command_buffer_local_before_submit() {
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
    assert!(
        !execution.contains("backend.queue.on_submitted_work_done("),
        "G5B completion must not regress to queue-relative previous-submit ownership"
    );
    assert!(
        !execution.contains(".slice(..).map_async("),
        "G5B readback mapping must be deferred on the command buffer before submission"
    );
    assert!(execution.contains("command_buffer.map_buffer_on_submit("));
    assert!(execution.contains("command_buffer.on_submitted_work_done("));

    let gate = execution
        .find("let_attribution_gate=backend.error_attribution_gate.acquire();")
        .expect("G5B physical submission must retain the accepted backend-operation gate");
    let finish = execution
        .find("letcommand_buffer=encoder.finish();")
        .expect("G5B execution must finish one owned command buffer");
    let attach = execution
        .find("execution.attach_staging(submission,&encoded)?;")
        .expect("accepted staging must be published before submission");
    let callbacks = execution
        .find("register_callbacks(execution,submission,&encoded,&command_buffer);")
        .expect("mapping and completion callbacks must be command-buffer-local before submission");
    let submit = execution
        .find("backend.queue.submit([command_buffer]);")
        .expect("G5B physical submission must submit that exact command buffer");
    assert!(
        gate < finish && finish < attach && attach < callbacks && callbacks < submit,
        "one accepted command buffer must own staging publication and deferred callbacks before physical submit"
    );
}

#[test]
fn g5b_rejection_identity_and_owner_local_execution_order_preserve_acceptance() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let execution = compact(&read(
        &manifest,
        "src/plugins/gpu/backend/wgpu/execution.rs",
    ));

    assert!(
        execution.contains("submission_order:Mutex<()>"),
        "G5B execution state must retain a context-local acceptance/execution ordering gate"
    );

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
    let order_gate = submit
        .find("let_submission_order=self.backend.execution.submission_order.lock().unwrap_or_else(std::sync::PoisonError::into_inner);")
        .expect("owner-local submission ordering must begin before irreversible acceptance");
    let accept = submit
        .find("self.backend.execution.accept_prepared(&prepared)")
        .expect("irreversible acceptance must remain explicit");
    let encode_submit = submit
        .find("encode_submit_and_register_buffers(")
        .expect("accepted physical execution must remain in the same submit entrypoint");
    assert!(
        foreign < stale
            && stale < owner
            && owner < order_gate
            && order_gate < accept
            && accept < encode_submit,
        "classification must precede one owner-local interval spanning irreversible acceptance through physical execution"
    );
    assert!(
        !submit.contains("drop(_submission_order)"),
        "the owner-local execution-order gate must remain held through physical encode/submit"
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
