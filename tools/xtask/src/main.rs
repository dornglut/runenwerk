#![forbid(unsafe_code)]

use std::{
    env,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

const BASELINE_CARGO_STEPS: &[&[&str]] = &[
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

const EXTENDED_STEPS: &[(&str, &[&str])] = &[
    ("cargo", &["deny", "check"]),
    ("cargo", &["machete", "--skip-target-dir"]),
    ("lychee", &["docs-site/src/content/docs"]),
    ("ast-grep", &["scan"]),
    ("pnpm", &["--dir", "docs-site", "build"]),
];

const RETIRED_PATHS: &[&str] = &[
    "tools/workflow",
    "pyproject.toml",
    "uv.lock",
    "workflow",
    "workflow.cmd",
    "quiet_editor_gate.sh",
    "quiet_full_gate.sh",
    ".github/workflows/runensdf-transfer-artifact.yml",
    "docs-site/src/content/docs/workspace/execution-contract-packs",
    "docs-site/src/content/docs/workspace/execution-locks",
    "docs-site/src/content/docs/workspace/track-execution-manifests",
    "docs-site/src/content/docs/workspace/truth-conformance-specs",
    "docs-site/src/content/docs/workspace/truth-verifier-registry.yaml",
    "docs-site/src/content/docs/reports/track-execution-manifests",
    "docs-site/src/content/docs/reports/track-execution-runs",
    "docs-site/src/content/docs/reports/truth-certificates",
    "docs-site/src/content/docs/workspace/roadmap-items.yaml",
    "docs-site/src/content/docs/workspace/roadmap-archive.yaml",
    "docs-site/src/content/docs/workspace/roadmap-deferred.yaml",
    "docs-site/src/content/docs/workspace/production-tracks.yaml",
];

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".to_owned());

    let result = match command.as_str() {
        "validate" => validate(args.any(|arg| arg == "--extended")),
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

fn validate(extended: bool) -> Result<(), String> {
    let root = repository_root()?;

    for args in BASELINE_CARGO_STEPS {
        run(&root, "cargo", args)?;
    }

    validate_docs(&root)?;
    audit_repository(&root)?;

    if extended {
        for (program, args) in EXTENDED_STEPS {
            run(&root, program, args)?;
        }
    }

    Ok(())
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
            Err(error) => {
                return Err(format!("failed to run {program}: {error}"));
            }
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
        "AGENTS.md",
        "TESTING.md",
        "Taskfile.yml",
        "tools/checks/ux_lab_terminology.py",
        "docs-site/src/content/docs/workspace/engineering-workflow.md",
        "docs-site/src/content/docs/workspace/planning/roadmap.md",
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
        "CI must invoke the same canonical baseline as local development",
    )?;
    require_text(
        root,
        "Taskfile.yml",
        "- cargo validate",
        "the optional Task helper must delegate to cargo validate",
    )?;
    require_text(
        root,
        "AGENTS.md",
        "engineering-workflow.md",
        "AGENTS.md must route agents to the concise workflow authority",
    )?;
    require_text(
        root,
        "docs-site/src/content/docs/workspace/start-here.md",
        "engineering-workflow.md",
        "the workspace start page must route to the concise workflow authority",
    )?;
    require_text(
        root,
        "docs-site/src/content/docs/workspace/engineering-workflow.md",
        "were retired under issue `#122`",
        "the canonical workflow must record the completed retirement",
    )?;

    for marker in [
        "uv run",
        "ci:local:",
        "ci:extended:",
        "roadmap:",
        "production:",
        "planning:",
        "puml:",
        "track:",
        "execution:",
        "truth:",
        "batch:",
    ] {
        forbid_text(
            root,
            "Taskfile.yml",
            marker,
            "retired or duplicate workflow command surfaces must not return",
        )?;
    }

    for (relative, marker) in [
        ("AGENTS.md", "remain temporarily available"),
        (
            "docs-site/src/content/docs/workspace/start-here.md",
            "remains temporarily available",
        ),
        (
            "docs-site/src/content/docs/workspace/engineering-workflow.md",
            "remain temporarily available",
        ),
    ] {
        forbid_text(
            root,
            relative,
            marker,
            "retired workflow systems must not be described as active compatibility paths",
        )?;
    }

    eprintln!("> repository audit passed");
    Ok(())
}

fn require_text(root: &Path, relative: &str, marker: &str, reason: &str) -> Result<(), String> {
    let path = root.join(relative);
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("repository audit: failed to read {relative}: {error}"))?;
    if text.contains(marker) {
        Ok(())
    } else {
        Err(format!(
            "repository audit: {relative} is missing required marker {marker:?}: {reason}"
        ))
    }
}

fn forbid_text(root: &Path, relative: &str, marker: &str, reason: &str) -> Result<(), String> {
    let path = root.join(relative);
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("repository audit: failed to read {relative}: {error}"))?;
    if text.contains(marker) {
        Err(format!(
            "repository audit: {relative} contains retired marker {marker:?}: {reason}"
        ))
    } else {
        Ok(())
    }
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
    eprintln!("  cargo validate                    required baseline");
    eprintln!("  cargo xtask docs                  documentation validation only");
    eprintln!("  cargo xtask audit                 deterministic repository audit only");
    eprintln!("  cargo xtask validate --extended  baseline plus optional deep checks");
}
