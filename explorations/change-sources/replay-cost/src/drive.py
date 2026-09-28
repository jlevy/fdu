"""Run a batch of replaycost cells sequentially and append one enriched JSON record per
cell to a JSONL file. Intended to be launched as `timing-lock python3 drive.py ...` so the
whole batch runs under one short lock hold.

usage: drive.py OUT_JSONL BATCH_NAME 'label|arg arg arg...' ...
Each cell's args are split on whitespace and passed to bin/replaycost. Use --label in
the args or the label before '|' (the driver adds --label if absent).
A cell whose args start with 'sleep N' just sleeps. A cell starting with 'cpuwatch N'
samples fseventsd CPU once per second for N seconds and records the series.
A cell starting with 'bg ' launches the rest as a background process (for concurrency
cells) and the next cell runs immediately; background processes are waited for at the
end of the batch and their stdout is recorded.
"""

from __future__ import annotations

import json
import os
import shlex
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
BIN = os.path.join(HERE, "..", "bin", "replaycost")


def fseventsd_cpu() -> float:
    try:
        pid = subprocess.check_output(["/usr/bin/pgrep", "-x", "fseventsd"], text=True).split()[0]
        t = subprocess.check_output(["/bin/ps", "-o", "time=", "-p", pid], text=True).strip()
        parts = t.split(":")
        if len(parts) == 3:
            return int(parts[0]) * 3600 + int(parts[1]) * 60 + float(parts[2])
        return int(parts[0]) * 60 + float(parts[1])
    except Exception:
        return -1.0


def loadavg() -> list[float]:
    return list(os.getloadavg())


def main() -> None:
    out, batch = sys.argv[1], sys.argv[2]
    cells = sys.argv[3:]
    background: list[tuple[str, subprocess.Popen, float]] = []
    batch_start = time.time()
    with open(out, "a") as f:
        for order, cell in enumerate(cells):
            label, _, args = cell.partition("|")
            argv = shlex.split(args)
            rec: dict = {"batch": batch, "order": order, "label": label, "args": args,
                         "started_at": time.time(), "loadavg": loadavg(), "cpu_before": fseventsd_cpu()}
            if argv and argv[0] == "sleep":
                time.sleep(float(argv[1]))
                rec["kind"] = "sleep"
            elif argv and argv[0] == "shell":
                # e.g. start or stop a load generator; output recorded, not parsed
                p = subprocess.run(["/bin/bash", "-c", args.partition(" ")[2]], capture_output=True, text=True)
                rec["kind"] = "shell"
                rec["rc"] = p.returncode
                rec["stdout"] = p.stdout[-300:]
                rec["stderr"] = p.stderr[-300:]
            elif argv and argv[0] == "cpuwatch":
                n = int(argv[1])
                series = []
                for _ in range(n):
                    series.append(round(fseventsd_cpu(), 2))
                    time.sleep(1)
                rec["kind"] = "cpuwatch"
                rec["series"] = series
            elif argv and argv[0] == "bg":
                cmd = [BIN] + argv[1:]
                if "--label" not in cmd:
                    cmd += ["--label", label]
                p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                background.append((label, p, time.time()))
                rec["kind"] = "bg-start"
                rec["pid"] = p.pid
            else:
                cmd = [BIN] + argv
                if "--label" not in cmd:
                    cmd += ["--label", label]
                t0 = time.time()
                p = subprocess.run(cmd, capture_output=True, text=True)
                rec["kind"] = "run"
                rec["wall_s"] = round(time.time() - t0, 3)
                rec["rc"] = p.returncode
                rec["stderr"] = p.stderr[-500:]
                lines = [ln for ln in p.stdout.splitlines() if ln.startswith("{")]
                rec["event_lines"] = sum(1 for ln in lines if '"type":"event"' in ln)
                try:
                    rec["result"] = json.loads(lines[-1]) if lines else None
                except json.JSONDecodeError:
                    rec["result"] = None
                    rec["raw_tail"] = p.stdout[-300:]
            rec["cpu_after"] = fseventsd_cpu()
            rec["finished_at"] = time.time()
            f.write(json.dumps(rec) + "\n")
            f.flush()
            r = rec.get("result") or {}
            print(f"[{batch}:{order}] {label:36s} kind={rec.get('kind')} wall={rec.get('wall_s')} "
                  f"hist_ms={r.get('history_ms')} events={r.get('events')} cpu_d={r.get('fseventsd_cpu_delta')} "
                  f"done={r.get('history_done')} rc={rec.get('rc')}", flush=True)
        for label, p, t0 in background:
            outp, err = p.communicate()
            lines = [ln for ln in outp.splitlines() if ln.startswith("{")]
            rec = {"batch": batch, "order": -1, "label": label, "kind": "bg-end", "started_at": t0,
                   "finished_at": time.time(), "wall_s": round(time.time() - t0, 3), "rc": p.returncode,
                   "stderr": err[-500:], "event_lines": sum(1 for ln in lines if '"type":"event"' in ln)}
            try:
                rec["result"] = json.loads(lines[-1]) if lines else None
            except json.JSONDecodeError:
                rec["result"] = None
            f.write(json.dumps(rec) + "\n")
            r = rec.get("result") or {}
            print(f"[{batch}:bg] {label:36s} wall={rec['wall_s']} hist_ms={r.get('history_ms')} events={r.get('events')} cpu_d={r.get('fseventsd_cpu_delta')} done={r.get('history_done')} rc={p.returncode}", flush=True)
    print(f"batch {batch} took {time.time() - batch_start:.1f}s", flush=True)


if __name__ == "__main__":
    main()
