#!/usr/bin/env python3
"""A deliberately broken fdu for the correctness runbook: nothing is ever stored.

Name this file as `FDU_BIN`, and the real binary as `FDU_REAL`. It passes every argument
to the real fdu, but turns `--cache on` into `--cache off`, so no snapshot is written.
Over the refusal-free tree, `warm_cold.py` and `cross_warm.py` must then both exit 1:
each metadata case reports `NO-SNAPSHOT`, each analysis case and each same-analyzer pair
`NOT-WARM(scanned)`. A pass that still succeeds against this cannot see a cache that
never serves; see the runbook's "The Check That Makes the Rest Worth Running".
"""

from __future__ import annotations

import os
import sys


def broken(args: list[str]) -> list[str]:
    """The arguments with every request to store a snapshot turned off."""
    out = []
    for index, arg in enumerate(args):
        if arg == "on" and index > 0 and args[index - 1] == "--cache":
            arg = "off"
        elif arg == "--cache=on":
            arg = "--cache=off"
        out.append(arg)
    return out


def main() -> None:
    real = os.environ.get("FDU_REAL")
    if not real:
        sys.exit("FDU_REAL must name the real fdu binary")
    os.execv(real, [real, *broken(sys.argv[1:])])


if __name__ == "__main__":
    main()
