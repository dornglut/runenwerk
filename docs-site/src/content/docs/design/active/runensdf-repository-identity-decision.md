---
title: RunenSDF Repository Identity Decision
status: active
owner: sdf
layer: domain/sdf
canonical: true
last_reviewed: 2026-07-20
related_docs:
  - ./runensdf-extraction-design.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/specs/pt-runensdf-003-standalone-transfer.ron
---

# RunenSDF Repository Identity Decision

```text
product: RunenSDF
repository: Crystonix/runen-sdf
package: runen-sdf
crate: runen_sdf
version: 0.1.0
```

This record supersedes the earlier `Crystonix/RunenSDF` and `runensdf` planning
spellings in the extraction design. No compatibility package or crate alias
preserves those spellings.

The repository uses one root public package plus `conformance/downstream` and
`xtask` as support workspace packages. It does not use `crates/runen-sdf`,
`domain/sdf`, a docs site, a facade, or speculative public subpackages.

The source-transfer baseline is Runenwerk merge commit
`8de096259eab30f8d67672010df9190970d0bfc4`, path `domain/sdf`.

Implementation authority is `PT-RUNENSDF-003`; Runenwerk integration remains
`PT-RUNENSDF-004` scope.
