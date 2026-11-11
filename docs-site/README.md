# Runenwerk Docs Site

This is the canonical Runenwerk documentation source tree.

Docs live in:

```text
docs-site/src/content/docs
```

## Commands

Run these from `docs-site/`:

```sh
pnpm install
pnpm build
pnpm dev
```

Run repository-wide docs validation from the repository root:

```sh
python3 tools/docs/validate_docs.py
```

## Structure

- `workspace/`: repository orientation, planning, documentation structure, and shared vocabulary
- `architecture/`: current Runenwerk platform and integration architecture
- `guidelines/`: stable Runenwerk engineering, dependency, module-structure, and placement guidance
- `foundation/`: foundation-specific documentation
- `domain/`: domain-specific documentation
- `engine/`: engine/runtime/plugin documentation
- `net/`: networking, simulation, replay, and runtime convergence documentation
- `apps/`: runnable application and product documentation
- `adapters/`: external runtime, host, and cross-system integration documentation
- `adr/`: durable architecture decision records
- `design/`: design lifecycle documents and section guidance
- `reports/`: investigations, proofs, closeouts, benchmarks, and retained historical evidence

## Source-backed Rust snippets

The example pages can embed maintained Rust source without copying it into
Markdown. Use an empty fenced code block with a `runenwerk-source` directive:

````text
```rust runenwerk-source=engine/examples/runtime_minimal/main.rs
```
````

The `remark-rust-example-source` plugin reads the actual source file during
Astro Markdown processing. Only Rust files under `engine/examples/` are
accepted; missing or malformed references fail the build. Node directive tests
execute automatically as part of `pnpm build`.

Explanations remain maintainer-authored. Source-backed code blocks guarantee
the displayed text matches the source file; they do not replace compilation,
runtime testing, or API-contract review. Rust example compilation continues
under the repository-owned Rust validation.
