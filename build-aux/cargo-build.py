#!/usr/bin/env python3
import pathlib
import shutil
import subprocess
import sys


def main() -> int:
    source_root = pathlib.Path(sys.argv[1])
    target_dir = pathlib.Path(sys.argv[2])
    output = pathlib.Path(sys.argv[3])
    subprocess.run(
        [
            "cargo",
            "build",
            "--release",
            "--locked",
            "-p",
            "nota-desktop",
            "--features",
            "preview-webkit",
            "--target-dir",
            str(target_dir),
        ],
        cwd=source_root,
        check=True,
    )
    shutil.copy2(target_dir / "release" / "nota-desktop", output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
