#!/usr/bin/env python3
"""Ask one request after warming with a different one -- the shape fdu-gija got wrong.

`--analyze lines` after `--analyze all` once reported metrics nobody requested and moved
document_words; `--analyze lines` after `--analyze code` read an Unsupported record whose
line counts had been discarded. Both matched their own tests. The oracle here is the cold
answer to the asked request, never the warmer's.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from answer import (
    age_problems,
    answer,
    content_source_of,
    reference_outside,
    reject_unknown_flags,
)

# Repository-relative so the runbook is not tied to one checkout.
DEFAULT_FDU = Path(__file__).resolve().parents[2] / "target" / "debug" / "fdu"
FDU = os.environ.get("FDU_BIN") or str(DEFAULT_FDU)
WARMERS = {
    "W_none": [],
    "W_lines": ["--analyze", "lines"],
    "W_code": ["--analyze", "code"],
    "W_words": ["--analyze", "words"],
    "W_all": ["--analyze", "all"],
}
ASKS = {
    "a_lines": ["--analyze", "lines"],
    "a_code": ["--analyze", "code"],
    "a_words": ["--analyze", "words"],
    "a_all": ["--analyze", "all"],
    "a_lines_documents": ["--analyze", "lines", "--view", "documents"],
    "a_code_languages": ["--analyze", "code", "--view", "languages"],
}


STALE_REFERENCES = []


def run(args, cache):
    env = dict(os.environ, XDG_CACHE_HOME=str(cache), NO_COLOR="1")
    started = time.time_ns()
    p = subprocess.run([FDU, *args], capture_output=True, text=True, env=env, timeout=300)
    problem = reference_outside(p.stdout, started, time.time_ns())
    if problem:
        STALE_REFERENCES.append(f"{' '.join(args[1:])}: {problem}")
    return p.returncode, p.stdout, p.stderr


def body(out):
    return answer(out)


def analyze_field(out):
    try:
        d = json.loads(out)
    except json.JSONDecodeError:
        return None
    analysis = d.get("analysis") or {}
    # The documented meaning of this field is what the report REQUESTED. Serving it from
    # a wider stored set is what fdu-gija got wrong, so it is compared, not ignored.
    return {
        "analyze": analysis.get("analyze"),
        "analyzers": analysis.get("analyzers"),
        "options_fingerprint": analysis.get("options_fingerprint"),
    }


def analyzers(args):
    """The analyzer set a request asks for, or None for a metadata-only request."""
    return args[args.index("--analyze") + 1] if "--analyze" in args else None


def main():
    reject_unknown_flags(sys.argv[1:], set())
    root = Path(sys.argv[1])
    # The oracle: each request answered with no cache at all.
    cold = {}
    for ask, args in ASKS.items():
        c = Path(tempfile.mkdtemp(prefix="fdu-cold-"))
        try:
            rc, out, _ = run([str(root), "--format", "json", "--cache", "off", *args], c)
            cold[ask] = (rc, body(out), analyze_field(out))
        finally:
            shutil.rmtree(c, ignore_errors=True)

    print(f"{'warmer':<8} {'ask':<20} {'rc':>3}  {'analysis.analyze':<26} verdict")
    print("-" * 82)
    bad = []
    for wname, wargs in WARMERS.items():
        for ask, aargs in ASKS.items():
            cache = Path(tempfile.mkdtemp(prefix="fdu-cross-"))
            try:
                run([str(root), "--format", "json", "--cache", "auto", *wargs], cache)
                rc, out, _ = run([str(root), "--format", "json", "--cache", "auto", *aargs], cache)
                got, want = body(out), cold[ask][1]
                gotf, wantf = analyze_field(out), cold[ask][2]
                v = []
                if rc != cold[ask][0]:
                    v.append(f"RC {cold[ask][0]}->{rc}")
                if got != want:
                    v.append("ANSWER!=COLD")
                if gotf != wantf:
                    v.append(f"ANALYZE {wantf}->{gotf}")
                if age_problems(out):
                    v.append("AGE")
                # A warmer that stored the same analyzer set must serve the ask's content
                # records. A wider set may not yet (the containment deferral, fdu-7dj6),
                # so only the matching pairs are held to serving; every pair is held to
                # the cold answer.
                if analyzers(wargs) == analyzers(aargs) and content_source_of(out) != "revalidated":
                    v.append(f"NOT-WARM({content_source_of(out)})")
                if v:
                    bad.append(f"{wname} -> {ask}: {' '.join(v)}")
                shown = str((gotf or {}).get("analyze"))
                verdict = " ".join(v) if v else "ok"
                print(f"{wname:<8} {ask:<20} {rc:>3}  {shown:<26} {verdict}")
            finally:
                shutil.rmtree(cache, ignore_errors=True)
    print()
    bad.extend(STALE_REFERENCES)
    print(f"cross-warm violations: {len(bad)}")
    for b in bad:
        print(f"  - {b}")
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())
