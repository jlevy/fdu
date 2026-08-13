"""Run the PyO3 Rust tests with uv's standalone Python visible to the loader."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import sysconfig


def prepend_env_path(env: dict[str, str], name: str, paths: list[Path]) -> None:
    """Prepend existing directories without discarding a caller's loader path."""
    values = [str(path) for path in paths if path.is_dir()]
    if current := env.get(name):
        values.append(current)
    if values:
        env[name] = os.pathsep.join(values)


def main() -> int:
    """Run the native concurrency tests against the interpreter selected by uv."""
    env = os.environ.copy()
    executable_dir = Path(sys.executable).resolve().parent
    configured_lib = sysconfig.get_config_var("LIBDIR")
    library_dirs = [executable_dir]
    if configured_lib:
        library_dirs.insert(0, Path(configured_lib))

    if sys.platform == "win32":
        prepend_env_path(env, "PATH", library_dirs)
    elif sys.platform == "darwin":
        prepend_env_path(env, "DYLD_FALLBACK_LIBRARY_PATH", library_dirs)
    else:
        prepend_env_path(env, "LD_LIBRARY_PATH", library_dirs)

    crate_dir = Path(__file__).resolve().parent.parent
    completed = subprocess.run(
        [
            "cargo",
            "test",
            "--locked",
            "-p",
            "fdu-py",
            "--lib",
            "--no-default-features",
        ],
        cwd=crate_dir,
        env=env,
        check=False,
    )
    return completed.returncode


if __name__ == "__main__":
    raise SystemExit(main())
