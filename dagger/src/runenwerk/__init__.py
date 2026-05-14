from __future__ import annotations

from typing import Annotated

import dagger
from dagger import DefaultPath, Ignore, dag, function, object_type


SOURCE_FILTER = [
    ".git",
    ".git/**",
    ".venv",
    ".venv/**",
    "target",
    "target/**",
    "node_modules",
    "node_modules/**",
    "docs-site/node_modules",
    "docs-site/node_modules/**",
    "docs-site/dist",
    "docs-site/dist/**",
    "docs-site/.astro",
    "docs-site/.astro/**",
    "playgrounds/godot-chunking-demo/.godot",
    "playgrounds/godot-chunking-demo/.godot/**",
    "**/__pycache__",
    "**/__pycache__/**",
    "**/*.pyc",
]


@object_type
class Runenwerk:
    @function
    async def ci_local(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("."),
            Ignore(SOURCE_FILTER),
        ],
    ) -> str:
        """Run the canonical local validation pipeline in containers."""
        workspace = source

        python = (
            dag.container()
            .from_("ghcr.io/astral-sh/uv:python3.14-bookworm-slim")
            .with_exec(
                [
                    "sh",
                    "-c",
                    "apt-get update && apt-get install -y --no-install-recommends git && rm -rf /var/lib/apt/lists/*",
                ]
            )
            .with_directory("/src", workspace)
            .with_workdir("/src")
            .with_exec(
                ["uv", "run", "python", "tools/workflow/roadmap_state.py", "validate"]
            )
            .with_exec(
                ["uv", "run", "python", "tools/workflow/roadmap_state.py", "schema", "--check"]
            )
            .with_exec(
                ["uv", "run", "python", "tools/workflow/generate_roadmap_docs.py", "check"]
            )
            .with_exec(
                ["uv", "run", "--group", "dev", "python", "-m", "pytest", "tools/workflow"]
            )
            .with_exec(["python", "tools/docs/validate_docs.py"])
        )

        rust = (
            dag.container()
            .from_("rust:1.95-bookworm")
            .with_directory("/src", workspace)
            .with_workdir("/src")
            .with_mounted_cache(
                "/usr/local/cargo/registry",
                dag.cache_volume("runenwerk-cargo-registry"),
            )
            .with_mounted_cache(
                "/usr/local/cargo/git",
                dag.cache_volume("runenwerk-cargo-git"),
            )
            .with_mounted_cache(
                "/src/target",
                dag.cache_volume("runenwerk-cargo-target"),
            )
            .with_exec(["cargo", "check", "--workspace"])
        )

        await python.stdout()
        await rust.stdout()
        return "Runenwerk Dagger local CI completed"
