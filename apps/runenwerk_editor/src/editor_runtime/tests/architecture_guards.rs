use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn legacy_history_workers_are_only_used_by_ratification_pipeline() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_root = crate_root.join("src");
    let mut files = Vec::new();
    collect_rust_files(&src_root, &mut files);

    assert_symbol_is_scoped(
        &crate_root,
        &files,
        "execute_scene_command_and_push_history_with_origin(",
        &[
            "src/editor_runtime/commands/scene_commands.rs",
            "src/editor_runtime/ratification/mod.rs",
        ],
    );
    assert_symbol_is_scoped(
        &crate_root,
        &files,
        "execute_scene_transaction_and_push_history_with_origin(",
        &[
            "src/editor_runtime/commands/transactions.rs",
            "src/editor_runtime/ratification/mod.rs",
        ],
    );
    assert_symbol_is_scoped(
        &crate_root,
        &files,
        "undo_last_scene_transaction_with_origin(",
        &[
            "src/editor_runtime/history/undo_redo.rs",
            "src/editor_runtime/ratification/mod.rs",
        ],
    );
    assert_symbol_is_scoped(
        &crate_root,
        &files,
        "redo_last_scene_transaction_with_origin(",
        &[
            "src/editor_runtime/history/undo_redo.rs",
            "src/editor_runtime/ratification/mod.rs",
        ],
    );
}

#[test]
fn editor_runtime_does_not_reintroduce_stringly_result_contracts() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_root = crate_root.join("src");
    let mut files = Vec::new();
    collect_rust_files(&src_root, &mut files);

    assert_pattern_absent(
        &crate_root,
        &files,
        "Result<(), &'static str>",
        &["src/editor_runtime/tests/architecture_guards.rs"],
    );
    assert_pattern_absent(
        &crate_root,
        &files,
        "Result<Option<&'static str>>",
        &["src/editor_runtime/tests/architecture_guards.rs"],
    );
}

fn assert_symbol_is_scoped(
    crate_root: &Path,
    files: &[PathBuf],
    symbol: &str,
    allowed_files: &[&str],
) {
    let offenders = files
        .iter()
        .filter_map(|path| {
            let source = fs::read_to_string(path).ok()?;
            if !source.contains(symbol) {
                return None;
            }

            let relative = path
                .strip_prefix(crate_root)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/");

            if relative == "src/editor_runtime/tests/architecture_guards.rs" {
                return None;
            }

            if allowed_files.iter().any(|allowed| relative == *allowed) {
                return None;
            }

            Some(relative)
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "symbol `{symbol}` is not allowed outside ratification pipeline; found in: {offenders:?}"
    );
}

fn assert_pattern_absent(
    crate_root: &Path,
    files: &[PathBuf],
    pattern: &str,
    allowed_files: &[&str],
) {
    let offenders = files
        .iter()
        .filter_map(|path| {
            let source = fs::read_to_string(path).ok()?;
            if !source.contains(pattern) {
                return None;
            }

            let relative = path
                .strip_prefix(crate_root)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/");

            if allowed_files.iter().any(|allowed| relative == *allowed) {
                return None;
            }

            Some(relative)
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "pattern `{pattern}` is forbidden in editor runtime surfaces; found in: {offenders:?}"
    );
}

fn collect_rust_files(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).expect("source directory should be readable");
    for entry in entries {
        let entry = entry.expect("directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}
