#!/usr/bin/env python3

from __future__ import annotations

import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MEMBERSHIP = ROOT / "build-aux" / "verify.toml"
WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
TASK_NAME = re.compile(r"^[A-Za-z0-9:_-]+$")
MISE_RUN = re.compile(r"mise run ([^\s]+)")
CI_TASKS = {"setup:rust", "verify", "package:windows"}


def load_membership() -> dict[str, list[str]]:
    data = tomllib.loads(MEMBERSHIP.read_text(encoding="utf-8"))
    hosts: dict[str, list[str]] = {}
    for name, table in data.items():
        tasks = table.get("tasks")
        if not isinstance(tasks, list) or not tasks:
            raise SystemExit(f"{name} has no tasks in {MEMBERSHIP.name}")
        for task in tasks:
            if not isinstance(task, str) or not TASK_NAME.fullmatch(task):
                raise SystemExit(f"invalid task {task!r} in host {name}")
        hosts[name] = tasks
    return hosts


def check_workflow(hosts: dict[str, list[str]]) -> None:
    if "check:no-wasm" not in hosts.get("check", []):
        raise SystemExit("the check host must run check:no-wasm")
    text = WORKFLOW.read_text(encoding="utf-8")
    used = set(MISE_RUN.findall(text))
    extra = sorted(used - CI_TASKS)
    if extra:
        raise SystemExit(
            "ci.yml runs mise tasks outside setup:rust, verify, and package:windows: "
            + ", ".join(extra)
        )
    if "verify" not in used:
        raise SystemExit("ci.yml does not run mise run verify")


def selected_host(hosts: dict[str, list[str]]) -> str:
    host = os.environ.get("NOTA_VERIFY_HOST")
    if not host:
        host = "windows" if os.name == "nt" else "linux"
    if host not in hosts:
        known = ", ".join(sorted(hosts))
        raise SystemExit(f"unknown NOTA_VERIFY_HOST {host!r}. Known hosts are {known}.")
    return host


def main() -> int:
    hosts = load_membership()
    check_workflow(hosts)
    host = selected_host(hosts)
    if "--list" in sys.argv:
        print("\n".join(hosts[host]))
        return 0
    for task in hosts[host]:
        print(f"verify {host}: {task}", flush=True)
        completed = subprocess.run(["mise", "run", task], cwd=ROOT)
        if completed.returncode != 0:
            return completed.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
