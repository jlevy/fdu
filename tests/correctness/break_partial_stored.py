#!/usr/bin/env python3
"""A deliberately broken fdu for the correctness runbook: a partial answer is served.

Name this file as `FDU_BIN`, and the real binary as `FDU_REAL`. It passes every argument
to the real fdu, except that it answers `--stale-ok` with a cold scan's output relabeled
as if the cache had served it (`cache_only`, `stale`), keeping the cold exit status.
Over the refusal tree, as an unprivileged user, every case of `warm_cold.py
--refusals-only` must then report `PARTIAL-STORED` and the script must exit 1: a partial
answer exits 2, so a check that trusted a zero exit would call it withheld.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys


def relabeled(stdout: str) -> str:
    """A JSON report relabeled as served from the cache without verifying; other output
    is returned unchanged."""
    try:
        document = json.loads(stdout)
    except json.JSONDecodeError:
        return stdout
    if not isinstance(document, dict):
        return stdout
    provenance = document.setdefault("provenance", {})
    provenance["source"] = "cache_only"
    provenance["freshness"] = "stale"
    return json.dumps(document)


def main() -> int:
    real = os.environ.get("FDU_REAL")
    if not real:
        sys.exit("FDU_REAL must name the real fdu binary")
    args = sys.argv[1:]
    if "--stale-ok" not in args:
        os.execv(real, [real, *args])
    cold = [arg for arg in args if arg != "--stale-ok"] + ["--cache", "off"]
    result = subprocess.run([real, *cold], capture_output=True, text=True, check=False)
    sys.stdout.write(relabeled(result.stdout))
    sys.stderr.write(result.stderr)
    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
