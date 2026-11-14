#!/usr/bin/env python3
"""Validate the generated documentation projection without rerunning source checks."""

import argparse
from pathlib import Path
import unittest

from validate_docs import report, validate_publication_build


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("build_output", type=Path)
    arguments = parser.parse_args()

    suite = unittest.defaultTestLoader.discover(str(Path(__file__).parent), pattern="test_rendered_links.py")
    if not unittest.TextTestRunner(verbosity=1).run(suite).wasSuccessful():
        return 1

    errors: list[str] = []
    validate_publication_build(arguments.build_output, errors)
    return report(errors)


if __name__ == "__main__":
    raise SystemExit(main())
