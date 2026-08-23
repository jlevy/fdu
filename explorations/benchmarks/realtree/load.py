"""Reproducible CPU background load for interactive-host performance cells."""

from __future__ import annotations

import argparse
import hashlib
import multiprocessing
import signal
import sys
import time
from typing import Any, Sequence


def _burn(stop: Any) -> None:
    payload = b"fdu-controlled-interactive-load-v1" * 256
    digest = payload
    while not stop.is_set():
        digest = hashlib.sha256(digest + payload).digest()


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(prog="benchmarks.realtree.load")
    parser.add_argument("--workers", type=int, required=True)
    arguments = parser.parse_args(list(argv))
    if arguments.workers < 1 or arguments.workers > 64:
        parser.error("--workers must be between 1 and 64")

    stop = multiprocessing.Event()
    processes = [
        multiprocessing.Process(target=_burn, args=(stop,), daemon=True)
        for _ in range(arguments.workers)
    ]
    for process in processes:
        process.start()

    def request_stop(_signum: int, _frame: object) -> None:
        stop.set()

    signal.signal(signal.SIGTERM, request_stop)
    signal.signal(signal.SIGINT, request_stop)
    print("ready", flush=True)
    while not stop.is_set() and all(process.is_alive() for process in processes):
        time.sleep(0.1)
    stop.set()
    for process in processes:
        process.join(timeout=5)
    return 0 if all(process.exitcode in {0, None} for process in processes) else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
