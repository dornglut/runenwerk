# Runenwerk xtask

This standalone Rust tool owns repository validation. Its validation dependencies
remain local to this tooling workspace and do not affect the product workspace.

It is intentionally outside the product workspace so adding or changing repository tooling cannot rewrite the product `Cargo.lock`.

## Commands

```text
cargo validate
cargo xtask docs
cargo xtask audit
cargo xtask validate --extended
```

`cargo validate` is the required, read-only local and CI baseline. The extended profile is manual or scheduled and may require additional tools.

The xtask must remain bounded to deterministic repository validation. It must not become a roadmap engine, work scheduler, permission system, prompt generator, or substitute for GitHub issues and pull requests.
