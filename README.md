# Runenwerk

Runenwerk is a long-term world, editor, simulation, and rendering platform organized around explicit domain ownership, stable contracts, and inspectable documentation.

Canonical documentation lives under:

```text
docs-site/src/content/docs
```

## Human start

Start here for repository orientation, planning, implementation, review, and cleanup:

```text
docs-site/src/content/docs/workspace/start-here.md
```

For the docs-site landing page, use:

```text
docs-site/src/content/docs/index.mdx
```

## AI agent start

AI coding agents must start from:

```text
AGENTS.md
```

`AGENTS.md` is the only root AI entrypoint. It explains how to work through GitHub connector, context tooling, Codex-style patching, manual file inspection, or a local checkout without requiring command execution.

## Core references

- Architecture summary: `ARCHITECTURE.md`
- Dependency direction: `DEPENDENCY_RULES.md`
- Concept ownership map: `DOMAIN_MAP.md`
- Crate inventory: `CRATES.md`
- Validation guidance: `TESTING.md`
- Terminology: `GLOSSARY.md`
- Programming principles: `docs-site/src/content/docs/guidelines/programming-principles.md`

## Optional local setup

Local tools are optional helpers. They are useful for validation, but they are not required to understand the repository workflow.

Toolchain bootstrap details live in:

```text
docs-site/src/content/docs/workspace/toolchain-bootstrap.md
```

## Optional local validation

When a local checkout is available, run the smallest relevant validation first. For docs-only changes, start with:

```text
python3 tools/docs/validate_docs.py
```

For code changes, use `TESTING.md` to choose focused validation before broad workspace checks.
