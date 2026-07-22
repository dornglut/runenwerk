#![forbid(unsafe_code)]

use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

const TOOLING_CARGO_STEPS: &[&[&str]] = &[
    &[
        "fmt",
        "--manifest-path",
        "tools/xtask/Cargo.toml",
        "--check",
    ],
    &[
        "test",
        "--manifest-path",
        "tools/xtask/Cargo.toml",
        "--locked",
    ],
    &[
        "clippy",
        "--manifest-path",
        "tools/xtask/Cargo.toml",
        "--all-targets",
        "--locked",
        "--",
        "-D",
        "warnings",
    ],
];

const PRODUCT_CARGO_STEPS: &[&[&str]] = &[
    &["fmt", "--all", "--check"],
    &["test", "--workspace", "--locked"],
    &[
        "clippy",
        "--workspace",
        "--all-targets",
        "--locked",
        "--",
        "-D",
        "warnings",
    ],
];

const RETIRED_PATHS: &[&str] = &[
    "Taskfile.yml",
    "tools/workflow",
    "workflow",
    "workflow.cmd",
    "quiet_editor_gate.sh",
    "quiet_full_gate.sh",
    ".github/workflows/runensdf-transfer-artifact.yml",
    "docs-site/src/content/docs/workspace/execution-contract-packs",
    "docs-site/src/content/docs/workspace/execution-locks",
    "docs-site/src/content/docs/workspace/track-execution-manifests",
    "docs-site/src/content/docs/workspace/truth-conformance-specs",
    "docs-site/src/content/docs/reports/track-execution-manifests",
    "docs-site/src/content/docs/reports/track-execution-runs",
    "docs-site/src/content/docs/reports/truth-certificates",
    "docs-site/src/content/docs/workspace/roadmap-items.yaml",
    "docs-site/src/content/docs/workspace/roadmap-archive.yaml",
    "docs-site/src/content/docs/workspace/roadmap-deferred.yaml",
    "docs-site/src/content/docs/workspace/production-tracks.yaml",
];

fn main() -> ExitCode {
    let command = env::args().nth(1).unwrap_or_else(|| "help".to_owned());

    let result = match command.as_str() {
        "validate" => validate(),
        "docs" => repository_root().and_then(|root| validate_docs(&root)),
        "audit" => repository_root().and_then(|root| audit_repository(&root)),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        other => Err(format!("unknown xtask command: {other}")),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn validate() -> Result<(), String> {
    let root = repository_root()?;

    for args in TOOLING_CARGO_STEPS {
        run(&root, "cargo", args)?;
    }

    for args in PRODUCT_CARGO_STEPS {
        run(&root, "cargo", args)?;
    }

    validate_docs(&root)?;
    audit_repository(&root)
}

fn validate_docs(root: &Path) -> Result<(), String> {
    let script = "tools/docs/validate_docs.py";
    let candidates: &[(&str, &[&str])] = &[
        ("python3", &[script]),
        ("python", &[script]),
        ("py", &["-3", script]),
    ];

    let mut unavailable = Vec::new();
    for (program, args) in candidates {
        match run_status(root, program, args) {
            Ok(true) => return Ok(()),
            Ok(false) => {
                return Err(format!(
                    "documentation validation failed: {} {}",
                    program,
                    args.join(" ")
                ));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => unavailable.push(*program),
            Err(error) => return Err(format!("failed to run {program}: {error}")),
        }
    }

    Err(format!(
        "documentation validation requires Python 3; unavailable commands: {}",
        unavailable.join(", ")
    ))
}

fn audit_repository(root: &Path) -> Result<(), String> {
    for required in [
        "Cargo.toml",
        "Cargo.lock",
        ".cargo/config.toml",
        ".github/workflows/ci.yml",
        ".github/workflows/docs-validation.yml",
        "README.md",
        "AGENTS.md",
        "ARCHITECTURE.md",
        "TESTING.md",
        "tools/checks/ux_lab_terminology.py",
        "docs-site/src/content/docs/workspace/engineering-workflow.md",
        "docs-site/src/content/docs/workspace/documentation-structure.md",
        "docs-site/src/content/docs/workspace/planning/roadmap.md",
        "docs-site/src/content/docs/guidelines/dependency-rules.md",
    ] {
        if !root.join(required).is_file() {
            return Err(format!("repository audit: missing required file {required}"));
        }
    }

    for retired in RETIRED_PATHS {
        if root.join(retired).exists() {
            return Err(format!(
                "repository audit: retired workflow path must not exist: {retired}"
            ));
        }
    }

    require_text(
        root,
        ".cargo/config.toml",
        "validate = \"run --manifest-path tools/xtask/Cargo.toml --locked -- validate\"",
        "the root Cargo alias must own the canonical baseline",
    )?;
    require_text(
        root,
        ".github/workflows/ci.yml",
        "run: cargo validate",
        "CI must invoke the same baseline as local development",
    )?;
    require_text(
        root,
        "AGENTS.md",
        "cargo validate",
        "the agent entrypoint must name the canonical baseline",
    )?;
    require_text(
        root,
        "docs-site/src/content/docs/workspace/engineering-workflow.md",
        "GitHub issues and pull requests manage work",
        "the canonical workflow must use ordinary repository artifacts",
    )?;

    forbid_text(
        root,
        ".github/workflows/docs-validation.yml",
        "validate_docs.py",
        "the path-scoped docs build must not duplicate baseline documentation validation",
    )?;

    eprintln!("> repository audit passed");
    Ok(())
}

fn require_text(root: &Path, relative: &str, marker: &str, reason: &str) -> Result<(), String> {
    let text = read_text(root, relative)?;
    if text.contains(marker) {
        Ok(())
    } else {
        Err(format!(
            "repository audit: {relative} is missing required marker {marker:?}: {reason}"
        ))
    }
}

fn forbid_text(root: &Path, relative: &str, marker: &str, reason: &str) -> Result<(), String> {
    let text = read_text(root, relative)?;
    if text.contains(marker) {
        Err(format!(
            "repository audit: {relative} contains forbidden marker {marker:?}: {reason}"
        ))
    } else {
        Ok(())
    }
}

fn read_text(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative))
        .map_err(|error| format!("repository audit: failed to read {relative}: {error}"))
}

fn repository_root() -> Result<PathBuf, String> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "xtask must live at tools/xtask".to_owned())?;

    if root.join("Cargo.toml").is_file() {
        Ok(root.to_path_buf())
    } else {
        Err(format!(
            "resolved repository root does not contain Cargo.toml: {}",
            root.display()
        ))
    }
}

fn run(root: &Path, program: &str, args: &[&str]) -> Result<(), String> {
    eprintln!("> {program} {}", args.join(" "));
    match run_status(root, program, args) {
        Ok(true) => Ok(()),
        Ok(false) => Err(format!("{program} {} failed", args.join(" "))),
        Err(error) => Err(format!(
            "failed to run {program} {}: {error}",
            args.join(" ")
        )),
    }
}

fn run_status(root: &Path, program: &str, args: &[&str]) -> std::io::Result<bool> {
    Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map(|status| status.success())
}

fn print_usage() {
    eprintln!("Runenwerk repository tasks:");
    eprintln!("  cargo validate       required baseline");
    eprintln!("  cargo xtask docs     documentation validation only");
    eprintln!("  cargo xtask audit    deterministic repository audit only");
}
