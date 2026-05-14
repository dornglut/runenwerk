# Runenwerk

Canonical documentation lives under `docs-site/src/content/docs`.

## First Clone Bootstrap

Runenwerk uses Taskfile as the human command entrypoint. A clone cannot safely
install tools automatically, so run the explicit bootstrap once after clone:

```powershell
powershell -ExecutionPolicy Bypass -File tools/bootstrap/bootstrap.ps1
```

Then open a new shell and verify:

```powershell
task toolchain:doctor
task roadmap:validate
task roadmap:check
task puml:validate
```

If the shell cannot find `task` or `uv` after bootstrap, activate the
known tool paths in the current PowerShell session:

```powershell
. .\tools\bootstrap\activate.ps1
```

Details: `docs-site/src/content/docs/workspace/toolchain-bootstrap.md`.

Start with:

- `docs-site/src/content/docs/index.mdx`
- `docs-site/src/content/docs/workspace/overview.md`
- `docs-site/src/content/docs/workspace/toolchain-bootstrap.md`
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
task ai:list
task ai:implementation -- --task "<task>" --scope "<scope>"
task ai:closeout -- --task "<completed phase>" --roadmap "<roadmap/design>"
```

Runenwerk does not configure remote CI. Run these validation shortcuts from the
repository root before pushing, opening a PR, or closing out a batch:

```bash
task docs:validate
task batch:validate -- --batch docs-site/src/content/docs/reports/batches/<batch-id>/batch.toml
task ci:local
```
