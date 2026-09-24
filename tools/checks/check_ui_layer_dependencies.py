#!/usr/bin/env python3
"""
Check Runenwerk UI crate dependency boundaries.

File: tools/checks/check_ui_layer_dependencies.py
Function: main

This checker is intentionally section-aware:
- production dependencies must be listed in allowed_dependencies or transitional_allowed_dependencies
- dev-dependencies may additionally use allowed_dev_dependencies or transitional_dev_dependencies
- build-dependencies may additionally use allowed_build_dependencies or transitional_build_dependencies

The checker catches boundary drift. It does not replace architecture review.
"""

from __future__ import annotations

import argparse
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


DEPENDENCY_SECTIONS = ("dependencies", "dev-dependencies", "build-dependencies")


@dataclass(frozen=True)
class CrateRule:
    key: str
    package: str
    path: Path
    layer: str
    allowed_dependencies: frozenset[str]
    transitional_allowed_dependencies: frozenset[str]
    allowed_dev_dependencies: frozenset[str]
    transitional_dev_dependencies: frozenset[str]
    allowed_build_dependencies: frozenset[str]
    transitional_build_dependencies: frozenset[str]
    forbidden_dependencies: frozenset[str]

    def allowed_for_section(self, section: str) -> frozenset[str]:
        production = self.allowed_dependencies | self.transitional_allowed_dependencies
        if section == "dev-dependencies":
            return (
                production
                | self.allowed_dev_dependencies
                | self.transitional_dev_dependencies
            )
        if section == "build-dependencies":
            return (
                production
                | self.allowed_build_dependencies
                | self.transitional_build_dependencies
            )
        return production


@dataclass(frozen=True)
class CheckIssue:
    severity: str
    package: str
    message: str


def load_toml(path: Path) -> dict:
    with path.open("rb") as file:
        return tomllib.load(file)


def find_repo_root(start: Path) -> Path:
    current = start.resolve()
    for candidate in [current, *current.parents]:
        if (candidate / "Cargo.toml").is_file() and (candidate / "domain").is_dir():
            return candidate
    raise SystemExit(f"could not find repository root from {start}")


def load_rules(root: Path) -> tuple[dict[str, CrateRule], dict]:
    ownership_path = root / "domain/ui/ui-crate-ownership.toml"
    if not ownership_path.is_file():
        raise SystemExit(f"missing ownership map: {ownership_path}")

    data = load_toml(ownership_path)
    policy = data.get("policy", {})
    crates = {}
    for key, raw in data.get("crate", {}).items():
        package = raw["package"]
        crates[package] = CrateRule(
            key=key,
            package=package,
            path=root / raw["path"],
            layer=raw["layer"],
            allowed_dependencies=frozenset(raw.get("allowed_dependencies", [])),
            transitional_allowed_dependencies=frozenset(
                raw.get("transitional_allowed_dependencies", [])
            ),
            allowed_dev_dependencies=frozenset(raw.get("allowed_dev_dependencies", [])),
            transitional_dev_dependencies=frozenset(
                raw.get("transitional_dev_dependencies", [])
            ),
            allowed_build_dependencies=frozenset(raw.get("allowed_build_dependencies", [])),
            transitional_build_dependencies=frozenset(
                raw.get("transitional_build_dependencies", [])
            ),
            forbidden_dependencies=frozenset(raw.get("forbidden_dependencies", [])),
        )
    return crates, policy


def package_name(cargo_toml: Path) -> str | None:
    try:
        data = load_toml(cargo_toml)
    except tomllib.TOMLDecodeError as error:
        raise SystemExit(f"failed to parse {cargo_toml}: {error}") from error
    package = data.get("package", {})
    name = package.get("name")
    return name if isinstance(name, str) else None


def dependency_names_by_section(cargo_toml: Path) -> dict[str, set[str]]:
    data = load_toml(cargo_toml)
    result: dict[str, set[str]] = {section: set() for section in DEPENDENCY_SECTIONS}

    for section in DEPENDENCY_SECTIONS:
        dependencies = data.get(section, {})
        if not isinstance(dependencies, dict):
            continue

        for dependency_name, dependency_value in dependencies.items():
            result[section].add(dependency_name)
            if isinstance(dependency_value, dict):
                package = dependency_value.get("package")
                if isinstance(package, str):
                    result[section].add(package)

    return result


def is_app_path_dependency(root: Path, cargo_toml: Path, dependency_value: object) -> bool:
    if not isinstance(dependency_value, dict):
        return False
    path_value = dependency_value.get("path")
    if not isinstance(path_value, str):
        return False
    dependency_path = (cargo_toml.parent / path_value).resolve()
    try:
        relative = dependency_path.relative_to(root)
    except ValueError:
        return False
    return relative.parts[:1] == ("apps",)


def app_path_dependency_names_by_section(root: Path, cargo_toml: Path) -> dict[str, set[str]]:
    data = load_toml(cargo_toml)
    names: dict[str, set[str]] = {section: set() for section in DEPENDENCY_SECTIONS}
    for section in DEPENDENCY_SECTIONS:
        dependencies = data.get(section, {})
        if not isinstance(dependencies, dict):
            continue
        for dependency_name, dependency_value in dependencies.items():
            if is_app_path_dependency(root, cargo_toml, dependency_value):
                names[section].add(dependency_name)
    return names


def discover_ui_crates(root: Path) -> dict[str, Path]:
    crates = {}
    ui_root = root / "domain/ui"
    if not ui_root.is_dir():
        return crates

    for cargo_toml in ui_root.rglob("Cargo.toml"):
        name = package_name(cargo_toml)
        if name is not None:
            crates[name] = cargo_toml.parent

    return crates


def check_crate(
    root: Path,
    rule: CrateRule,
    all_rules: dict[str, CrateRule],
    policy: dict,
) -> list[CheckIssue]:
    issues: list[CheckIssue] = []
    cargo_toml = rule.path / "Cargo.toml"
    if not cargo_toml.is_file():
        issues.append(
            CheckIssue(
                "error",
                rule.package,
                f"declared crate path has no Cargo.toml: {rule.path}",
            )
        )
        return issues

    dependencies_by_section = dependency_names_by_section(cargo_toml)
    app_path_deps_by_section = app_path_dependency_names_by_section(root, cargo_toml)

    hard_forbidden_packages = set(policy.get("hard_forbidden_packages", []))
    hard_forbidden_packages.update(rule.forbidden_dependencies)

    for section, dependencies in dependencies_by_section.items():
        for dependency in sorted(dependencies):
            if dependency in hard_forbidden_packages:
                issues.append(
                    CheckIssue(
                        "error",
                        rule.package,
                        f"forbidden {section} dependency on {dependency}",
                    )
                )

    for section, dependencies in app_path_deps_by_section.items():
        for dependency in sorted(dependencies):
            issues.append(
                CheckIssue(
                    "error",
                    rule.package,
                    f"domain/ui crate depends on apps/ path dependency {dependency} in {section}",
                )
            )

    known_ui_dependencies = set(all_rules.keys())
    for section, dependencies in dependencies_by_section.items():
        allowed = rule.allowed_for_section(section)
        for dependency in sorted(dependencies & known_ui_dependencies):
            if dependency == rule.package:
                issues.append(CheckIssue("error", rule.package, "crate depends on itself"))
                continue
            if dependency not in allowed:
                issues.append(
                    CheckIssue(
                        "warning",
                        rule.package,
                        f"UI {section} dependency {dependency} is not listed in ownership map",
                    )
                )

    return issues


def check_unclassified_ui_crates(
    discovered: dict[str, Path],
    rules: dict[str, CrateRule],
) -> list[CheckIssue]:
    issues = []
    for package, path in sorted(discovered.items()):
        if package not in rules:
            issues.append(
                CheckIssue(
                    "error",
                    package,
                    f"UI crate is missing from domain/ui/ui-crate-ownership.toml: {path}",
                )
            )
    return issues


def print_issues(issues: Iterable[CheckIssue]) -> int:
    issues = list(issues)
    if not issues:
        print("UI dependency boundary check passed.")
        return 0

    error_count = sum(1 for issue in issues if issue.severity == "error")
    warning_count = sum(1 for issue in issues if issue.severity == "warning")

    for issue in issues:
        print(f"{issue.severity.upper()}: {issue.package}: {issue.message}")

    print(f"UI dependency boundary check found {error_count} errors and {warning_count} warnings.")
    return 1 if error_count else 0


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Check Runenwerk UI crate ownership and dependency boundaries."
    )
    parser.add_argument(
        "--root",
        default=".",
        help="Repository root. Defaults to current directory or nearest parent repository.",
    )
    parser.add_argument(
        "--deny-warnings",
        action="store_true",
        help="Treat undocumented UI dependencies as errors.",
    )
    args = parser.parse_args()

    root = find_repo_root(Path(args.root))
    rules, policy = load_rules(root)
    discovered = discover_ui_crates(root)

    issues: list[CheckIssue] = []
    issues.extend(check_unclassified_ui_crates(discovered, rules))
    for rule in rules.values():
        issues.extend(check_crate(root, rule, rules, policy))

    if args.deny_warnings:
        issues = [
            CheckIssue(
                "error" if issue.severity == "warning" else issue.severity,
                issue.package,
                issue.message,
            )
            for issue in issues
        ]

    raise SystemExit(print_issues(issues))


if __name__ == "__main__":
    main()
