"""The path-independence matrix against the builds under test.

Environment:
    FDU_BIN          absolute path to the fdu executable (default: target/debug/fdu)
    FDU_PYTHON       absolute path to a Python with the fdu wheel installed
    FDU_PI_TIER      subset (default) or full
    FDU_PI_SURFACES  cli, or cli,python (default)
    FDU_PI_OUT       directory for a diff of every failing case (optional)
"""

from __future__ import annotations

import os
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import matrix
import registry
import runner


class PathIndependence(unittest.TestCase):
    """Every answer equals a cold run's, apart from provenance, or is a registered gap."""

    def test_matrix(self) -> None:
        tier = matrix.TIERS[os.environ.get("FDU_PI_TIER", "subset")]
        surface_names = os.environ.get("FDU_PI_SURFACES", "cli,python").split(",")
        surfaces = runner.discover_surfaces(surface_names)
        with tempfile.TemporaryDirectory(prefix="fdu-path-independence-") as scratch:
            result = runner.run_tier(tier, surfaces, Path(scratch))
        out = os.environ.get("FDU_PI_OUT")
        known = registry.load(registry.DEFAULT_PATH)
        failures = runner.judge(result, known, Path(out) if out else None)
        report = runner.summary(result, failures)
        print(report)
        if failures:
            self.fail(report)


if __name__ == "__main__":
    unittest.main()
