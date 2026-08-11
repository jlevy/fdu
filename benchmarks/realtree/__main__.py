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
import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Dict, List, Sequence

from benchmarks.realtree import compat, evidence, ledger, measure, profile, scale, tree
from benchmarks.realtree import environment as benchmark_environment

DEFAULT_RESULTS = Path("benchmarks/results/realtree")
DEFAULT_SCRATCH = Path("benchmarks/corpus/realtree-scratch")


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
        "--variant-metadata",
        action="append",
        default=[],
        metavar="NAME=JSON",
        help="claim-build provenance manifest for a named variant",
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
    run.add_argument("--scratch", type=Path, default=DEFAULT_SCRATCH)
    run.add_argument("--output-dir", type=Path, default=DEFAULT_RESULTS)
    run.add_argument("--name", default="", help="short slug for the output files")
    run.add_argument("--note", default="")
    run.add_argument(
        "--environment-cell",
        help="stable path-free cell id, for example github-ubuntu-24.04-x64",
    )
    run.add_argument(
        "--runner-class",
        choices=benchmark_environment.RUNNER_CLASSES,
        help="control grade of the runner; inferred as local or GitHub-hosted by default",
    )
    run.add_argument(
        "--run-group",
        help="stable id shared only by equivalent runs in different environments",
    )
    run.add_argument(
        "--corpus-manifest",
        type=Path,
        help="verified observed-corpus.json whose portable identity binds this run",
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
    profiled.add_argument("--scratch", type=Path, default=DEFAULT_SCRATCH)
    profiled.add_argument("--output", type=Path)
    profiled.add_argument("--label", default="")

    rendered = subparsers.add_parser("render", help="render a stored run as Markdown")
    rendered.add_argument("--run", required=True, type=Path)
    rendered.add_argument("--profiles", type=Path)
    rendered.add_argument("--output", type=Path)

    archived = subparsers.add_parser(
        "archive", help="commit-safe copy of a raw run with paired samples"
    )
    archived.add_argument("--run", required=True, type=Path)
    archived.add_argument("--output", required=True, type=Path)
    archived.add_argument("--tree-label", required=True)

    compatible = subparsers.add_parser(
        "compat-probe", help="generate the v2 probe for the pre-threads PR base"
    )
    compatible.add_argument("--source", required=True, type=Path)
    compatible.add_argument("--output", required=True, type=Path)

    scale_run = subparsers.add_parser(
        "snapshot-scale", help="measure v2 snapshot load on wide 10k-1M corpora"
    )
    scale_run.add_argument("--variant", required=True, metavar="NAME=PATH")
    scale_run.add_argument("--variant-metadata", required=True, metavar="NAME=JSON")
    scale_run.add_argument("--work-dir", required=True, type=Path)
    scale_run.add_argument("--output", required=True, type=Path)
    scale_run.add_argument("--scale", action="append", type=int, dest="scales")
    scale_run.add_argument("--trials", type=int, default=5)
    scale_run.add_argument("--warmups", type=int, default=1)

    matrix = subparsers.add_parser(
        "environment-matrix",
        help="compare decisions, not absolute timings, across equivalent environment cells",
    )
    matrix.add_argument("--run", action="append", required=True, type=Path)
    matrix.add_argument("--id", required=True, dest="matrix_id", help="env-NNN")
    matrix.add_argument("--control-variant", required=True)
    matrix.add_argument("--candidate-variant", required=True)
    matrix.add_argument("--output", required=True, type=Path)
    matrix.add_argument("--report", type=Path)
    matrix.add_argument("--max-cpu-regression-pct", type=float, default=10.0)
    matrix.add_argument("--max-rss-regression-pct", type=float, default=10.0)

    provenance = subparsers.add_parser(
        "provenance", help="write one path-redacted claim-build manifest"
    )
    provenance.add_argument("--engine-revision", required=True)
    provenance.add_argument("--harness-revision", required=True)
    provenance.add_argument("--harness-source", required=True, type=Path)
    provenance.add_argument("--build-command", required=True)
    provenance.add_argument("--target")
    provenance.add_argument("--output", required=True, type=Path)

    arguments = parser.parse_args(list(argv))
    if arguments.command == "baseline":
        return _baseline(arguments)
    if arguments.command == "measure":
        return _measure(arguments)
    if arguments.command == "profile":
        return _profile(arguments)
    if arguments.command == "archive":
        return _archive(arguments)
    if arguments.command == "compat-probe":
        return _compat_probe(arguments)
    if arguments.command == "snapshot-scale":
        return _snapshot_scale(arguments)
    if arguments.command == "environment-matrix":
        return _environment_matrix(arguments)
    if arguments.command == "provenance":
        return _provenance(arguments)
    return _render(arguments)


def _baseline(arguments: argparse.Namespace) -> int:
    document = tree.fingerprint(arguments.root, label=arguments.label)
    destination = arguments.output or (
        DEFAULT_RESULTS / f"tree-{arguments.label}.json"
    )
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(document, indent=2, sort_keys=True), encoding="utf-8")
    print(f"wrote {destination}", file=sys.stderr)
    print(json.dumps({key: document[key] for key in ("counts", "sizes", "engine_digest")}))
    return 0


def _measure(arguments: argparse.Namespace) -> int:
    metadata = _variant_metadata(arguments.variant_metadata)
    variants = [
        _variant(item, kind="fdu-probe", provenance=metadata.get(_variant_name(item), {}))
        for item in arguments.variant
    ]
    unknown_metadata = sorted(set(metadata) - {variant.name for variant in variants})
    if unknown_metadata:
        raise SystemExit(
            "variant metadata names no measured variant: " + ", ".join(unknown_metadata)
        )
    references = [_variant(item, kind="reference") for item in arguments.reference]
    jobs = [
        measure.PROBE_JOBS[job]
        for job in (arguments.job or ["cold-scan-index", "warm-revalidate"])
    ]

    baseline_document = (
        json.loads(arguments.baseline_fingerprint.read_text(encoding="utf-8"))
        if arguments.baseline_fingerprint
        else None
    )

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
        environment_cell=arguments.environment_cell,
        runner_class=arguments.runner_class,
        run_group=arguments.run_group,
        corpus_manifest=arguments.corpus_manifest,
    )

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
    return 0 if not document["tree_mutated_during_run"] else 2


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
        measure.PROBE_JOBS[job]
        for job in (arguments.job or ["cold-scan-index", "warm-revalidate"])
    ]
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
        )
        entry = profile.capture(
            binary=arguments.binary,
            argv=argv,
            seconds=arguments.seconds,
            repeat=arguments.repeat,
            label=f"{arguments.label or 'profile'} / {job.id}",
        )
        captured.append(entry)
        print(f"\n=== {entry['label']} ({entry['total_samples']:,} samples) ===")
        for layer in entry["by_layer"]:
            print(f"  {layer['percent']:6.2f}%  {layer['layer']}")
        print("  ---")
        for frame in entry["self_time"][:14]:
            print(f"  {frame['percent']:6.2f}%  {frame['symbol'][:80]}")

    destination = arguments.output or (
        DEFAULT_RESULTS / f"profile-{arguments.label or 'latest'}.json"
    )
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(captured, indent=2, sort_keys=True), encoding="utf-8")
    print(f"\nwrote {destination}", file=sys.stderr)
    return 0


def _render(arguments: argparse.Namespace) -> int:
    document = ledger.load(arguments.run)
    profiles = (
        json.loads(arguments.profiles.read_text(encoding="utf-8"))
        if arguments.profiles
        else ()
    )
    text = ledger.render(document, profiles=profiles)
    if arguments.output:
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(text, encoding="utf-8")
        print(f"wrote {arguments.output}", file=sys.stderr)
    else:
        print(text)
    return 0


def _archive(arguments: argparse.Namespace) -> int:
    document = json.loads(arguments.run.read_text(encoding="utf-8"))
    _archived, digest = evidence.archive_run(
        document,
        destination=arguments.output,
        tree_label=arguments.tree_label,
    )
    print(f"wrote {arguments.output}\nsha256 {digest}", file=sys.stderr)
    return 0


def _compat_probe(arguments: argparse.Namespace) -> int:
    try:
        compat.write_pr2_base_probe(arguments.source, arguments.output)
    except compat.CompatibilityError as error:
        raise SystemExit(str(error)) from error
    print(f"wrote {arguments.output}", file=sys.stderr)
    return 0


def _environment_matrix(arguments: argparse.Namespace) -> int:
    runs = []
    for path in arguments.run:
        encoded = path.read_bytes()
        runs.append(
            benchmark_environment.RunEvidence(
                document=json.loads(encoded),
                artifact_name=path.name,
                artifact_sha256=hashlib.sha256(encoded).hexdigest(),
            )
        )
    try:
        matrix = benchmark_environment.build_matrix(
            runs,
            matrix_id=arguments.matrix_id,
            control_variant=arguments.control_variant,
            candidate_variant=arguments.candidate_variant,
            maximum_cpu_regression_pct=arguments.max_cpu_regression_pct,
            maximum_rss_regression_pct=arguments.max_rss_regression_pct,
        )
    except benchmark_environment.EnvironmentError as error:
        raise SystemExit(str(error)) from error
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(
        json.dumps(
            matrix.model_dump(mode="json", by_alias=True), indent=2, sort_keys=True
        )
        + "\n",
        encoding="utf-8",
    )
    if arguments.report:
        arguments.report.parent.mkdir(parents=True, exist_ok=True)
        arguments.report.write_text(
            benchmark_environment.render_matrix(matrix), encoding="utf-8"
        )
    print(f"wrote {arguments.output}", file=sys.stderr)
    if arguments.report:
        print(f"wrote {arguments.report}", file=sys.stderr)
    print(
        json.dumps(
            {
                "all_cells_valid": matrix.all_cells_valid,
                "decision_consistent": matrix.decision_consistent,
                "divergent_jobs": matrix.divergent_jobs,
            },
            sort_keys=True,
        )
    )
    return 0


def _provenance(arguments: argparse.Namespace) -> int:
    source_digest = hashlib.sha256(arguments.harness_source.read_bytes()).hexdigest()
    manifest = {
        "schema": measure.BINARY_PROVENANCE_SCHEMA,
        "engine_revision": arguments.engine_revision,
        "harness_revision": arguments.harness_revision,
        "harness_sha256": source_digest,
        "target": arguments.target or _rust_target(),
        "build_profile": "release",
        "features": [],
        "build_command": arguments.build_command,
    }
    normalized = measure._validated_provenance(manifest)
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(
        json.dumps(
            {"schema": measure.BINARY_PROVENANCE_SCHEMA, **normalized},
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    print(f"wrote {arguments.output}", file=sys.stderr)
    return 0


def _rust_target() -> str:
    try:
        completed = subprocess.run(
            ["rustc", "-vV"], capture_output=True, check=False, timeout=30
        )
    except (FileNotFoundError, subprocess.TimeoutExpired) as error:
        raise SystemExit(f"cannot discover the Rust target: {error}") from error
    if completed.returncode == 0:
        for line in completed.stdout.decode("utf-8", errors="replace").splitlines():
            if line.startswith("host: "):
                return line.removeprefix("host: ").strip()
    raise SystemExit("rustc -vV did not report a host target")


def _snapshot_scale(arguments: argparse.Namespace) -> int:
    metadata = _variant_metadata([arguments.variant_metadata])
    name = _variant_name(arguments.variant)
    if set(metadata) != {name}:
        raise SystemExit("snapshot-scale metadata name must match its variant")
    variant = _variant(
        arguments.variant,
        kind="fdu-probe",
        provenance=metadata[name],
    )
    document = scale.run(
        variant=variant,
        work_directory=arguments.work_dir,
        scales=arguments.scales or scale.DEFAULT_SCALES,
        trials=arguments.trials,
        warmups=arguments.warmups,
    )
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(
        json.dumps(document, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    invalid = sum(row["invalid_samples"] for row in document["scales"])
    print(f"wrote {arguments.output}; invalid samples: {invalid}", file=sys.stderr)
    return 0 if invalid == 0 else 2


def _variant(
    specification: str, *, kind: str, provenance: Dict[str, Any] | None = None
) -> measure.Variant:
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
        provenance=provenance or {},
    )


def _variant_name(specification: str) -> str:
    name, separator, _remainder = specification.partition("=")
    if not separator or not name:
        raise SystemExit(f"variant {specification!r} must be NAME=PATH")
    return name


def _variant_metadata(specifications: Sequence[str]) -> Dict[str, Dict[str, Any]]:
    metadata: Dict[str, Dict[str, Any]] = {}
    for specification in specifications:
        name, separator, path_text = specification.partition("=")
        if not separator or not name or not path_text:
            raise SystemExit(f"variant metadata {specification!r} must be NAME=JSON")
        if name in metadata:
            raise SystemExit(f"variant metadata for {name!r} was supplied twice")
        path = Path(path_text)
        if not path.is_file():
            raise SystemExit(f"variant metadata file does not exist: {path}")
        document = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(document, dict):
            raise SystemExit(f"variant metadata for {name!r} must be a JSON object")
        metadata[name] = document
    return metadata


def _print_headline(document: Dict[str, Any]) -> None:
    for job_id, statistics in document["statistics"].items():
        print(f"\n{job_id}")
        for name, entry in statistics["variants"].items():
            wall = entry["metrics"].get("wall_ns")
            component = entry["metrics"].get("component_ns")
            if wall:
                print(
                    f"  {name:<16} wall {wall['median'] / 1e6:8.1f} ms"
                    + (
                        f"   component {component['median'] / 1e6:8.1f} ms"
                        if component
                        else ""
                    )
                    + f"   (n={entry['samples']})"
                )
        for key, comparison in statistics["comparisons"].items():
            decision = ledger.verdict(comparison)
            print(
                f"  {key}: {'ACCEPT' if decision['accepted'] else 'REJECT'} — "
                f"{decision['reason']}"
            )
    for name, entry in (document.get("reference_tools") or {}).items():
        if entry["wall_ns"]:
            print(f"\nreference {name}: wall {entry['wall_ns']['median'] / 1e6:8.1f} ms")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
