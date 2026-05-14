from __future__ import annotations

import dagger
from dagger import dag, function, object_type


@object_type
class Runenwerk:
    @function
    async def ci_local(self, source: dagger.Directory) -> str:
        """Run the canonical local validation pipeline in containers."""
        workspace = (
            source
            .without_directory(".git")
            .without_directory("target")
            .without_directory(".venv")
        )

        python = (
            dag.container()
            .from_("ghcr.io/astral-sh/uv:python3.14-bookworm-slim")
            .with_directory("/src", workspace)
            .with_workdir("/src")
            .with_exec(["uv", "run", "python", "tools/workflow/roadmap_state.py", "validate"])
            .with_exec(["uv", "run", "python", "tools/workflow/roadmap_state.py", "schema", "--check"])
            .with_exec(["uv", "run", "python", "tools/workflow/generate_roadmap_docs.py", "check"])
            .with_exec(["uv", "run", "--group", "dev", "python", "-m", "pytest", "tools/workflow"])
            .with_exec(["python", "tools/docs/validate_docs.py"])
        )

        rust = (
            dag.container()
            .from_("rust:1.95-bookworm")
            .with_directory("/src", workspace)
            .with_workdir("/src")
            .with_exec(["cargo", "check", "--workspace"])
        )

        await python.stdout()
        await rust.stdout()
        return "Runenwerk Dagger local CI completed"
