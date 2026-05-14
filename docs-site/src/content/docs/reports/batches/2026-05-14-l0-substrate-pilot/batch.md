---
title: Batch 2026-05-14-l0-substrate-pilot
description: Parallel roadmap batch closeout report.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-14
---

# Batch 2026-05-14-l0-substrate-pilot

Goal: L0 substrate pilot: ECS/runtime and render contract support
Approval state: approved
Integration status: integrated
Closeout status: completed

## Validation Results

- task toolchain:doctor passed
- task roadmap:validate passed
- task roadmap:check passed
- task batch:scope-check passed
- task puml:validate passed
- task docs:validate passed
- task links:check passed
- task python:test passed
- task rust:test passed: 1407 tests passed, 1 skipped
- task rust:policy passed
- task ast:scan passed
- task ci:local passed after Dagger source filtering and container setup hardening; final run completed in about 2m27s

## Roadmap Evidence Updates

- WR-002 evidence updated: ECS runtime now exposes stable plan reports for product-job diagnostics; lifecycle/finalization remains the next support gap.
- WR-003 evidence updated: render product selection snapshots are view-ordered, and selected-source residency now derives, invalidates, and inspects source contract state.
- Generated roadmap decision register, triage status, schema files, and value-weighted dependency roadmap are in sync with roadmap-items.yaml.

## Tooling Hardening

- batch:prepare now validates against canonical roadmap YAML and generates missing worker prompts before worktree setup.
- batch:approve rejects stale gates, blocked roadmap items, and scope drift before approval.
- scope enforcement now checks committed, staged, unstaged, and untracked worktree changes.
- worker prompts are generated as docs-site-compatible lowercase Markdown files with frontmatter.
- batch closeout reports render validation results, roadmap evidence updates, and tooling hardening from batch.toml.
- PlantUML and ast-grep gates are clean under the Taskfile workflow.
- Rust policy baseline now gates advisories, licenses, sources, and unused dependencies with cargo-deny plus cargo-machete.
- Workspace crates are marked publish=false so cargo-deny treats them as private crates.
- Dagger module layout now uses the package path expected by the Python SDK and keeps main.py as a compatibility re-export.
- Dagger source input now uses DefaultPath plus Ignore filters so target, .git, virtualenvs, node modules, and generated caches are excluded before upload.
- Dagger Rust validation now mounts persistent Cargo registry, git, and target caches.
- Dagger Python validation installs git because workflow tests exercise git-backed batch state.

## Items

### WR-002 ECS/runtime convergence support for product jobs and diagnostics

- Branch: `codex/2026-05-14-l0-substrate-pilot-wr-002`
- Worktree: `C:/Users/joshi/Projekte/Runenwerk-worktrees/2026-05-14-l0-substrate-pilot/WR-002`
- Status: `completed`
- Write scopes: `domain/ecs`, `docs-site/src/content/docs/net/ecs-runtime-prioritized-roadmap.md`

### WR-003 Render contract follow-ups through product selection and derived residency

- Branch: `codex/2026-05-14-l0-substrate-pilot-wr-003`
- Worktree: `C:/Users/joshi/Projekte/Runenwerk-worktrees/2026-05-14-l0-substrate-pilot/WR-003`
- Status: `completed`
- Write scopes: `engine/src/plugins/render`, `docs-site/src/content/docs/engine/plugins/render`
