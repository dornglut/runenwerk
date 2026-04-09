---
title: "Foundation SDF Implementation Roadmap"
description: "Documentation for Foundation SDF Implementation Roadmap."
---

# Foundation SDF Implementation Roadmap

Status: implemented baseline
Scope: `domain/sdf`

## Completed Structure

```text
domain/sdf/
├── Cargo.toml
├── README.md
├── docs/
│   ├── index.md
│   ├── implementation-roadmap.md
│   ├── api-notes.md
│   ├── ownership-boundary.md
│   ├── QUERY_MODEL.md
│   └── NUMERICS.md
├── src/
│   ├── lib.rs
│   ├── field.rs
│   ├── sample.rs
│   ├── bounds.rs
│   ├── gradient.rs
│   ├── normal.rs
│   ├── epsilon.rs
│   ├── primitives/
│   ├── ops/
│   ├── transform/
│   ├── queries/
│   ├── combine/
│   └── util/
└── tests/
```

## Phase Summary

- S1 scaffold: complete
- S2 primitives: complete (`sphere`, `box3`, `capsule`, `plane`, `torus`, `cylinder`)
- S3 core ops: complete (`union`, `subtract`, `intersect`, smooth variants)
- S4 transforms: complete (`translate`, `rotate`, `scale`, `affine`)
- S5 gradients/normals/epsilon: complete
- S6 raymarch query: complete
- S7 projection query: complete
- S8 classification: complete
- S9 smooth ops: complete
- S10 extended transforms: complete
- S11 docs closeout: complete

## Milestone State

- M1 minimal usable SDF core: complete
- M2 query-ready core: complete
- M3 gameplay-ready static interaction base: complete
- M4 mature reusable SDF foundation: complete
