"""Command line for the real-tree performance loop.

    python -m benchmarks.realtree baseline --root TREE --label NAME
    python -m benchmarks.realtree measure  --root TREE --label NAME \
        --variant baseline=path/to/perf_probe --variant candidate=path/to/perf_probe
    python -m benchmarks.realtree profile  --root TREE --job cold-scan-index
    python -m benchmarks.realtree render   --run results/run.json

See ``docs/project/guides/performance-loop.md`` for the workflow these commands
implement and the rule that decides whether a change is kept.
"""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
import time
from pathlib import Path
from typing import Any, Dict, List, Sequence

from benchmarks import corpus as corpus_tools
from benchmarks.realtree import ledger, measure, profile, provenance, tree
from benchmarks.realtree import subjects as subjects_module

DEFAULT_RESULTS = Path(tempfile.gettempdir()) / "fdu-realtree" / "results"
DEFAULT_SCRATCH = Path(tempfile.gettempdir()) / "fdu-realtree" / "scratch"


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(prog="benchmarks.realtree", description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    baseline = subparsers.add_parser(
        "baseline", help="record a reference tree's redacted fingerprint"
    )
    baseline.add_argument("--root", required=True, type=Path)
    baseline.add_argument("--label", required=True)
    baseline.add_argument("--output", type=Path)

    run = subparsers.add_parser("measure", help="measure variants against a tree")
    run.add_argument("--root", required=True, type=Path)
    run.add_argument("--label", required=True)
    run.add_argument(
        "--variant",
        action="append",
        required=True,
        metavar="NAME=PATH[:NOTES]",
        help="a probe binary under test; repeat, control first",
    )
    run.add_argument(
        "--reference",
        action="append",
        default=[],
        metavar="NAME=PATH",
        help="a third-party tool measured for context only",
    )
    run.add_argument(
        "--job",
        action="append",
        default=[],
        choices=sorted(measure.PROBE_JOBS),
        help="repeat; defaults to the cold and warm pair",
    )
    run.add_argument("--trials", type=int, default=measure.DEFAULT_TRIALS)
    run.add_argument("--warmups", type=int, default=measure.DEFAULT_WARMUPS)
    run.add_argument("--baseline-fingerprint", type=Path)
    run.add_argument(
        "--corpus-manifest",
        type=Path,
        help="independently verify and bind a generated corpus manifest to this run",
    )
    run.add_argument(
        "--provenance-manifest",
        type=Path,
        help="verified clean probe/build/host manifest; required for held-out evidence",
    )
    run.add_argument("--scratch", type=Path, default=DEFAULT_SCRATCH)
    run.add_argument("--output-dir", type=Path, default=DEFAULT_RESULTS)
    run.add_argument("--name", default="", help="short slug for the output files")
    run.add_argument("--note", default="")
    run.add_argument(
        "--stage",
        choices=("exploratory", "discovery", "held-out"),
        default="exploratory",
        help="evidence stage; only held-out runs can confirm a selected arm",
    )
    run.add_argument(
        "--host-regime",
        choices=sorted(measure.HOST_REGIMES),
        default="uncontrolled",
        help="predeclared host-pressure cell; held-out evidence must be controlled",
    )
    run.add_argument(
        "--background-load-workers",
        type=int,
        default=2,
        help="CPU load workers for the controlled-interactive host regime",
    )
    run.add_argument(
        "--purge",
        action="store_true",
        help="drop the OS page cache before every trial; needs root",
    )

    profiled = subparsers.add_parser(
        "profile", help="attribute time to functions on a profiling build"
    )
    profiled.add_argument("--root", required=True, type=Path)
    profiled.add_argument(
        "--binary",
        type=Path,
        default=Path("target/profiling/examples/perf_probe"),
    )
    profiled.add_argument("--job", action="append", default=[], choices=sorted(measure.PROBE_JOBS))
    profiled.add_argument("--seconds", type=int, default=profile.DEFAULT_SAMPLE_SECONDS)
    profiled.add_argument("--repeat", type=int, default=40)
    profiled.add_argument(
        "--counters",
        choices=("enabled", "disabled"),
        default="enabled",
        help="collect logical counters during sampling; disable when timers perturb stacks",
    )
    profiled.add_argument(
        "--oracle",
        choices=("enabled", "disabled"),
        default="enabled",
        help="run the probe oracle on every repeat or omit it for labelled attribution only",
    )
    profiled.add_argument(
        "--extra-arg",
        action="append",
        default=[],
        help="probe flag appended after the job argv; use --extra-arg=--flag",
    )
    profiled.add_argument("--scratch", type=Path, default=DEFAULT_SCRATCH)
    profiled.add_argument("--output", type=Path)
    profiled.add_argument("--label", default="")

    subjects = subparsers.add_parser(
        "subjects", help="observe and check this host's nominated real-tree set"
    )
    subjects.add_argument("--nominations", type=Path, default=None)
    subjects.add_argument("--out", type=Path, default=None)
    subjects.add_argument("--check", type=Path, default=None)

    rendered = subparsers.add_parser("render", help="render a stored run as Markdown")
    rendered.add_argument("--run", required=True, type=Path)
    rendered.add_argument("--profiles", type=Path)
    rendered.add_argument("--output", type=Path)

    arguments = parser.parse_args(list(argv))
    if arguments.command == "baseline":
        return _baseline(arguments)
    if arguments.command == "measure":
        return _measure(arguments)
    if arguments.command == "profile":
        return _profile(arguments)
    if arguments.command == "subjects":
        return _subjects(arguments)
    return _render(arguments)


def _subjects(arguments: argparse.Namespace) -> int:
    """Delegate to the subjects module, which owns its own argument shape."""
    forwarded = []
    if arguments.nominations is not None:
        forwarded += ["--nominations", str(arguments.nominations)]
    if arguments.out is not None:
        forwarded += ["--out", str(arguments.out)]
    if arguments.check is not None:
        forwarded += ["--check", str(arguments.check)]
    return subjects_module.main(forwarded)


def _baseline(arguments: argparse.Namespace) -> int:
    destination = arguments.output or (DEFAULT_RESULTS / f"tree-{arguments.label}.json")
    _require_external(arguments.root, destination, description="baseline output")
    document = tree.fingerprint(arguments.root, label=arguments.label)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(document, indent=2, sort_keys=True), encoding="utf-8")
    print(f"wrote {destination}", file=sys.stderr)
    print(json.dumps({key: document[key] for key in ("counts", "sizes", "engine_digest")}))
    return 0


def _measure(arguments: argparse.Namespace) -> int:
    _require_external(arguments.root, arguments.scratch, description="scratch directory")
    _require_external(arguments.root, arguments.output_dir, description="result directory")
    variants = [_variant(item, kind="fdu-probe") for item in arguments.variant]
    references = [_variant(item, kind="reference") for item in arguments.reference]
    jobs = [
        measure.PROBE_JOBS[job] for job in (arguments.job or ["cold-scan-index", "warm-revalidate"])
    ]

    baseline_document = (
        json.loads(arguments.baseline_fingerprint.read_text(encoding="utf-8"))
        if arguments.baseline_fingerprint
        else None
    )
    corpus_document = (
        _verified_corpus_manifest(arguments.root, arguments.corpus_manifest)
        if arguments.corpus_manifest
        else None
    )
    provenance_document = (
        _verified_measurement_provenance(
            arguments.provenance_manifest,
            arguments.root,
            variants,
        )
        if arguments.provenance_manifest
        else None
    )

    try:
        document = measure.run(
            root=arguments.root,
            label=arguments.label,
            variants=variants,
            jobs=jobs,
            trials=arguments.trials,
            warmups=arguments.warmups,
            scratch=arguments.scratch,
            baseline_fingerprint=baseline_document,
            purge=arguments.purge,
            note=arguments.note,
            campaign_stage=arguments.stage,
            corpus=corpus_document,
            host_regime=arguments.host_regime,
            background_load_workers=arguments.background_load_workers,
            provenance_document=provenance_document,
        )
    except measure.MeasureError as error:
        # A refused cell is the harness doing its job -- the host was not quiet, the tree
        # moved, a snapshot could not be prepared -- and it should read as a verdict
        # about the run, not as a traceback from the harness. An operator, or an agent
        # at 3 a.m., retries when the stated condition clears.
        print(f"measurement refused: {error}", file=sys.stderr)
        return 1

    if arguments.corpus_manifest:
        verified_after = _verified_corpus_manifest(arguments.root, arguments.corpus_manifest)
        if verified_after != corpus_document:
            raise SystemExit("generated-corpus manifest changed during measurement")

    if references:
        document["reference_tools"] = _measure_references(
            references,
            root=arguments.root,
            trials=arguments.trials,
            warmups=arguments.warmups,
        )

    slug = arguments.name or time.strftime("%Y%m%d-%H%M%S", time.gmtime())
    arguments.output_dir.mkdir(parents=True, exist_ok=True)
    run_path = arguments.output_dir / f"run-{slug}.json"
    report_path = arguments.output_dir / f"run-{slug}.md"
    run_path.write_text(json.dumps(document, indent=2, sort_keys=True), encoding="utf-8")
    ledger.write(document, report_path)

    print(f"\nwrote {run_path}\nwrote {report_path}", file=sys.stderr)
    _print_headline(document)
    if document["tree_mutated_during_run"] or document["baseline_drift"]:
        return 2
    if document["invalid_samples"]:
        return 3
    return 0


def _verified_corpus_manifest(root: Path, manifest_path: Path) -> Dict[str, Any]:
    """Bind a path-free generated-corpus contract to a real-tree run."""
    run_root = manifest_path.resolve(strict=True).parent
    expected_root = (run_root / corpus_tools.CORPUS_NAME).resolve(strict=True)
    if root.resolve(strict=True) != expected_root:
        raise SystemExit("--corpus-manifest does not describe the measured --root")
    manifest = corpus_tools.verify_corpus(run_root)
    return {
        "capabilities": manifest["capabilities"],
        "counts": manifest["counts"],
        "manifest_hash": manifest["manifest_hash"],
        "oracle": {
            "engine_digest": manifest["oracle"]["engine_digest"],
            "engine_digest_algorithm": manifest["oracle"]["engine_digest_algorithm"],
        },
        "recipe": manifest["recipe"],
        "recipe_hash": manifest["recipe_hash"],
        "recipe_id": manifest["recipe_id"],
        "schema": manifest["schema"],
        "semantic_digest": manifest["semantic_digest"],
        "state": manifest["state"],
        "topology": manifest["topology"],
    }


def _verified_measurement_provenance(
    manifest_path: Path,
    root: Path,
    variants: Sequence[measure.Variant],
) -> Dict[str, Any]:
    """Match every measured probe hash to a verified provenance artifact."""
    try:
        document = json.loads(manifest_path.read_text(encoding="utf-8"))
        artifacts = document.get("artifacts") if isinstance(document, dict) else None
        if not isinstance(artifacts, dict):
            raise provenance.ProvenanceError("provenance artifacts are missing")
        selected: Dict[str, Path] = {}
        for variant in variants:
            identity = variant.identity()
            matches = [
                label
                for label, artifact in artifacts.items()
                if isinstance(artifact, dict)
                and artifact.get("kind") == "fdu-perf-probe"
                and any(
                    isinstance(file, dict)
                    and file.get("name") == variant.path.name
                    and file.get("sha256") == identity["sha256"]
                    for file in artifact.get("files", [])
                )
            ]
            if len(matches) != 1:
                raise provenance.ProvenanceError(
                    f"variant {variant.name!r} matches {len(matches)} perf-probe artifacts"
                )
            selected[matches[0]] = variant.path
        return provenance.verify(
            document,
            source_root=provenance.PROJECT_ROOT,
            subject_root=root,
            artifacts=selected,
            require_claim_grade=True,
        )
    except (
        OSError,
        UnicodeDecodeError,
        json.JSONDecodeError,
        provenance.ProvenanceError,
    ) as error:
        raise SystemExit(f"cannot verify measurement provenance: {error}") from error


def _measure_references(
    references: Sequence[measure.Variant],
    *,
    root: Path,
    trials: int,
    warmups: int,
) -> Dict[str, Any]:
    """Time third-party tools on the same tree, in their own table.

    They are not variants of fdu and cannot be compared to it entry for entry: each
    answers a slightly different question with slightly different guarantees. What
    they establish is the order of magnitude a mature tool achieves on this hardware,
    which is the only thing that makes an fdu number mean anything.
    """
    results: Dict[str, Any] = {}
    for reference in references:
        argv = measure.REFERENCE_ARGV.get(reference.path.name)
        if argv is None:
            argv = ("{binary}", "{root}")
        samples: List[int] = []
        cpu: List[int] = []
        for ordinal in range(-warmups, trials):
            expanded = measure._expand(argv, binary=reference.path, root=root, snapshot=None)
            outcome = measure._spawn(expanded, timeout_seconds=measure.DEFAULT_TIMEOUT_SECONDS)
            if ordinal < 0 or outcome["exit_code"] != 0:
                continue
            samples.append(outcome["wall_ns"])
            resources = outcome["resources"]
            if resources["user_cpu_ns"] is not None:
                cpu.append(resources["user_cpu_ns"] + resources["system_cpu_ns"])
        results[reference.name] = {
            "argv": list(argv),
            "identity": reference.identity(),
            "wall_ns": measure.distribution(samples),
            "cpu_ns": measure.distribution(cpu),
        }
    return results


def _profile(arguments: argparse.Namespace) -> int:
    jobs = [
        measure.PROBE_JOBS[job] for job in (arguments.job or ["cold-scan-index", "warm-revalidate"])
    ]
    destination = arguments.output or (
        DEFAULT_RESULTS / f"profile-{arguments.label or 'latest'}.json"
    )
    _require_external(arguments.root, arguments.scratch, description="scratch directory")
    _require_external(arguments.root, destination, description="profile output")
    arguments.scratch.mkdir(parents=True, exist_ok=True)
    variant = measure.Variant(name="profiling", path=arguments.binary)
    snapshots = measure._prepare_snapshots(
        arguments.root, [variant], jobs, arguments.scratch, measure.DEFAULT_TIMEOUT_SECONDS
    )

    captured: List[Dict[str, Any]] = []
    for job in jobs:
        argv = measure._expand(
            job.argv,
            binary=arguments.binary,
            root=arguments.root,
            snapshot=snapshots.get((variant.name, job.id)),
            extra=arguments.extra_arg,
        )
        entry = profile.capture(
            binary=arguments.binary,
            argv=argv,
            seconds=arguments.seconds,
            repeat=arguments.repeat,
            label=f"{arguments.label or 'profile'} / {job.id}",
            require_diagnostics=job.require_scan_diagnostics,
            counters_enabled=arguments.counters == "enabled",
            oracle_enabled=arguments.oracle == "enabled",
        )
        if job.require_scan_diagnostics:
            probe_document = entry.get("probe") or {}
            probe_summary = probe_document.get("summary") or {}
            diagnostic_reasons = measure._validate_scan_diagnostics(
                probe_document.get("scan_diagnostics"),
                require_macos_backend=sys.platform == "darwin",
                expected_dirs_read=probe_summary.get("dirs_read"),
            )
            if diagnostic_reasons:
                raise profile.ProfileError(
                    "profile companion diagnostics are invalid: " + "; ".join(diagnostic_reasons)
                )
        captured.append(entry)
        print(f"\n=== {entry['label']} ({entry['total_samples']:,} samples) ===")
        for layer in entry["by_layer"]:
            print(f"  {layer['percent']:6.2f}%  {layer['layer']}")
        print("  ---")
        for frame in entry["self_time"][:14]:
            print(f"  {frame['percent']:6.2f}%  {frame['symbol'][:80]}")

    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(captured, indent=2, sort_keys=True), encoding="utf-8")
    print(f"\nwrote {destination}", file=sys.stderr)
    return 0


def _require_external(root: Path, path: Path, *, description: str) -> None:
    """Reject harness state inside the tree whose immutability it is proving."""
    subject = root.resolve(strict=True)
    candidate = path.resolve(strict=False)
    if candidate == subject or subject in candidate.parents:
        raise SystemExit(f"{description} must be outside the measured --root")


def _render(arguments: argparse.Namespace) -> int:
    document = ledger.load(arguments.run)
    profiles = (
        json.loads(arguments.profiles.read_text(encoding="utf-8")) if arguments.profiles else ()
    )
    text = ledger.render(document, profiles=profiles)
    if arguments.output:
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(text, encoding="utf-8")
        print(f"wrote {arguments.output}", file=sys.stderr)
    else:
        print(text)
    return 0


def _variant(specification: str, *, kind: str) -> measure.Variant:
    """Parse ``NAME=PATH[ ARG...][:NOTES]``.

    Flags after the path let one binary be measured under several configurations in
    the same interleaved run, which is how thread counts and batch sizes get compared
    fairly rather than across separate runs on a drifting machine.
    """
    name, separator, remainder = specification.partition("=")
    if not separator:
        raise SystemExit(f"variant {specification!r} must be NAME=PATH")
    command, _separator, notes = remainder.partition(":")
    parts = command.split()
    if not parts:
        raise SystemExit(f"variant {name!r} has no binary")
    resolved = Path(parts[0]).resolve()
    if not resolved.is_file():
        raise SystemExit(f"variant {name!r} binary does not exist: {resolved}")
    return measure.Variant(
        name=name,
        path=resolved,
        kind=kind,
        notes=notes,
        extra_args=parts[1:],
    )


def _print_headline(document: Dict[str, Any]) -> None:
    for job_id, statistics in document["statistics"].items():
        print(f"\n{job_id}")
        for name, entry in statistics["variants"].items():
            wall = entry["metrics"].get("wall_ns")
            component = entry["metrics"].get("component_ns")
            if wall:
                print(
                    f"  {name:<16} wall {wall['median'] / 1e6:8.1f} ms"
                    + (f"   component {component['median'] / 1e6:8.1f} ms" if component else "")
                    + f"   (n={entry['samples']})"
                )
        for key, comparison in statistics["comparisons"].items():
            decision = ledger.verdict(comparison)
            print(
                f"  {key}: {'ACCEPT' if decision['accepted'] else 'REJECT'} — {decision['reason']}"
            )
    for name, entry in (document.get("reference_tools") or {}).items():
        if entry["wall_ns"]:
            print(f"\nreference {name}: wall {entry['wall_ns']['median'] / 1e6:8.1f} ms")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
