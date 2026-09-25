#!/usr/bin/env python3
"""Validate the generated documentation projection without rerunning source checks."""

import argparse
from pathlib import Path

from validate_docs import report, validate_publication_build


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("build_output", type=Path)
    arguments = parser.parse_args()

    errors: list[str] = []
    validate_publication_build(arguments.build_output, errors)
    return report(errors)


if __name__ == "__main__":
    raise SystemExit(main())
