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

## What Bootstrap Owns

The bootstrap script installs or checks the host tools that cannot be provided
by `uv.lock`:

- Git;
- uv;
- Task;
- Dagger CLI;
- Rust/Cargo;
- cargo-nextest;
- cargo-deny;
- cargo-machete;
- lychee;
- PlantUML;
- Node.js and npm;
- ast-grep;
- Renovate CLI.

It also warns when no Docker, Podman, or compatible container runtime command is
available. Dagger needs a running container runtime for `task ci:local`.

Some WinGet portable packages do not expose a command on `PATH`. The bootstrap
script creates stable command shims under `%USERPROFILE%\.runenwerk\bin` and
adds that directory to the user `PATH` when needed.

## Boundaries

The repo pins Python dependencies with `pyproject.toml`, `uv.lock`, and
`.python-version`. The bootstrap script is only for host executables.

Taskfile remains the canonical command surface after bootstrap:

```powershell
task --list
task ci:host
task ci:local
```

## Maintenance

When a tool is added to `Taskfile.yml`, update all three places together:

1. `tools/bootstrap/bootstrap.ps1`;
2. this document;
3. `README.md` quickstart.

Do not duplicate installer logic in docs-only snippets without updating the
bootstrap script.
