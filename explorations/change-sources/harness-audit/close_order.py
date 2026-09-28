"""Discriminator for the open-writer miss: is FSE_CONTENT_MODIFIED generated at close, not lagged?

prepare ROOT : create aged leaves a1, a2, a3 (let them age >= 10 minutes before `run`).
run ROOT PROBE : fence -> append while open (a1), sibling create (b), replay while open,
                 close a1, replay; plus a2 (second independent descriptor closed while
                 a long-lived one stays open) and a3 (dup'd descriptor, only one dup closed).
Output is JSON with relative fixture paths only (synthetic fixture on external scratch).
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path

FLAGS = 144


def probe(helper: str, *args: str) -> list[dict]:
    res = subprocess.run([helper, *args], capture_output=True, text=True, timeout=60, check=False)
    recs = [json.loads(line) for line in res.stdout.splitlines()]
    if res.returncode:
        raise RuntimeError(f"probe rc={res.returncode} {res.stderr}")
    return recs


def events(recs: list[dict]) -> list[dict]:
    out = []
    for r in recs:
        if r.get("type") != "event" or r["flags"] & 0x10:
            continue
        out.append({"id": r["id"], "flags": hex(r["flags"]), "path": bytes.fromhex(r["path_hex"]).decode() if r["inside"] else "(outside)", "ahd": r["after_history_done"]})
    return out


def summary(recs: list[dict]) -> dict:
    s = [r for r in recs if r.get("type") == "summary"][-1]
    return {k: s[k] for k in ("history_done", "timeout", "history_ms", "flush_ms", "events", "latest_id", "history_id")}


def prepare(root: Path) -> None:
    root.mkdir(parents=True, exist_ok=False)
    for name in ("a1", "a2", "a3"):
        (root / name).write_bytes(b"baseline\n")
    (root / "created_at").write_text(str(time.time()))
    print(json.dumps({"prepared": str(root), "at": time.time()}))


def run(root: Path, helper: str) -> None:
    created_at = float((root / "created_at").read_text())
    age = time.time() - created_at
    out: dict = {"leaf_age_s": age, "steps": []}
    fence = probe(helper, "capture", str(root))[0]
    cursor = int(fence["device_fence"])
    out["fence"] = {"device_fence": cursor, "global_now": fence["global_now"], "at": time.time()}
    payload = b"append while open\n" * 1024

    # a1: long-lived descriptor, append+fsync, sibling created afterwards, replay while open.
    fd1 = os.open(root / "a1", os.O_RDWR | os.O_APPEND)
    os.write(fd1, payload)
    os.fsync(fd1)
    t_write = time.time()
    time.sleep(5)  # defeat any fseventsd processing lag
    (root / "b-after-write").write_bytes(b"sibling\n")
    time.sleep(1)
    r = probe(helper, "replay", str(root), str(cursor), str(FLAGS))
    out["steps"].append({"step": "replay-while-a1-open(+5s, sibling b created after the write)", "summary": summary(r), "events": events(r)})
    id_b = max([e["id"] for e in events(r) if e["path"] == "b-after-write"], default=None)
    time.sleep(1)
    os.close(fd1)
    t_close = time.time()
    time.sleep(1)
    r = probe(helper, "replay", str(root), str(cursor), str(FLAGS))
    ev = events(r)
    id_a1 = [e["id"] for e in ev if e["path"] == "a1"]
    out["steps"].append({"step": "replay-after-a1-close", "summary": summary(r), "events": ev,
                         "a1_event_ids": id_a1, "b_event_id": id_b,
                         "a1_id_greater_than_b": (bool(id_a1) and id_b is not None and min(id_a1) > id_b),
                         "write_to_close_s": t_close - t_write})

    # a2: keep one descriptor open; write through a second, independently opened descriptor and close it.
    keep = os.open(root / "a2", os.O_RDONLY)
    fd2 = os.open(root / "a2", os.O_RDWR | os.O_APPEND)
    os.write(fd2, payload)
    os.fsync(fd2)
    os.close(fd2)
    time.sleep(1)
    r = probe(helper, "replay", str(root), str(cursor), str(FLAGS))
    out["steps"].append({"step": "a2: independent writer fd closed while another fd stays open", "summary": summary(r), "a2_events": [e for e in events(r) if e["path"] == "a2"]})
    os.close(keep)

    # a3: dup'd descriptor; write via original, close original, keep the dup open.
    fd3 = os.open(root / "a3", os.O_RDWR | os.O_APPEND)
    dup3 = os.dup(fd3)
    os.write(fd3, payload)
    os.fsync(fd3)
    os.close(fd3)
    time.sleep(1)
    r = probe(helper, "replay", str(root), str(cursor), str(FLAGS))
    out["steps"].append({"step": "a3: original fd closed, dup still open (same open file description)", "summary": summary(r), "a3_events": [e for e in events(r) if e["path"] == "a3"]})
    os.close(dup3)
    time.sleep(1)
    r = probe(helper, "replay", str(root), str(cursor), str(FLAGS))
    out["steps"].append({"step": "a3: last dup closed", "summary": summary(r), "a3_events": [e for e in events(r) if e["path"] == "a3"]})
    print(json.dumps(out, indent=1))


if __name__ == "__main__":
    if sys.argv[1] == "prepare":
        prepare(Path(sys.argv[2]))
    else:
        run(Path(sys.argv[2]), sys.argv[3])
