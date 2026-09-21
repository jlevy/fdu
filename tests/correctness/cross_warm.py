#!/usr/bin/env python3
"""Ask one request after warming with a different one -- the shape fdu-gija got wrong.

`--analyze lines` after `--analyze all` once reported metrics nobody requested and moved
document_words; `--analyze lines` after `--analyze code` read an Unsupported record whose
line counts had been discarded. Both matched their own tests. The oracle here is the cold
answer to the asked request, never the warmer's.
"""
from __future__ import annotations
import json, os, shutil, subprocess, sys, tempfile
from pathlib import Path

FDU = os.environ.get("FDU_BIN", "/home/user/fdu/target/debug/fdu")
WARMERS = {
    "W_none":  [],
    "W_lines": ["--analyze", "lines"],
    "W_code":  ["--analyze", "code"],
    "W_words": ["--analyze", "words"],
    "W_all":   ["--analyze", "all"],
}
ASKS = {
    "a_lines": ["--analyze", "lines"],
    "a_code":  ["--analyze", "code"],
    "a_words": ["--analyze", "words"],
    "a_all":   ["--analyze", "all"],
    "a_lines_documents": ["--analyze", "lines", "--view", "documents"],
    "a_code_languages":  ["--analyze", "code", "--view", "languages"],
}

def run(args, cache):
    env = dict(os.environ, XDG_CACHE_HOME=str(cache), NO_COLOR="1")
    p = subprocess.run([FDU, *args], capture_output=True, text=True, env=env, timeout=300)
    return p.returncode, p.stdout, p.stderr

def scrub(node):
    if isinstance(node, dict):
        return {k: scrub(v) for k, v in node.items()
                if k not in {"generated_at", "scan_started_at", "source", "freshness", "elapsed_ns"}}
    if isinstance(node, list):
        return [scrub(v) for v in node]
    return node

def body(out):
    try:
        return scrub(json.loads(out))
    except json.JSONDecodeError:
        return out

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

def main():
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
                if v:
                    bad.append(f"{wname} -> {ask}: {' '.join(v)}")
                shown = str((gotf or {}).get("analyze"))
                verdict = " ".join(v) if v else "ok"
                print(f"{wname:<8} {ask:<20} {rc:>3}  {shown:<26} {verdict}")
            finally:
                shutil.rmtree(cache, ignore_errors=True)
    print()
    print(f"cross-warm violations: {len(bad)}")
    for b in bad:
        print(f"  - {b}")
    return 1 if bad else 0

if __name__ == "__main__":
    raise SystemExit(main())
