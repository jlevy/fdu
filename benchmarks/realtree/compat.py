"""Generate a review-base probe without changing the engine under measurement.

PR #3's base predates ``ScanConfig.threads`` but otherwise exposes every API used by
the v2 semantic probe. The claim-grade cumulative comparison does not pass a thread
override, so removing that one parser arm produces a source-compatible probe while
leaving every timed and oracle path identical. The transformation is exact and fails
if the source drifts.
"""

from __future__ import annotations

from pathlib import Path


class CompatibilityError(RuntimeError):
    """The compatibility probe could not be generated exactly."""


_THREAD_ARM = """                Some(\"--threads\") => {
                    scan.threads = Some(next_usize(&mut arguments, \"--threads\")?);
                }
"""


def write_pr2_base_probe(source: Path, destination: Path) -> None:
    """Remove only the unused post-base thread-override parser arm."""
    text = source.read_text(encoding="utf-8")
    if text.count(_THREAD_ARM) != 1:
        raise CompatibilityError(
            "perf_probe thread parser changed; review the compatibility transform"
        )
    generated = text.replace(_THREAD_ARM, "", 1)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(generated, encoding="utf-8")


__all__ = ["CompatibilityError", "write_pr2_base_probe"]
