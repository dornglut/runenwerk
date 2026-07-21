---
title: RunenSDF Repository Identity Decision
status: active
owner: sdf
layer: domain/sdf
canonical: true
last_reviewed: 2026-07-21
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
edition: 2024
minimum supported Rust version: 1.93.0
license: MIT OR Apache-2.0
publication: disabled
```

This decision supersedes the former `Crystonix/RunenSDF` and `runensdf` planning
spellings. No compatibility package, crate alias, forwarding namespace, deprecated
re-export, or branch alias preserves those spellings.

The repository uses one root public package. Its only support workspace packages
are:

```text
conformance/downstream  public downstream-consumer proof
xtask                   repository validation authority
```

It does not use `crates/runen-sdf`, `domain/sdf`, a documentation site, a façade,
a source submodule, a private source include, or speculative public subpackages.

The source-transfer baseline is Runenwerk merge commit
`8de096259eab30f8d67672010df9190970d0bfc4`, path `domain/sdf`.

`PT-RUNENSDF-003` owns standalone transfer and independent conformance.
`PT-RUNENSDF-004` owns Runenwerk dependency cutover and deletion of the original
source.

The repository commits `Cargo.lock`. GitHub Actions invokes the repository-local
`cargo validate` command and never generates or mutates the lockfile. Before
publication, Runenwerk may consume only an exact commit or exact prerelease; moving
branch dependencies are forbidden.