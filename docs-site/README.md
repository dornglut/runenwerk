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

## Documentation links

Within the docs source tree, use ordinary relative links to other Markdown
documents (for example, `../plugins/README.md`). The site build resolves
routed `.md` and `.mdx` source references to published Starlight URLs,
including the `/runenwerk` base and `README.md` route mapping.

Do not link public pages directly to `repository-current` or `history`
documents. Link to the repository source if historical evidence is needed.

The generated-HTML publication validator checks actual in-page links,
target pages and files, and fragment IDs. Run it after building:

```sh
python3 tools/docs/validate_publication_build.py docs-site/dist
```
