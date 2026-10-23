#!/usr/bin/env python3

"""Fail when a bold label in the usage guide or verify-nota is missing from owner sources.

A quoted label is text wrapped in ** ** that does not end with a colon.
A trailing colon marks a procedure heading, not a control label.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LABEL = re.compile(r"\*\*([^*]+)\*\*")
OWNER_ROOTS = (
    ROOT / "crates" / "nota-desktop" / "src",
    ROOT / "crates" / "nota-core" / "src",
    ROOT / "crates" / "nota-app" / "src",
    ROOT / "windows",
)
OWNER_SUFFIXES = {".rs", ".cs", ".xaml"}


def quoted_labels(text: str) -> list[str]:
    labels: list[str] = []
    for match in LABEL.finditer(text):
        label = match.group(1).strip()
        if not label or label.endswith(":"):
            continue
        labels.append(label)
    return labels


def documents() -> list[Path]:
    skill = ROOT / ".agents" / "skills" / "verify-nota"
    return [ROOT / "docs" / "usage.md", *sorted(skill.rglob("*.md"))]


def owner_text() -> str:
    chunks: list[str] = []
    for owner in OWNER_ROOTS:
        for path in sorted(owner.rglob("*")):
            if path.is_file() and path.suffix.lower() in OWNER_SUFFIXES:
                chunks.append(path.read_text(encoding="utf-8"))
    return "\n".join(chunks)


def missing_labels(docs: list[tuple[Path, str]], owners: str) -> list[str]:
    seen: set[str] = set()
    missing: list[str] = []
    for path, text in docs:
        for label in quoted_labels(text):
            if label in seen:
                continue
            seen.add(label)
            if label not in owners:
                missing.append(f"{path.relative_to(ROOT)}: {label}")
    return missing


def self_check() -> None:
    labels = quoted_labels("Select **New note** in **CUA:** now.")
    if labels != ["New note"]:
        raise SystemExit(f"label extractor returned {labels!r}")
    sample = missing_labels(
        [(ROOT / "docs" / "usage.md", "Press **Not a real control**.")],
        "other text",
    )
    if sample != ["docs/usage.md: Not a real control"]:
        raise SystemExit(f"missing-label check returned {sample!r}")


def main() -> int:
    self_check()
    docs = [(path, path.read_text(encoding="utf-8")) for path in documents()]
    missing = missing_labels(docs, owner_text())
    if missing:
        print("Quoted labels missing from the owner modules:")
        for line in missing:
            print(line)
        return 1
    print("ok: quoted labels are present in the owner modules")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
