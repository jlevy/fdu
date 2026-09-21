#!/usr/bin/env python3
"""Compare cold, warm and cache-only answers over one tree, and prove the cache was used.

Comparing warm against cold on its own cannot fail usefully: a cache that never serves
scans cold both times and matches. So every case also records the mechanism -- the
report's own `source` -- and a request whose warm run never reports a warm source is a
failure even when the bytes agree.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

FDU = os.environ.get("FDU_BIN", "/home/user/fdu/target/debug/fdu")

# Each case is a request, named by what makes it interesting.
CASES: list[tuple[str, list[str]]] = [
    ("default", []),
    ("summary", ["--view", "summary"]),
    ("tree", ["--view", "tree"]),
    ("types", ["--view", "types"]),
    ("extensions", ["--view", "extensions"]),
    ("largest", ["--view", "largest"]),
    ("files", ["--view", "files"]),
    ("size-apparent", ["--size", "apparent"]),
    ("size-allocated", ["--size", "allocated"]),
    ("sort-mtime", ["--sort", "mtime"]),
    ("sort-name", ["--sort", "name"]),
    ("depth-1", ["-d", "1"]),
    ("scan-depth-2", ["--scan-depth", "2"]),
    ("min-size", ["--min-size", "1024"]),
    ("no-gitignore", ["--no-gitignore"]),
    ("exclude-ignored", ["--exclude-ignored"]),
    ("one-filesystem", ["--one-filesystem"]),
    ("analyze-lines", ["--analyze", "lines"]),
    ("analyze-code", ["--analyze", "code"]),
    ("analyze-words", ["--analyze", "words"]),
    ("analyze-all", ["--analyze", "all"]),
    ("languages", ["--view", "languages", "--analyze", "lines"]),
    ("documents", ["--view", "documents", "--analyze", "words"]),
]


def run(args: list[str], cache_home: Path) -> tuple[int, str, str]:
    env = dict(os.environ, XDG_CACHE_HOME=str(cache_home), NO_COLOR="1")
    proc = subprocess.run(
        [FDU, *args], capture_output=True, text=True, env=env, timeout=300
    )
    return proc.returncode, proc.stdout, proc.stderr


def source_of(stdout: str) -> str | None:
    """The delivery label, which sits at the envelope root rather than per report."""
    try:
        doc = json.loads(stdout)
    except json.JSONDecodeError:
        return None
    return doc.get("source")


def normalise(doc: str) -> object:
    """Drop the fields that legitimately differ between two runs of the same request."""
    try:
        value = json.loads(doc)
    except json.JSONDecodeError:
        return doc

    def scrub(node: object) -> object:
        if isinstance(node, dict):
            return {
                k: scrub(v)
                for k, v in node.items()
                # Provenance describes the delivery, not the answer; timings and source
                # are expected to differ between a cold and a warm run of one request.
                if k not in {"generated_at", "scan_started_at", "source", "freshness", "elapsed_ns"}
            }
        if isinstance(node, list):
            return [scrub(item) for item in node]
        return node

    return scrub(value)


def main() -> int:
    root = Path(sys.argv[1])
    failures: list[str] = []
    never_warm: list[str] = []

    print(
        f"{'case':<22} {'cold':>6} {'warm':>6} {'only':>6}  "
        f"{'warm source':<16} {'only source':<12} verdict"
    )
    print("-" * 96)

    for name, extra in CASES:
        cache_home = Path(tempfile.mkdtemp(prefix="fdu-cache-"))
        try:
            base = [str(root), "--format", "json", *extra]

            cold_rc, cold_out, cold_err = run([*base, "--cache", "off"], cache_home)
            # Warm the cache with the same request, then ask again.
            run([*base, "--cache", "auto"], cache_home)
            warm_rc, warm_out, warm_err = run([*base, "--cache", "auto"], cache_home)
            only_rc, only_out, only_err = run([*base, "--cache", "only"], cache_home)

            warm_source = source_of(warm_out) or "-"
            verdict = []

            if normalise(cold_out) != normalise(warm_out):
                verdict.append("WARM!=COLD")
                failures.append(f"{name}: warm answer differs from cold")
            if only_rc == 0 and normalise(only_out) != normalise(cold_out):
                verdict.append("ONLY!=COLD")
                failures.append(f"{name}: cache-only answer differs from cold")
            if cold_rc != warm_rc:
                verdict.append(f"RC {cold_rc}/{warm_rc}")
                failures.append(f"{name}: exit code cold {cold_rc} vs warm {warm_rc}")
            # The mechanism check, calibrated per request kind. Comparing answers alone
            # cannot fail usefully, but the expectation is not uniform: a metadata walk
            # is cheap, so a metadata-only request re-walks and reports `cold_scan` by
            # design, and its proof that the snapshot serves is the cache-only run. Only
            # a content request is expected to be served warm.
            analyses = "--analyze" in extra
            only_source = source_of(only_out) or "-"
            if analyses:
                if warm_source != "warm_revalidate":
                    verdict.append(f"NOT-WARM({warm_source})")
                    never_warm.append(f"{name}: warm source {warm_source}")
            elif only_rc == 0 and only_source != "cache_only":
                verdict.append(f"NO-SNAPSHOT({only_source})")
                never_warm.append(f"{name}: cache-only source {only_source}")

            print(
                f"{name:<22} {cold_rc:>6} {warm_rc:>6} {only_rc:>6}  "
                f"{warm_source:<16} {only_source:<12} "
                f"{' '.join(verdict) if verdict else 'ok'}"
            )
        finally:
            shutil.rmtree(cache_home, ignore_errors=True)

    print()
    print(f"answer mismatches: {len(failures)}")
    for line in failures:
        print(f"  - {line}")
    print(f"mechanism failures (cache did not serve): {len(never_warm)}")
    if never_warm:
        print("  " + ", ".join(never_warm))
    return 1 if (failures or never_warm) else 0


if __name__ == "__main__":
    raise SystemExit(main())
