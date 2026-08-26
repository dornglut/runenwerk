use std::{collections::BTreeSet, fs, path::Path};

#[test]
fn workflow_inventory_stays_deliberate() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask manifest must live at <repository>/tools/xtask");
    let workflows = root.join(".github/workflows");

    let found = fs::read_dir(&workflows)
        .expect("workflow directory should exist")
        .map(|entry| entry.expect("workflow entry should be readable").path())
        .filter(|path| {
            path.is_file()
                && matches!(
                    path.extension().and_then(|value| value.to_str()),
                    Some("yml" | "yaml")
                )
        })
        .map(|path| {
            path.strip_prefix(root)
                .expect("workflow path should be repository-relative")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<BTreeSet<_>>();

    let expected = BTreeSet::from([
        ".github/workflows/ci.yml".to_owned(),
        ".github/workflows/docs-validation.yml".to_owned(),
        ".github/workflows/runengpu-native-conformance.yml".to_owned(),
    ]);

    assert_eq!(found, expected, "workflow inventory should stay intentional");
}
