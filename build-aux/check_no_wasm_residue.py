#!/usr/bin/env python3

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CRATES = ROOT / "crates"
PATTERN = re.compile(r"target_arch\s*=\s*\"wasm32\"|cfg\(.*wasm32")

def main() -> int:
    hits: list[str] = []
    for path in sorted(CRATES.rglob("*.rs")):
        text = path.read_text(encoding="utf-8")
        for lineno, line in enumerate(text.splitlines(), start=1):
            if PATTERN.search(line):
                hits.append(f"{path.relative_to(ROOT)}:{lineno}:{line.strip()}")
    if hits:
        print("wasm32 residue found in crates/:")
        for hit in hits:
            print(hit)
        return 1
    print("ok: no wasm32 residue in crates/")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
