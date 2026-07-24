---
title: PT-RUNENSDF-004 Internal SDF Retirement Closeout
description: Exact census, retirement-only decision, deletion, validation, and no-return evidence for removing Runenwerk's duplicate internal SDF authority.
status: completed
owner: workspace
layer: reports
canonical: true
last_reviewed: 2026-07-24
related_docs:
  - ../../workspace/planning/completed-work.md
  - ../../workspace/planning/roadmap.md
  - ../../architecture/repository-family-architecture.md
  - ../../reports/investigations/repository-family-current-state-investigation.md
---

# PT-RUNENSDF-004 Internal SDF Retirement Closeout

## Outcome

Runenwerk retired the duplicate internal `domain/sdf` package without adding an external dependency. Reusable signed-field mathematics and CPU reference-query authority remains in `dornglut/runen-sdf`; Runenwerk retains only product/world integration such as `domain/world_sdf`.

## Census evidence

The exact-head census ran against feature head `5462fb4f2453787370da832bf360d07798f3f8eb` in workflow run `30076695148`. The evidence artifact recorded:

| Class | Result | Disposition |
|---|---|---|
| Workspace/package authority | `domain/sdf/Cargo.toml`, root workspace member, and one `Cargo.lock` package | retired |
| Cargo inverse dependencies | only `sdf v0.1.0 (domain/sdf)` | zero external consumers |
| Other manifests | no dependency on package `sdf`, `runen-sdf`, or path `domain/sdf` | no migration required |
| Rust consumer patterns | no `use sdf::`, `extern crate sdf`, or package-path consumer outside the package | zero real code consumers |
| Similar identifiers | `world_sdf` imports and SDF-named examples/features | unrelated Runenwerk product/world terminology; retained |
| Documentation | internal package docs plus historical transfer/cutover references | stale framework mirror removed; provenance retained |
| Git index/submodules | tracked internal source; no gitlinks or submodules | internal paths deleted; no submodule introduced |
| External dependency | none | none added |

## Delivered retirement

- deleted all tracked `domain/sdf` source, tests, and manifest files;
- removed the workspace member and internal lockfile package;
- deleted the duplicate local framework documentation under `docs/domain/sdf`;
- corrected canonical crate inventories, domain maps, repository-family state, planning, and editor navigation;
- removed the transient census workflow;
- extended the permanent repository audit to reject return of the internal path, package, lockfile entry, stale documentation mirror, path/source forwarding, submodule, or `runen-sdf` dependency;
- preserved historical extraction and boundary evidence without presenting it as current source authority.

## Validation contract

The final PR must pass the repository-owned `cargo validate` baseline, exact-head GitHub Actions, documentation production build, metadata/lockfile checks, and the no-return repository audit. The PR closes only with zero tracked `domain/sdf` paths and no validation artifact.

## Follow-up

RunenGPU G1A issue `#131` is the next implementation slice. Any future Runenwerk use of RunenSDF requires a separately authorized exact dependency and a real consumer; this retirement does not create one.
