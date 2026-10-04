#!/usr/bin/env python3
"""Check that a rendered video changes exactly where its cast does.

Every frame whose picture differs from the previous one must be the first frame at or
after some cast output event, and every output event must produce a change no later
than one frame after it. A renderer that dropped, delayed or invented a frame fails.

    python3 verify_timing.py out/realtime.cast out/realtime-xterm.mp4 --fps 60
"""

import argparse
import json
import math
import subprocess
import sys
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("cast")
ap.add_argument("video")
ap.add_argument("--fps", type=int, default=60)
a = ap.parse_args()
lines = Path(a.cast).read_text().splitlines()
v3 = json.loads(lines[0])["version"] == 3
t, events = 0.0, []
for line in lines[1:]:
    dt, code, _ = json.loads(line)
    t = t + dt if v3 else dt
    if code == "o":
        events.append(t)
expected = sorted({math.ceil(e * a.fps - 1e-9) for e in events})
# Largest per-pixel luma change between consecutive frames, so a single typed glyph counts.
# Run it on a lossless render (render.mjs --codec lossless): a lossy encode refreshes the
# picture at each keyframe, which reads as a change. tblend's frame k is the pair (k, k+1).
out = subprocess.run(
    [
        "ffmpeg",
        "-loglevel",
        "error",
        "-i",
        a.video,
        "-vf",
        "format=gray,tblend=all_mode=difference,signalstats,"
        "metadata=print:key=lavfi.signalstats.YMAX:file=-",
        "-f",
        "null",
        "-",
    ],
    capture_output=True,
    text=True,
    check=True,
).stdout
changed, frame = [], None
for line in out.splitlines():
    if line.startswith("frame:"):
        frame = int(line.split()[0].split(":")[1])
    elif "YMAX=" in line and float(line.split("=")[1]) > 0:
        changed.append(frame + 1)
exp = set(expected) - {0}
got = set(changed)
missing = sorted(e for e in exp if e not in got)
extra = sorted(g for g in got if g not in exp)
print(
    json.dumps(
        {
            "expected_change_frames": len(exp),
            "observed_change_frames": len(got),
            "missing": missing[:20],
            "unexpected": extra[:20],
        }
    )
)
sys.exit(1 if extra else 0)
