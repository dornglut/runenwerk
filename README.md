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