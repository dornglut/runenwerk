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


## Root, adapters, and params

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/app_ext.rs` | `8fe36a7dbb2a164c8cbdf617f127592bcbca2ecf` | `inspect`, `root-facade` | — | — | 1 |
| `engine/src/plugins/render/gpu_context_policy.rs` | `124f212ed4233bcd1b3efa53a40e73b8c7a56ce4` | — | `runen_gpu` | — | 2 |
| `engine/src/plugins/render/mod.rs` | `2e673aaae7108834518ee930b77e96314bb89dde` | — | — | — | 42 |
| `engine/src/plugins/render/native_host.rs` | `f9adc551e92faaa9eb034cd32822f888beaa2bc7` | `backend`, `renderer` | — | — | 5 |
| `engine/src/plugins/render/plugin.rs` | `b850ab0417e85ab967c717684b2104c03541718b` | `backend`, `composition`, `features`, `frame`, `inspect`, `pipelines`, `residency`, `runtime`, `shader` | — | `scene`, `ui` | 3 |
| `engine/src/plugins/render/readiness.rs` | `c2257ef4c864fd9a9135d9ca690b6472be87c53b` | — | — | — | 9 |
| `engine/src/plugins/render/texture_upload.rs` | `b0a0221a7974c0c86cbd54d7fb7d2e626a0a6d3c` | `root-facade` | `runen_gpu` | — | 2 |
| `engine/src/plugins/render/adapters/gpu_capabilities.rs` | `7bd8b8e4ec45acc85e863a3070257fb4c0acd663` | `graph`, `root-facade` | `runen_gpu` | — | 6 |
| `engine/src/plugins/render/adapters/gpu_data.rs` | `25ac5ec41d973733066e1ffb4df8d1a0a33cdaac` | `root-facade` | `runen_gpu` | — | 11 |
| `engine/src/plugins/render/adapters/gpu_resources.rs` | `2fa8106dad686ca9d1409064fddf19986ca224e3` | `root-facade` | `runen_gpu` | — | 84 |
| `engine/src/plugins/render/adapters/gpu_work.rs` | `5d211514155119d9d21a6bb43775b29fef288e5f` | — | `runen_gpu` | — | 15 |
| `engine/src/plugins/render/adapters/mod.rs` | `f181b2da9f78c99100d9972d24c54c30ed7a2aca` | — | — | — | 4 |
| `engine/src/plugins/render/params/gpu_params.rs` | `bd1a983cc36555eadc32563cada1472ccb94d805` | — | — | — | 3 |
| `engine/src/plugins/render/params/gpu_value.rs` | `c0c79a339bedd9492dc54ecff30980787567e09e` | — | — | — | 5 |
| `engine/src/plugins/render/params/mod.rs` | `f8bb7609fa145b0b8c051107d495c244bf2b4f5c` | — | — | — | 4 |
