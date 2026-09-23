#!/usr/bin/env python3
"""Compare cold, warm and cache-only answers over one tree, and prove the cache was used.

Comparing warm against cold on its own cannot fail usefully: a cache that never serves
scans cold both times and matches. So every case also records the mechanism -- the
report's `provenance.source` -- and a request whose warm run never reports a warm source
is a failure even when the bytes agree. `--refusals-only` runs over a tree whose refusal
entries make every answer partial, where the check is instead that nothing was stored.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from answer import answer, freshness_of, is_complete, source_of

# Repository-relative so the runbook is not tied to one checkout.
DEFAULT_FDU = Path(__file__).resolve().parents[2] / "target" / "debug" / "fdu"
FDU = os.environ.get("FDU_BIN") or str(DEFAULT_FDU)

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
    proc = subprocess.run([FDU, *args], capture_output=True, text=True, env=env, timeout=300)
    return proc.returncode, proc.stdout, proc.stderr


def main() -> int:
    args = sys.argv[1:]
    # A tree built with its refusal entries is partial for an unprivileged user, and a
    # partial scan never writes the entry tier: a snapshot missing an entry would be
    # served as the tree's totals. Such a run proves the refusal path and that nothing
    # partial was stored; the serving proof needs a tree built --without-refusals.
    refusals_only = "--refusals-only" in args
    root = Path(next(arg for arg in args if not arg.startswith("--")))
    failures: list[str] = []
    never_warm: list[str] = []
    served = 0

    print(
        f"{'case':<22} {'cold':>6} {'warm':>6} {'only':>6}  "
        f"{'warm source':<16} {'only source':<12} {'fresh':<7} verdict"
    )
    print("-" * 104)

    for name, extra in CASES:
        cache_home = Path(tempfile.mkdtemp(prefix="fdu-cache-"))
        try:
            base = [str(root), "--format", "json", *extra]

            cold_rc, cold_out, _ = run([*base, "--cache", "off"], cache_home)
            # Warm the cache with the same request, then ask again.
            run([*base, "--cache", "auto"], cache_home)
            warm_rc, warm_out, _ = run([*base, "--cache", "auto"], cache_home)
            only_rc, only_out, _ = run([*base, "--cache", "only"], cache_home)

            warm_source = source_of(warm_out) or "-"
            verdict = []

            if answer(cold_out) != answer(warm_out):
                verdict.append("WARM!=COLD")
                failures.append(f"{name}: warm answer differs from cold")
            if only_rc == 0 and answer(only_out) != answer(cold_out):
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
            #
            # Every clause here must be reachable. An earlier version guarded the
            # cache-only check with `only_rc == 0`, which can never be false in the
            # failure it was written for: the engine either serves `cache_only` or exits
            # 1, so a build that never writes a snapshot exited 1 and skipped the check
            # entirely. Seventeen of these cases then printed `ok` against a cache that
            # was never written or served -- the exact failure this file exists to catch.
            analyses = "--analyze" in extra
            only_source = source_of(only_out) or "-"
            only_freshness = freshness_of(only_out) or "-"

            partial = is_complete(cold_out) is False
            if refusals_only and not partial:
                verdict.append("NOT-PARTIAL")
                failures.append(f"{name}: expected a partial answer; are the refusals effective?")
            if partial:
                # Correct behaviour is to store nothing, so cache-only must refuse.
                if only_rc == 0:
                    verdict.append("PARTIAL-STORED")
                    never_warm.append(f"{name}: a partial scan was served from the cache")
                else:
                    verdict.append("withheld")
            elif only_rc != 0:
                verdict.append(f"NO-SNAPSHOT(rc={only_rc})")
                never_warm.append(f"{name}: cache-only exited {only_rc}, so nothing was stored")
            elif only_source != "cache_only":
                verdict.append(f"NO-SNAPSHOT({only_source})")
                never_warm.append(f"{name}: cache-only source {only_source}")
            elif only_freshness != "stale":
                # Cache-only serves without verifying, so it must label the answer stale.
                verdict.append(f"NOT-STALE({only_freshness})")
                never_warm.append(f"{name}: cache-only freshness {only_freshness}")
            else:
                served += 1

            if analyses and not partial and warm_source != "warm_revalidate":
                verdict.append(f"NOT-WARM({warm_source})")
                never_warm.append(f"{name}: warm source {warm_source}")

            print(
                f"{name:<22} {cold_rc:>6} {warm_rc:>6} {only_rc:>6}  "
                f"{warm_source:<16} {only_source:<12} {only_freshness:<7} "
                f"{' '.join(verdict) if verdict else 'ok'}"
            )
        finally:
            shutil.rmtree(cache_home, ignore_errors=True)

    print()
    print(f"answer mismatches: {len(failures)}")
    for line in failures:
        print(f"  - {line}")
    print(f"mechanism failures (cache did not serve): {len(never_warm)}")
    for line in never_warm:
        print(f"  - {line}")
    print(f"cases the snapshot served: {served} of {len(CASES)}")
    if not refusals_only and served == 0:
        # A run that never served proved nothing about serving, whatever else matched.
        print("the cache never served: on a tree built --without-refusals, serving is broken")
        return 1
    return 1 if (failures or never_warm) else 0


if __name__ == "__main__":
    raise SystemExit(main())
