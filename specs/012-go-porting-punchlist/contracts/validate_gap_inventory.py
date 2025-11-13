#!/usr/bin/env python3
import json
import sys
from pathlib import Path

try:
    # jsonschema is installed via: make py-install-jsonschema
    import jsonschema
except Exception as e:
    print("jsonschema module not found. Run 'make py-install-jsonschema' first.", file=sys.stderr)
    sys.exit(2)

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / 'contracts' / 'gap-inventory.schema.json'
DATA_PATH = ROOT / 'gap-inventory.json'

def main():
    with SCHEMA_PATH.open('r', encoding='utf-8') as f:
        schema = json.load(f)
    with DATA_PATH.open('r', encoding='utf-8') as f:
        data = json.load(f)
    try:
        jsonschema.validate(instance=data, schema=schema)
    except jsonschema.ValidationError as e:
        print('Gap inventory validation: FAIL', file=sys.stderr)
        print(e.message, file=sys.stderr)
        # Show location context
        if e.path:
            print('At path:', '/'.join(map(str, e.path))),
        sys.exit(1)
    print('Gap inventory validation: PASS')

if __name__ == '__main__':
    main()
