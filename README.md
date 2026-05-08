# Runenwerk

Canonical documentation lives under `docs-site/src/content/docs`.

Start with:

- `docs-site/src/content/docs/index.mdx`
- `docs-site/src/content/docs/workspace/overview.md`
- `docs-site/src/content/docs/guidelines/architecture.md`
- `CRATES.md`
- `DOMAIN_MAP.md`

## AI Context Export

Use this to export repository source and documentation into a single line-numbered context file for AI review:

```bash
python3 tools/context/export_repo_context.py
```

Optional output path:

```bash
python3 tools/context/export_repo_context.py --output ./Runenwerk-content.txt
```

## AI Workflow Kickoff

Use this to generate a task-shaped AI workflow prompt and checklist:

```bash
./workflow list
./workflow implementation --task "<task>" --scope "<scope>"
./workflow closeout --task "<completed phase>" --roadmap "<roadmap/design>"
```

Use these validation shortcuts from the repository root:

```bash
./workflow docs
./workflow full-gate
```
