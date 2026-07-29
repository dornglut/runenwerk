---
title: Investigations
status: active
owner: workspace
layer: reports
last_reviewed: 2026-07-29
---

# Investigations

Use this folder for source-grounded current-state findings that inform design,
implementation, migration, or review.

Investigation reports should identify:

- question and scope;
- evidence inspected;
- current implementation facts;
- ownership or boundary findings;
- uncertainty and evidence gaps;
- concrete next decision.

Investigations do not authorize implementation by themselves.

## Repository family and framework extraction

- [Repository Family Current-State Investigation](repository-family-current-state-investigation.md)
- [RunenSDF Extraction Investigation](runensdf-extraction-investigation.md)
- [RunenECS Extraction Investigation](runenecs-extraction-investigation.md)
- [RunenGPU and RunenRender Boundary Investigation](runenrender-extraction-investigation.md)
- [GPU and Render S0 Inventory](runengpu-render-s0-inventory.md)
- [GPU and Render S0 File Disposition](runengpu-render-s0-file-disposition.md)
- [GPU and Render S0 Identity, Consumer, and Lifecycle Inventory](runengpu-render-s0-identity-consumer-lifecycle.md)

## RunenGPU planning and review

- [RunenGPU Industry Architecture Comparison](runengpu-industry-comparison.md)
- [RunenGPU Public API Ergonomics Review](runengpu-public-api-ergonomics-review.md)
- [RunenGPU Proof Workload Strategy](runengpu-proof-workload-strategy.md)
- [RunenGPU G2 Capabilities and Resources Investigation](runengpu-g2-capabilities-resources-investigation.md)
- [RunenGPU G3 Access and Work Graph Investigation](runengpu-g3-access-work-graph-investigation.md)
- [RunenGPU G4 Context, Program, and WGPU Realization Investigation](runengpu-g4-context-program-realization-investigation.md)
- [Runen Family Operational Hardening Investigation](runen-family-operational-hardening-investigation.md)

## RunenRender and application pressure

- [RunenGPU and RunenRender Application-Domain Fit](runengpu-runenrender-application-domain-fit.md)

Reports in this directory remain supporting evidence. Canonical ownership and phase
requirements live in accepted ADRs, architecture documents, active designs, and
implementation specifications.