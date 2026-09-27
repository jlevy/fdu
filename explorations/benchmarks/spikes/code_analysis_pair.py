#!/usr/bin/env python3
"""Measure the same fdu population as cold and seeded-warm adjacent pairs.

The fixture generator and runner use only the Python standard library. Timed runs
exclude counter rendering; separate counter runs record the work each arm did.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import platform
import random
import statistics
import subprocess
import tempfile
import time
from pathlib import Path


def write_tree(root: Path, scenario: str) -> dict[str, int]:
    root.mkdir(parents=True, exist_ok=True)
    files = 0
    bytes_written = 0

    def put(name: str, body: bytes) -> None:
        nonlocal files, bytes_written
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(body)
        files += 1
        bytes_written += len(body)

    if scenario == "mixed":
        put(".gitignore", b"target/\n")
        for i in range(180):
            put(f"src/group{i // 30}/module{i:03}.rs", b"// module\nfn run() { let n = 1; }\n" * 8)
        for i in range(80):
            put(f"docs/guide{i:03}.md", b"# Guide\n\nA short explanation.\n" * 12)
        for i in range(60):
            put(f"scripts/task{i:03}.py", b"# task\nprint('ready')\n" * 10)
        for i in range(40):
            put(f"target/cache{i:03}.rs", b"pub const N: usize = 1;\n" * 12)
    elif scenario == "long-line":
        put("src/long.rs", b"const HUGE: &str = \"" + b"x" * (8 * 1024 * 1024) + b"\";\n")
        for i in range(24):
            put(f"src/small{i:03}.rs", b"fn small() {}\n" * 20)
    elif scenario == "generated":
        put(".gitignore", b"target/\n")
        for i in range(24):
            put(f"generated/schema{i:03}.rs", b"pub const VALUE: u64 = 42;\n" * 8_000)
        for i in range(24):
            put(f"src/hand{i:03}.rs", b"fn hand() {}\n" * 20)
    elif scenario == "ignored-heavy":
        put(".gitignore", b"ignored/\n")
        for i in range(500):
            put(f"ignored/batch{i // 50}/unit{i:03}.rs", b"pub fn generated() {}\n" * 120)
        for i in range(25):
            put(f"src/main{i:03}.rs", b"fn main() {}\n" * 40)
    else:
        raise ValueError(scenario)
    return {"files": files, "apparent_bytes": bytes_written}


def argv(binary: Path, tree: Path, cache: Path, population: str, measured: str, warm: bool) -> list[str]:
    cmd = [str(binary), "--cache", "auto" if warm else "off", "--cache-dir", str(cache),
           "--ignored", population, "--format", "json"]
    if measured == "code":
        cmd += ["--analyze", "code", "--view", "code"]
    else:
        cmd += ["--view", "summary"]
    return cmd + [str(tree)]


def oracle(cmd: list[str]) -> dict[str, object]:
    result = subprocess.run(cmd, capture_output=True, check=True, text=True)
    report = json.loads(result.stdout)
    return {"status": report["status"], "reports": report["reports"]}


def sample(cmd: list[str]) -> dict[str, int]:
    start = time.perf_counter_ns()
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    _, status, usage = os.wait4(proc.pid, 0)
    wall = time.perf_counter_ns() - start
    proc.returncode = os.waitstatus_to_exitcode(status)
    if proc.returncode != 0:
        raise RuntimeError(f"sample exited {proc.returncode}: {cmd[1:]}")
    rss = usage.ru_maxrss if platform.system() == "Darwin" else usage.ru_maxrss * 1024
    return {
        "wall_ns": wall,
        "user_ns": round(usage.ru_utime * 1_000_000_000),
        "system_ns": round(usage.ru_stime * 1_000_000_000),
        "peak_rss_bytes": rss,
        "minor_faults": usage.ru_minflt,
        "major_faults": usage.ru_majflt,
    }


def counters(cmd: list[str]) -> dict[str, int]:
    env = dict(os.environ, FDU_COUNTERS="1")
    result = subprocess.run(cmd, capture_output=True, check=True, text=True, env=env)
    output: dict[str, int] = {}
    group = ""
    for line in result.stderr.splitlines():
        if line.startswith("[") and line.endswith("]"):
            group = line[1:-1]
        elif group and line.startswith("  "):
            label, _, value = line.strip().rpartition("  ")
            if value.strip().isdigit():
                output[f"{group}.{label.strip()}"] = int(value)
    return output


def assert_seeded_analysis(cold: dict[str, int], warm: dict[str, int],
                           scenario: str, population: str) -> None:
    """Require observed sidecar reuse where cold analysis read source bodies."""
    source_opens = "filesystem operations.file opens"
    sidecar_apply = "content sidecar timing.sidecar apply microseconds"
    if cold.get(source_opens, 0) == 0:
        return
    if warm.get(source_opens, -1) != 0 or warm.get(sidecar_apply, 0) <= 0:
        raise RuntimeError(
            f"seeded analysis was not observed: {scenario}/{population}; "
            f"warm source opens={warm.get(source_opens)}, "
            f"sidecar apply us={warm.get(sidecar_apply)}"
        )


def publish_json(output: Path, evidence: dict[str, object]) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary: Path | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=output.parent,
            prefix=f".{output.name}.", suffix=".tmp", delete=False,
        ) as handle:
            temporary = Path(handle.name)
            json.dump(evidence, handle, indent=2)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, output)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def interval(deltas: list[float]) -> tuple[float, float]:
    rng = random.Random(20260926)
    medians = sorted(statistics.median(rng.choices(deltas, k=len(deltas))) for _ in range(4_000))
    return medians[100], medians[3_900]


def summarize(rows: list[dict[str, dict[str, int]]]) -> dict[str, object]:
    cold = [row["cold"] for row in rows]
    warm = [row["warm"] for row in rows]
    changes = [(w["wall_ns"] / c["wall_ns"] - 1) * 100 for c, w in zip(cold, warm, strict=True)]
    low, high = interval(changes)
    def med(samples: list[dict[str, int]], field: str) -> int:
        return round(statistics.median(sample[field] for sample in samples))
    return {
        "pairs": len(rows),
        "cold_wall_median_ns": med(cold, "wall_ns"),
        "warm_wall_median_ns": med(warm, "wall_ns"),
        "warm_vs_cold_median_pct": round(statistics.median(changes), 2),
        "warm_vs_cold_bootstrap_95_pct": [round(low, 2), round(high, 2)],
        "cold_cpu_median_ns": med(cold, "user_ns") + med(cold, "system_ns"),
        "warm_cpu_median_ns": med(warm, "user_ns") + med(warm, "system_ns"),
        "cold_peak_rss_median_bytes": med(cold, "peak_rss_bytes"),
        "warm_peak_rss_median_bytes": med(warm, "peak_rss_bytes"),
        "cold_peak_rss_max_bytes": max(sample["peak_rss_bytes"] for sample in cold),
        "warm_peak_rss_max_bytes": max(sample["peak_rss_bytes"] for sample in warm),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--scratch", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--pairs", type=int, default=12)
    parser.add_argument("--host-regime", default="not established")
    parser.add_argument("--filesystem", default="not established")
    args = parser.parse_args()
    if args.pairs < 2:
        parser.error("--pairs must be at least 2")
    binary = args.binary.resolve(strict=True)
    scratch = args.scratch.resolve(strict=True)
    if not os.access(scratch, os.W_OK):
        parser.error("scratch directory must be writable")
    for name in ("subjects", "caches"):
        path = scratch / name
        if path.is_symlink() or (path.exists() and (not path.is_dir() or any(path.iterdir()))):
            parser.error(f"{path} is not empty; use a fresh task-specific scratch directory")
    scenarios = ["mixed", "long-line", "generated", "ignored-heavy"]
    fixtures = {name: write_tree(scratch / "subjects" / name, name) for name in scenarios}
    cells = []
    for scenario in scenarios:
        for population, measured in [("include", "metadata"), ("include", "code"),
                                     ("exclude", "code"), ("only", "code")]:
            tree = scratch / "subjects" / scenario
            cache = scratch / "caches" / scenario / population / measured
            cold = argv(binary, tree, cache, population, measured, False)
            warm = argv(binary, tree, cache, population, measured, True)
            cold_answer = oracle(cold)
            warm_answer = oracle(warm)  # also seeds this exact scope and analyzer set
            if cold_answer != warm_answer:
                raise RuntimeError(f"cold/warm report mismatch: {scenario}/{population}/{measured}")
            # Warm OS namespace/page-cache state without touching the measured pairs.
            oracle(cold)
            oracle(warm)
            rows = []
            for pair in range(args.pairs):
                order = [("cold", cold), ("warm", warm)]
                if pair % 2:
                    order.reverse()
                row = {arm: sample(cmd) for arm, cmd in order}
                rows.append(row)
            cold_counters = counters(cold)
            warm_counters = counters(warm)
            if measured == "code":
                assert_seeded_analysis(cold_counters, warm_counters, scenario, population)
            cells.append({
                "scenario": scenario,
                "population": population,
                "measurement": measured,
                "summary": summarize(rows),
                "cold_counters": cold_counters,
                "warm_counters": warm_counters,
                "samples": rows,
            })
            print(f"{scenario}/{population}/{measured}: {args.pairs} pairs", flush=True)
    evidence = {
        "schema": "fdu.code-analysis-pairs/1",
        "collected_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "tool_version": subprocess.run(
            [str(binary), "--version"], capture_output=True, check=True, text=True
        ).stdout.strip(),
        "platform": platform.system(),
        "platform_version": platform.mac_ver()[0] if platform.system() == "Darwin" else platform.release(),
        "machine": platform.machine(),
        "python": platform.python_version(),
        "host_virtualization": args.host_regime,
        "filesystem": args.filesystem,
        "host_pressure": "uncontrolled",
        "os_cache": "warm-steady after two warmups per arm",
        "pairs_per_cell": args.pairs,
        "arms": {"cold": "cache off; fresh filesystem scan", "warm": "cache auto; exact-scope seeded snapshot and analysis sidecar"},
        "comparison": "same-build cache cost for the same population; not a before/after metadata regression test",
        "fixture_counts": fixtures,
        "cells": cells,
    }
    publish_json(args.output, evidence)


if __name__ == "__main__":
    main()
