---
title: Toolchain Bootstrap
description: First-clone setup for the Runenwerk workflow toolchain.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-14
related:
  - ./planning-and-implementation-workflow.md
  - ./parallel-roadmap-batch-automation.md
  - ./roadmap-items.yaml
---

# Toolchain Bootstrap

## Policy

Do not make clone itself install tools. Git intentionally does not run trusted
repository code after clone, and adding hidden hooks would be unsafe.

The Runenwerk policy is one explicit bootstrap command after clone, followed by
Taskfile commands for all normal work.

## First Clone

From the repository root on Windows:

```powershell
powershell -ExecutionPolicy Bypass -File tools/bootstrap/bootstrap.ps1
```

Then open a new shell and verify:

```powershell
task toolchain:doctor
task roadmap:validate
task roadmap:check
task puml:validate
```

If a new shell still cannot resolve `task` or `uv`, load the known
tool paths into the current session:

```powershell
. .\tools\bootstrap\activate.ps1
```

## What Bootstrap Owns

The bootstrap script installs or checks the host tools that cannot be provided
by `uv.lock`:

- Git;
- uv;
- Task;
- Rust/Cargo;
- cargo-nextest;
- cargo-deny;
- cargo-machete;
- lychee;
- PlantUML;
- Node.js and npm;
- ast-grep;
- Renovate CLI.

Some WinGet portable packages do not expose a command on `PATH`. The bootstrap
script creates stable command shims under `%USERPROFILE%\.runenwerk\bin` and
adds that directory to the user `PATH` when needed.

`tools/bootstrap/activate.ps1` does not install anything. Dot-source it with
`. .\tools\bootstrap\activate.ps1` so it can prepend the known bootstrap
locations to the current shell. Taskfile, uv, cargo, PlantUML, lychee, and
ast-grep can be found without restarting the terminal.

## Boundaries

The repo pins Python dependencies with `pyproject.toml`, `uv.lock`, and
`.python-version`. The bootstrap script is only for host executables.

Taskfile remains the canonical command surface after bootstrap:

```powershell
task --list
task batch:validate -- --batch docs-site/src/content/docs/reports/batches/<batch-id>/batch.toml
task ci:local
```

Runenwerk does not configure remote CI. Treat `task ci:local` as the manual
full validation gate before pushing, opening a PR, or closing out a batch.

## Maintenance

When a tool is added to `Taskfile.yml`, update all three places together:

1. `tools/bootstrap/bootstrap.ps1`;
2. this document;
3. `README.md` quickstart.

Do not duplicate installer logic in docs-only snippets without updating the
bootstrap script.
