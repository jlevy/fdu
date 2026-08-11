"""Measure snapshot loading across deterministic wide-directory scale points."""

from __future__ import annotations

import hashlib
import json
import time
from pathlib import Path
from typing import Any, Dict, List, Sequence

from benchmarks import corpus
from benchmarks.realtree import measure

SCALE_SCHEMA = "fdu-snapshot-load-scale-v1"
DEFAULT_SCALES = (10_000, 100_000, 500_000, 1_000_000)


def run(
    *,
    variant: measure.Variant,
    work_directory: Path,
    scales: Sequence[int] = DEFAULT_SCALES,
    trials: int = 5,
    warmups: int = 1,
) -> Dict[str, Any]:
    """Generate, verify, load, and clean one wide corpus at a time."""
    if trials < 1 or warmups < 0:
        raise measure.MeasureError("scale trials must be positive and warmups non-negative")
    frozen_identity = measure._freeze_variant_identities([variant])[variant.name]
    work_directory.mkdir(parents=True, exist_ok=True)
    started = time.time()
    rows: List[Dict[str, Any]] = []
    schedule: List[Dict[str, Any]] = []
    for scale in scales:
        rows.append(
            _measure_scale(
                variant=variant,
                work_directory=work_directory,
                entries=scale,
                trials=trials,
                warmups=warmups,
                schedule=schedule,
            )
        )
    schedule_encoded = json.dumps(
        schedule, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    measure._assert_variants_unchanged([variant], {variant.name: frozen_identity})
    return {
        "schema": SCALE_SCHEMA,
        "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(started)),
        "duration_seconds": round(time.time() - started, 3),
        "host": measure.host_facts(),
        "conditions": {
            "filesystem_cache": "warm-steady",
            "order": "ascending-scale-v1",
            "schedule_seed": None,
            "schedule_sha256": hashlib.sha256(schedule_encoded).hexdigest(),
            "trials": trials,
            "warmups": warmups,
        },
        "variant": frozen_identity,
        "scales": rows,
    }


def _measure_scale(
    *,
    variant: measure.Variant,
    work_directory: Path,
    entries: int,
    trials: int,
    warmups: int,
    schedule: List[Dict[str, Any]],
) -> Dict[str, Any]:
    run_root = corpus.reserve_run_directory(work_directory)
    try:
        manifest = corpus.create_corpus(
            run_root,
            "wide-snapshot",
            target_entries=entries,
        )
        root = run_root / corpus.CORPUS_NAME
        snapshot = run_root / "scale.fdu"
        save = measure._spawn(
            [
                str(variant.path),
                "snapshot-save",
                "--root",
                str(root),
                "--snapshot",
                str(snapshot),
                *variant.extra_args,
            ],
            timeout_seconds=measure.DEFAULT_TIMEOUT_SECONDS,
        )
        save_probe, save_reasons = measure._read_probe_output(save["stdout"])
        save_reasons.extend(_oracle_reasons(manifest, save_probe, mode="snapshot-save"))
        if save["exit_code"] != 0 or save_reasons or not snapshot.is_file():
            raise measure.MeasureError(
                f"cannot prepare wide-{entries} snapshot: "
                + "; ".join(save_reasons or [f"exit {save['exit_code']}"])
            )

        samples: List[Dict[str, Any]] = []
        for ordinal in range(-warmups, trials):
            warmup = ordinal < 0
            schedule.append(
                {"entries": entries, "ordinal": ordinal, "warmup": warmup}
            )
            outcome = measure._spawn(
                [
                    str(variant.path),
                    "snapshot-load",
                    "--root",
                    str(root),
                    "--snapshot",
                    str(snapshot),
                    *variant.extra_args,
                ],
                timeout_seconds=measure.DEFAULT_TIMEOUT_SECONDS,
            )
            probe, reasons = measure._read_probe_output(outcome["stdout"])
            reasons.extend(_oracle_reasons(manifest, probe, mode="snapshot-load"))
            if outcome["exit_code"] != 0:
                reasons.append(f"command exited with {outcome['exit_code']}")
            metrics = measure._process_metrics(
                outcome["resources"],
                wall_ns=outcome["wall_ns"],
                process_cpu_can_exceed_wall=False,
            )
            metrics["component_ns"] = probe.get("component_ns")
            samples.append(
                {
                    "ordinal": ordinal,
                    "warmup": warmup,
                    "valid": not reasons,
                    "reasons": reasons,
                    "metrics": metrics,
                }
            )

        corpus.verify_corpus(run_root)
        measured = [sample for sample in samples if not sample["warmup"] and sample["valid"]]
        return {
            "entries": entries,
            "corpus": {
                "recipe_id": manifest["recipe_id"],
                "recipe_hash": manifest["recipe_hash"],
                "manifest_hash": manifest["manifest_hash"],
                "engine_digest": manifest["oracle"]["engine_digest"],
                "rollup_digest": manifest["oracle"]["rollup_digest"],
                "counts": manifest["counts"],
            },
            "invalid_samples": sum(
                not sample["valid"] for sample in samples if not sample["warmup"]
            ),
            "samples": samples,
            "statistics": {
                metric: measure.distribution(
                    [
                        sample["metrics"][metric]
                        for sample in measured
                        if sample["metrics"].get(metric) is not None
                    ]
                )
                for metric in ("wall_ns", "component_ns", "cpu_ns", "peak_rss_bytes")
            },
        }
    finally:
        corpus.cleanup_run_directory(run_root)


def _oracle_reasons(
    manifest: Dict[str, Any], probe: Dict[str, Any], *, mode: str
) -> List[str]:
    summary = probe.get("summary") if isinstance(probe, dict) else None
    if not isinstance(summary, dict):
        return ["probe emitted no summary"]
    expected = {
        "mode": mode,
        "source": "snapshot" if mode == "snapshot-load" else "scan",
        "entries": manifest["counts"]["total"],
        "index_len": manifest["counts"]["total"],
        "apparent_bytes": manifest["sizes"]["entry_apparent_bytes"],
        "engine_digest": manifest["oracle"]["engine_digest"],
        "rollup_digest": manifest["oracle"]["rollup_digest"],
    }
    reasons: List[str] = []
    for field_name in ("mode", "source"):
        if probe.get(field_name) != expected[field_name]:
            reasons.append(
                f"probe {field_name}={probe.get(field_name)!r}, "
                f"expected {expected[field_name]!r}"
            )
    for field_name in (
        "entries",
        "index_len",
        "apparent_bytes",
        "engine_digest",
        "rollup_digest",
    ):
        if summary.get(field_name) != expected[field_name]:
            reasons.append(
                f"probe {field_name}={summary.get(field_name)!r}, "
                f"expected {expected[field_name]!r}"
            )
    return reasons


__all__ = ["DEFAULT_SCALES", "SCALE_SCHEMA", "run"]
