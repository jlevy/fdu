#!/usr/bin/env python3
"""Record a scripted terminal demo as an asciicast, running every command for real.

A scenario (TOML) lists steps. Each step is typed at a human cadence, then executed in the
recording's own PTY, so what the cast holds is the command's real output at its real speed:
colors, progress lines and timing come from the program, not from a mock. Only the typing
is synthetic, and the cast says so in its header.

The run has two halves:

- The outer half starts `asciinema rec --headless` with this script as the recorded
  command (`--inner`), at a fixed window size.
- The inner half, inside the PTY, prints a prompt, types each command, runs it, and emits
  an in-band marker (an OSC sequence no terminal acts on) at each step boundary. It also
  writes a sidecar with each command's measured wall time.

Afterwards the outer half rewrites the cast: markers become asciicast `m` events (chapter
points in the web player), the OSC bytes are removed, and a receipt joins the sidecar to
the cast so a viewer can check that the playback speed is the measured speed.

Usage:
    python3 record.py scenarios/realtime.toml out/realtime.cast
"""

from __future__ import annotations

import argparse
import json
import os
import random
import re
import shutil
import signal
import subprocess
import sys
import time
import tomllib
from pathlib import Path

MARKER = re.compile(r"\x1b\]1337;fdu-demo-marker=([^\x07]*)\x07")
ESC = "\x1b"

# Typing cadence, in seconds. Deterministic per scenario seed so re-recording a scenario
# changes only what the commands themselves print and how long they take.
TYPE_BASE = 0.045
TYPE_JITTER = 0.035
TYPE_SPACE_EXTRA = 0.04
PAUSE_BEFORE_ENTER = 0.35
SHELL = "/bin/bash"


def fdu_bin_dir() -> str:
    """The fdu under demonstration: `FDU_DEMO_BIN`, else this checkout's release build."""
    explicit = os.environ.get("FDU_DEMO_BIN")
    if explicit:
        return explicit
    target = os.environ.get("CARGO_TARGET_DIR") or str(Path(__file__).resolve().parents[2] / "target")
    return str(Path(target) / "release")


def out(text: str) -> None:
    sys.stdout.write(text)
    sys.stdout.flush()


def marker(label: str) -> None:
    out(f"{ESC}]1337;fdu-demo-marker={label}\x07")


def type_text(text: str, rng: random.Random, speed: float) -> None:
    for ch in text:
        out(ch)
        delay = TYPE_BASE + rng.uniform(0, TYPE_JITTER)
        if ch == " ":
            delay += TYPE_SPACE_EXTRA
        time.sleep(delay / speed)


def run_inner(scenario_path: Path, sidecar: Path) -> int:
    scenario = tomllib.loads(scenario_path.read_text())
    rng = random.Random(scenario.get("seed", 1))
    speed = float(scenario.get("typing_speed", 1.0))
    prompt = scenario.get("prompt", f"{ESC}[1;32m❯{ESC}[0m ")
    cwd = os.path.expandvars(os.path.expanduser(scenario.get("cwd", ".")))
    env = dict(os.environ, **scenario.get("env", {}))
    env["PATH"] = fdu_bin_dir() + os.pathsep + env["PATH"]
    records = []

    for setup in scenario.get("setup", []):
        subprocess.run(setup, shell=True, executable=SHELL, cwd=cwd, env=env, check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    time.sleep(scenario.get("lead_in", 0.6))
    for index, step in enumerate(scenario["step"]):
        for hidden in step.get("before", []):
            subprocess.run(hidden, shell=True, executable=SHELL, cwd=cwd, env=env, check=True,
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        label = step.get("label", f"step {index + 1}")
        marker(label)
        if step.get("clear"):
            out(f"{ESC}[H{ESC}[2J{ESC}[3J")
        if "comment" in step:
            out(prompt)
            out(f"{ESC}[2m")
            type_text(step["comment"], rng, speed)
            out(f"{ESC}[0m\r\n")
            time.sleep(step.get("hold", 0.5))
            continue
        out(prompt)
        type_text(step["cmd"], rng, speed)
        time.sleep(PAUSE_BEFORE_ENTER)
        out("\r\n")
        record = {"label": label, "cmd": step["cmd"]}
        started = time.monotonic()
        proc = subprocess.Popen(step["cmd"], shell=True, executable=SHELL, cwd=cwd, env=env)
        concurrent = []
        for action in step.get("during", []):
            concurrent.append((started + action["at"], action["run"]))
        stop_at = started + step["stop_after"] if "stop_after" in step else None
        while proc.poll() is None:
            now = time.monotonic()
            while concurrent and concurrent[0][0] <= now:
                _, script = concurrent.pop(0)
                subprocess.Popen(script, shell=True, executable=SHELL, cwd=cwd, env=env,
                                 stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            if stop_at is not None and now >= stop_at:
                proc.send_signal(signal.SIGINT)
                stop_at = None
            time.sleep(0.005)
        record["wall_seconds"] = round(time.monotonic() - started, 4)
        record["exit"] = proc.returncode
        records.append(record)
        time.sleep(step.get("hold", 1.5))

    marker("end")
    out(prompt)
    time.sleep(scenario.get("tail", 2.0))
    sidecar.write_text(json.dumps(records, indent=2) + "\n")
    return 0


def postprocess(raw: Path, final: Path, sidecar: Path, scenario: dict) -> dict:
    lines = raw.read_text().splitlines()
    header = json.loads(lines[0])
    header["title"] = scenario.get("title", "")
    header["env"] = {"TERM": "xterm-256color"}
    header.pop("command", None)
    events = []
    clock = 0.0
    pending = 0.0  # interval owed to the next emitted event when one is dropped
    for line in lines[1:]:
        interval, code, data = json.loads(line)
        clock += interval
        if code != "o":
            events.append([round(interval + pending, 6), code, data])
            pending = 0.0
            continue
        pieces = MARKER.split(data)
        # split() alternates text, label, text, label, ...
        first = True
        for i, piece in enumerate(pieces):
            if i % 2 == 1:
                events.append([round((interval + pending) if first else 0.0, 6), "m", piece])
                first, pending = False, 0.0
            elif piece:
                events.append([round((interval + pending) if first else 0.0, 6), "o", piece])
                first, pending = False, 0.0
        if first:
            pending += interval
    with final.open("w") as fh:
        fh.write(json.dumps(header) + "\n")
        for event in events:
            fh.write(json.dumps(event) + "\n")

    marks, t = {}, 0.0
    for interval, code, data in events:
        t += interval
        if code == "m":
            marks.setdefault(data, round(t, 4))
    records = json.loads(sidecar.read_text())
    for record in records:
        record["cast_seconds"] = marks.get(record["label"])
    return {
        "cast": final.name,
        "title": header["title"],
        "window": f'{header["term"]["cols"]}x{header["term"]["rows"]}',
        "duration_seconds": round(t, 4),
        "typing": "synthetic, seeded cadence",
        "command_output": "real execution in the recording PTY, real timing",
        "steps": records,
        "tools": {
            "asciinema": subprocess.run(["asciinema", "--version"], capture_output=True,
                                        text=True).stdout.strip(),
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("scenario", type=Path)
    parser.add_argument("cast", type=Path, nargs="?")
    parser.add_argument("--inner", type=Path, help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.inner:
        return run_inner(args.scenario, args.inner)

    scenario = tomllib.loads(args.scenario.read_text())
    final = args.cast
    final.parent.mkdir(parents=True, exist_ok=True)
    raw = final.with_suffix(".raw.cast")
    sidecar = final.with_suffix(".steps.json")
    size = f'{scenario.get("cols", 100)}x{scenario.get("rows", 30)}'
    inner = f"{sys.executable} {Path(__file__).resolve()} {args.scenario.resolve()} --inner {sidecar.resolve()}"
    asciinema = shutil.which("asciinema") or sys.exit("asciinema 3.x is required on PATH")
    if not Path(fdu_bin_dir(), "fdu").exists():
        sys.exit(f"no fdu binary in {fdu_bin_dir()}: run `cargo build --release -p fdu` or set FDU_DEMO_BIN")
    subprocess.run([asciinema, "rec", "--headless", "--overwrite", "--window-size", size,
                    "--command", inner, str(raw)], check=True,
                   env=dict(os.environ, TERM="xterm-256color"))
    receipt = postprocess(raw, final, sidecar, scenario)
    final.with_suffix(".receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    raw.unlink()
    sidecar.unlink()
    print(json.dumps(receipt, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
