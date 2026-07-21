# Runenwerk

Runenwerk is an experimental Rust platform for building world-centric applications, editors, simulations, and rendering systems.

It is the integration and product repository for the wider Runen framework family. Reusable foundations are being separated into focused repositories, while Runenwerk retains application lifecycle, cross-domain composition, editor and runtime integration, adapters, and product-level policy.

> **Status:** Active development. Architecture and package boundaries are still evolving. Runenwerk does not yet provide a stable public API or production-readiness guarantee.

## Design direction

- explicit domain ownership and one-way dependencies;
- field-first spatial and rendering systems without mandatory mesh authoring;
- deterministic, inspectable contracts and structured diagnostics;
- direct authoring, procedural generation, simulation, and persistent state as composable sources;
- reusable frameworks separated from application and product integration;
- headless validation and focused conformance tests.

## Repository family

Runenwerk is being decomposed into focused peer frameworks:

- **RunenSDF** — signed-distance and implicit-field foundations;
- **RunenECS** — ECS and scheduling foundations;
- **RunenGPU** — shared GPU execution;
- **RunenRender** — image formation and rendering;
- **RunenUI** — standalone UI framework developed as a separate workstream.

Some extractions are still in progress. Runenwerk remains responsible for composing these frameworks into applications and tools.

## Repository layout

```text
foundation/  low-level shared contracts
domain/      engine-agnostic domain crates
engine/      runtime and integration
apps/        executable products and examples
adapters/    external-host and cross-system adapters
docs-site/   canonical documentation
```

## Documentation

- [Architecture overview](ARCHITECTURE.md)
- [Dependency rules](DEPENDENCY_RULES.md)
- [Domain ownership map](DOMAIN_MAP.md)
- [Crate inventory](CRATES.md)
- [Testing and validation](TESTING.md)
- [Documentation entrypoint](docs-site/src/content/docs/index.mdx)

Canonical long-form documentation lives under `docs-site/src/content/docs`.

## Contributions

Runenwerk is currently a solo-maintained research and development project.

Issue reports and technical discussion are welcome. External code contributions and pull requests are not currently accepted.

## License

Runenwerk is licensed under the [MIT License](LICENSE).
