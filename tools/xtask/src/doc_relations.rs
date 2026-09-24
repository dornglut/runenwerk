use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use yaml_serde::Value;

const DOCS_ROOT: &str = "docs-site/src/content/docs";
const RELATION_KEYS: &[&str] = &[
    "related",
    "related_docs",
    "related_designs",
    "related_adrs",
    "related_roadmaps",
    "related_reports",
    "superseded_by",
    "replaced_by",
];
const NAVIGATION_KEYS: &[&str] = &[
    "related",
    "related_docs",
    "related_designs",
    "related_adrs",
    "related_roadmaps",
    "related_reports",
];

pub(crate) fn validate(root: &Path) -> Result<(), String> {
    let docs_root = root.join(DOCS_ROOT);
    let mut documents = Vec::new();
    collect_documents(&docs_root, root, &mut documents)?;

    let mut errors = Vec::new();
    for relative in documents {
        let path = root.join(&relative);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                errors.push(format!(
                    "{}: failed to read documentation source: {error}",
                    display_path(&relative)
                ));
                continue;
            }
        };

        match validate_document(root, &relative, &text) {
            Ok(()) => {}
            Err(error) => errors.push(error),
        }
    }

    if errors.is_empty() {
        eprintln!("> documentation relation validation passed");
        Ok(())
    } else {
        errors.sort();
        Err(format!(
            "documentation relation validation failed:\n{}",
            errors.join("\n")
        ))
    }
}

fn collect_documents(
    directory: &Path,
    root: &Path,
    documents: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = fs::read_dir(directory).map_err(|error| {
        format!(
            "documentation relation validation: failed to inspect {}: {error}",
            directory.display()
        )
    })?;

    let mut entries = entries.collect::<Result<Vec<_>, _>>().map_err(|error| {
        format!(
            "documentation relation validation: failed to enumerate {}: {error}",
            directory.display()
        )
    })?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_documents(&path, root, documents)?;
        } else if path.is_file()
            && matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("md" | "mdx")
            )
        {
            let relative = path.strip_prefix(root).map_err(|error| {
                format!(
                    "documentation relation validation: failed to relativize {}: {error}",
                    path.display()
                )
            })?;
            documents.push(relative.to_path_buf());
        }
    }

    Ok(())
}

fn validate_document(root: &Path, relative: &Path, text: &str) -> Result<(), String> {
    let frontmatter = extract_frontmatter(text)
        .map_err(|error| format!("{}: {error}", display_path(relative)))?;
    let value: Value = yaml_serde::from_str(&frontmatter).map_err(|error| {
        format!(
            "{}: malformed YAML frontmatter: {error}",
            display_path(relative)
        )
    })?;
    let mapping = match value {
        Value::Mapping(mapping) => mapping,
        other => {
            return Err(format!(
                "{}: frontmatter must be a mapping, found {}",
                display_path(relative),
                value_shape(&other)
            ));
        }
    };
    let docs_relative = relative.strip_prefix(DOCS_ROOT).unwrap_or(relative);
    let historical = is_historical_source(docs_relative, &mapping);
    let mut errors = Vec::new();

    for key in RELATION_KEYS {
        let Some(value) = mapping.get(Value::String((*key).to_owned())) else {
            continue;
        };
        let values = match relation_values(value) {
            Ok(values) => values,
            Err(shape) => {
                errors.push(format!(
                    "{}: {key}: unsupported relation shape {shape}; expected a string or list of strings",
                    display_path(relative)
                ));
                continue;
            }
        };

        for raw in values {
            let resolution = resolve_relation(root, relative, raw);
            if let RelationResolution::Unsupported(reason) = &resolution {
                errors.push(format!(
                    "{}: {key} value {raw:?}: unsupported non-local relation ({reason})",
                    display_path(relative)
                ));
                continue;
            }
            if matches!(resolution, RelationResolution::EscapesRepository) {
                errors.push(format!(
                    "{}: {key} value {raw:?}: relation path escapes the repository root",
                    display_path(relative)
                ));
                continue;
            }

            let must_resolve = key == &"superseded_by"
                || key == &"replaced_by"
                || (!historical && NAVIGATION_KEYS.contains(key));
            if must_resolve && matches!(resolution, RelationResolution::Missing(_)) {
                let RelationResolution::Missing(target) = resolution else {
                    unreachable!("resolution was checked above");
                };
                errors.push(format!(
                    "{}: {key} value {raw:?}: relation target does not exist at {}",
                    display_path(relative),
                    display_path(&target)
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        Err(errors.join("\n"))
    }
}

fn extract_frontmatter(text: &str) -> Result<String, &'static str> {
    let mut lines = text.lines();
    if lines.next() != Some("---") {
        return Err("missing YAML frontmatter opening delimiter");
    }

    let mut frontmatter = String::new();
    for line in lines {
        if line == "---" {
            return Ok(frontmatter);
        }
        frontmatter.push_str(line);
        frontmatter.push('\n');
    }

    Err("unterminated YAML frontmatter")
}

fn relation_values(value: &Value) -> Result<Vec<&str>, String> {
    match value {
        Value::String(value) => Ok(vec![value]),
        Value::Sequence(values) => values
            .iter()
            .map(|value| match value {
                Value::String(value) => Ok(value.as_str()),
                other => Err(value_shape(other)),
            })
            .collect(),
        other => Err(value_shape(other)),
    }
}

fn value_shape(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(_) => "boolean".to_owned(),
        Value::Number(_) => "number".to_owned(),
        Value::String(_) => "string".to_owned(),
        Value::Sequence(_) => "list".to_owned(),
        Value::Mapping(_) => "mapping".to_owned(),
        _ => "unsupported YAML value".to_owned(),
    }
}

fn is_historical_source(relative: &Path, frontmatter: &yaml_serde::Mapping) -> bool {
    let path = display_path(relative);
    if path.starts_with("reports/")
        && relative.file_name().and_then(|name| name.to_str()) != Some("README.md")
    {
        return true;
    }
    if path.starts_with("archive/") {
        return true;
    }
    if path.starts_with("adr/superseded/")
        || path.starts_with("adr/rejected/")
        || path.starts_with("design/superseded/")
        || path.starts_with("design/rejected/")
        || path.starts_with("design/archived/")
    {
        return true;
    }

    matches!(
        frontmatter.get(Value::String("status".to_owned())),
        Some(Value::String(status)) if matches!(status.as_str(), "superseded" | "archived" | "rejected")
    )
}

#[derive(Debug, PartialEq, Eq)]
enum RelationResolution {
    Existing(PathBuf),
    Missing(PathBuf),
    EscapesRepository,
    Unsupported(&'static str),
}

fn resolve_relation(root: &Path, source: &Path, raw: &str) -> RelationResolution {
    let value = raw.trim();
    if value.is_empty() {
        return RelationResolution::Unsupported("empty value");
    }
    if value.starts_with('/') || Path::new(value).is_absolute() {
        return RelationResolution::Unsupported("absolute path");
    }
    if value.starts_with('#')
        || value.starts_with("//")
        || value.contains("://")
        || value
            .as_bytes()
            .windows(2)
            .any(|window| window[1] == b':' && window[0].is_ascii_alphabetic())
    {
        return RelationResolution::Unsupported("external or URI value");
    }

    let mut normalized = PathBuf::new();
    for component in source
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .components()
        .chain(Path::new(value).components())
    {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    return RelationResolution::EscapesRepository;
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return RelationResolution::Unsupported("absolute path");
            }
        }
    }

    let path = root.join(&normalized);
    if path.is_file() {
        RelationResolution::Existing(normalized)
    } else {
        RelationResolution::Missing(normalized)
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::{DOCS_ROOT, RelationResolution, resolve_relation, validate};
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock must be after Unix epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "runenwerk-xtask-doc-relations-{name}-{}-{stamp}",
                std::process::id()
            ));
            fs::create_dir_all(root.join(DOCS_ROOT)).expect("create fixture docs root");
            Self { root }
        }

        fn document(&self, path: &str, frontmatter: &str) {
            let path = self.root.join(DOCS_ROOT).join(path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create fixture document directory");
            }
            fs::write(path, format!("---\n{frontmatter}\n---\n\n# Fixture\n"))
                .expect("write fixture document");
        }

        fn file(&self, path: &str) {
            let path = self.root.join(path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create fixture target directory");
            }
            fs::write(path, "target\n").expect("write fixture target");
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).expect("remove owned fixture directory");
        }
    }

    fn assert_passes(fixture: &Fixture) {
        assert_eq!(validate(&fixture.root), Ok(()));
    }

    fn assert_fails_with(fixture: &Fixture, expected: &str) {
        let error = validate(&fixture.root).expect_err("fixture should fail validation");
        assert!(
            error.contains(expected),
            "expected {expected:?} in diagnostic:\n{error}"
        );
    }

    #[test]
    fn broken_live_related_edge_fails() {
        let fixture = Fixture::new("broken-live-related");
        fixture.document("current.md", "related_docs:\n  - ./missing.md");
        assert_fails_with(&fixture, "current.md: related_docs");
    }

    #[test]
    fn valid_live_relation_passes_and_normalizes_path() {
        let fixture = Fixture::new("valid-live-related");
        fixture.document("current.md", "related: [./nested/../target.txt]");
        fixture.file("docs-site/src/content/docs/target.txt");
        assert_passes(&fixture);
    }

    #[test]
    fn stale_historical_report_relation_is_allowed() {
        let fixture = Fixture::new("historical-report");
        fixture.document(
            "reports/old-investigation.md",
            "status: completed\nrelated_docs: [../../deleted-at-the-time.md]",
        );
        assert_passes(&fixture);
    }

    #[test]
    fn maintained_report_index_relation_must_resolve() {
        let fixture = Fixture::new("report-index");
        fixture.document("reports/section/README.md", "related_docs: [./missing.md]");
        assert_fails_with(&fixture, "reports/section/README.md: related_docs");
    }

    #[test]
    fn broken_successor_edges_fail_for_historical_sources() {
        let fixture = Fixture::new("successor-edges");
        fixture.document("reports/old.md", "superseded_by: ./missing.md");
        fixture.document("archive/older.md", "replaced_by: [./missing.md]");
        assert_fails_with(&fixture, "superseded_by");
        assert_fails_with(&fixture, "replaced_by");
    }

    #[test]
    fn nested_closeout_provenance_is_not_live_navigation() {
        let fixture = Fixture::new("closeout-evidence");
        fixture.document(
            "reports/closeouts/example/closeout.md",
            "closeout_evidence:\n  files_changed: [./deleted.md]\n  closeout_path: ./deleted-closeout.md",
        );
        assert_passes(&fixture);
    }

    #[test]
    fn malformed_relation_shape_fails_usefully() {
        let fixture = Fixture::new("malformed-shape");
        fixture.document("current.md", "related_docs:\n  target: ./missing.md");
        assert_fails_with(&fixture, "unsupported relation shape mapping");
    }

    #[test]
    fn scalar_successor_shape_is_supported() {
        let fixture = Fixture::new("scalar-successor");
        fixture.document("old.md", "replaced_by: ./successor.txt");
        fixture.file("docs-site/src/content/docs/successor.txt");
        assert_passes(&fixture);
    }

    #[test]
    fn external_relation_values_are_rejected_explicitly() {
        let fixture = Fixture::new("external-relation");
        fixture.document("current.md", "related_docs: [https://example.invalid/docs]");
        assert_fails_with(&fixture, "unsupported non-local relation");
    }

    #[test]
    fn relation_paths_cannot_escape_repository_root() {
        let fixture = Fixture::new("path-escape");
        fixture.document("current.md", "related_docs: [../../../../../../outside.md]");
        assert_fails_with(&fixture, "escapes the repository root");
    }

    #[test]
    fn completed_sources_remain_live() {
        let fixture = Fixture::new("completed-live");
        fixture.document("completed.md", "status: completed\nrelated: [./missing.md]");
        assert_fails_with(&fixture, "completed.md: related");
    }

    #[test]
    fn repository_root_relations_can_resolve_current_authority_files() {
        let fixture = Fixture::new("repository-root");
        fixture.document(
            "workspace/ai-agent-boundaries.md",
            "related_docs: [../../../../../AGENTS.md]",
        );
        fixture.file("AGENTS.md");
        assert_passes(&fixture);
    }

    #[test]
    fn relation_resolution_reports_missing_and_escape_deterministically() {
        let root = Path::new("/repository");
        let source = Path::new("docs-site/src/content/docs/current.md");
        assert_eq!(
            resolve_relation(root, source, "./missing.md"),
            RelationResolution::Missing(PathBuf::from("docs-site/src/content/docs/missing.md"))
        );
        assert_eq!(
            resolve_relation(root, source, "../../../../../../outside.md"),
            RelationResolution::EscapesRepository
        );
    }
}
