---
title: Runenwerk Render Runtime Lexical Dependency Census
description: Immutable accepted-revision source dependency appendix for investigation #1138, recording explicit render-group, framework, peer-plugin, and public-surface references per central Render Rust file.
status: active
owner: engine
layer: investigation
canonical: false
last_reviewed: 2026-10-03
publication: reference
pagefind: false
related_docs:
  - ./2026-10-03-runenwerk-render-runtime-decomposition.md
  - ./2026-10-03-runenwerk-render-runtime-source-census.md
---

# Runenwerk Render Runtime Lexical Dependency Census

This appendix closes the exact dependency-census requirement for investigation #1138.

```text
RUNENWERK_REVISION=8d0bad6c7e4c24f2579bd89c3f797946af85dba3
PATH_PREFIX=engine/src/plugins/render/
```

Every row below is derived from the exact file bytes fetched at that immutable revision. The
dependency columns are an **exact lexical source-reference census**, not a compiler-resolved Rust
type graph:

- **Render deps** records explicit cross-top-level Render module references, plus `root-facade`
  when a file imports re-exported `crate::plugins::render::{...}` / root symbols whose owning
  submodule is intentionally hidden by the facade.
- **Framework deps** records direct `runen_render`, `runen_gpu`, `runen_shader`, `wgpu`,
  and `naga` source references.
- **Peer deps** records direct references to other `crate::plugins::<peer>` owners.
- **Public items** is the count of lexically declared `pub` / `pub(...)` structs, enums,
  traits, functions, aliases, constants, statics, modules, and uses in the file. It is a surface
  pressure indicator, not a claim that every item is externally reachable after module visibility.

Relative `super::...` paths are resolved against the file's module path before assigning a
top-level Render dependency. Same-top-level-module references are omitted from **Render deps**.

