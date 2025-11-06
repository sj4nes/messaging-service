#!/usr/bin/env python3
"""
Validate all example event payloads against the common envelope JSON Schema.

Usage:
  python specs/004-create-domain-events/contracts/events/validate_examples.py

Requires:
  - Python 3.8+
  - jsonschema (pip install jsonschema)

Exit codes:
  0 = all examples valid
  1 = one or more examples invalid
  2 = environment error (e.g., missing dependency or files)
"""
from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    here = Path(__file__).resolve().parent
    repo_root = here.parents[4] if len(here.parents) >= 5 else here
    schema_path = here / "envelope.schema.json"
    examples_dir = here / "examples"

    try:
        import jsonschema
        from jsonschema import Draft7Validator
    except Exception as e:  # pragma: no cover
        sys.stderr.write(
            "Missing dependency 'jsonschema'. Install with: pip install jsonschema\n"
        )
        return 2

    if not schema_path.is_file():
        sys.stderr.write(f"Schema not found: {schema_path}\n")
        return 2
    if not examples_dir.is_dir():
        sys.stderr.write(f"Examples directory not found: {examples_dir}\n")
        return 2

    # Load schema and validate it is a valid Draft-07 schema
    try:
        schema = json.loads(schema_path.read_text())
        Draft7Validator.check_schema(schema)
    except Exception as e:
        sys.stderr.write(f"Schema invalid or unreadable: {e}\n")
        return 2

    validator = Draft7Validator(schema)
    example_files = sorted(examples_dir.glob("*.example.json"))
    if not example_files:
        sys.stderr.write(f"No example files found in {examples_dir}\n")
        return 2

    failures: list[tuple[Path, list[str]]] = []
    for path in example_files:
        try:
            instance = json.loads(path.read_text())
        except Exception as e:
            failures.append((path, [f"Invalid JSON: {e}"]))
            continue

        errors = sorted(validator.iter_errors(instance), key=lambda e: e.path)
        if errors:
            msgs = []
            for err in errors:
                loc = "/".join(str(p) for p in err.absolute_path) or "<root>"
                msgs.append(f"{loc}: {err.message}")
            failures.append((path, msgs))
        else:
            print(f"✓ {path.relative_to(repo_root)}")

    if failures:
        print("\nValidation failures:")
        for path, msgs in failures:
            print(f"✗ {path}")
            for m in msgs:
                print(f"  - {m}")
        return 1

    print(f"\nAll {len(example_files)} examples are valid.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
