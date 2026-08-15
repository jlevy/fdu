"""Build Git revisions and measure them as one interleaved benchmark series.

The experiment ledger answers whether one candidate beat its local control.  This
runner answers a different question: how did the same absolute workload move across a
sequence of source revisions?  It archives each commit into a temporary directory,
builds the same release probe with the current toolchain, and gives every resulting
binary to the existing real-tree harness in one interleaved run.

Run ``--dry-run`` first.  It resolves every revision to a full commit and prints the
exact order without building or measuring anything.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Sequence

from benchmarks.realtree import ledger, measure

DEFAULT_RESULTS = Path(tempfile.gettempdir()) / "fdu-realtree" / "revision-series"
DEFAULT_SCRATCH = Path(tempfile.gettempdir()) / "fdu-realtree" / "revision-scratch"
CHECKPOINT_JOBS = (
    "cold-scan-index",
    "cold-scan-producer",
    "cold-snapshot-save",
    "warm-revalidate",
    "warm-snapshot-load",
)
SERIES_SCHEMA = "fdu-revision-series-v1"


class RevisionSeriesError(RuntimeError):
    """A revision sequence could not be resolved, built, or measured."""


@dataclass(frozen=True)
class Revision:
    """One immutable source state in the requested measurement order."""

    commit: str
    committed_unix: int
    subject: str

    def as_json(self) -> dict[str, object]:
        return {
            "commit": self.commit,
            "committed_unix": self.committed_unix,
            "subject": self.subject,
        }


def _run_git(repository: Path, arguments: Sequence[str]) -> str:
    completed = subprocess.run(
        ["git", "-C", str(repository), *arguments],
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip()
        raise RevisionSeriesError(f"git {' '.join(arguments)} failed: {detail}")
    return completed.stdout


def _resolve_one(repository: Path, requested: str) -> Revision:
    if not requested or requested.startswith("-"):
        raise RevisionSeriesError(f"unsafe or empty revision: {requested!r}")
    commit = _run_git(
        repository, ["rev-parse", "--verify", f"{requested}^{{commit}}"]
    ).strip()
    fields = _run_git(
        repository, ["show", "-s", "--format=%H%x00%ct%x00%s", commit]
    ).rstrip("\n").split("\0", maxsplit=2)
    if len(fields) != 3:
        raise RevisionSeriesError(f"could not read commit metadata for {commit}")
    return Revision(commit=fields[0], committed_unix=int(fields[1]), subject=fields[2])


def _range_revisions(
    repository: Path, revision_range: str, paths: Sequence[str]
) -> list[str]:
    if not revision_range or revision_range.startswith("-"):
        raise RevisionSeriesError(f"unsafe or empty revision range: {revision_range!r}")
    command = ["rev-list", "--reverse", "--first-parent", revision_range]
    if paths:
        command.extend(["--", *paths])
    requested = _run_git(repository, command).splitlines()

    # A..B excludes A, but A is the absolute anchor a progression chart needs.
    if ".." in revision_range and "..." not in revision_range:
        anchor = revision_range.split("..", maxsplit=1)[0]
        if anchor:
            requested.insert(0, anchor)
    return requested


def resolve_revisions(
    repository: Path,
    *,
    requested: Sequence[str] = (),
    revision_range: str = "",
    paths: Sequence[str] = (),
) -> list[Revision]:
    """Resolve an explicit list or first-parent range, preserving measurement order."""
    if bool(requested) == bool(revision_range):
        raise RevisionSeriesError(
            "provide either explicit revisions or one --range, but not both"
        )
    names = list(requested) or _range_revisions(repository, revision_range, paths)
    revisions: list[Revision] = []
    seen: set[str] = set()
    for name in names:
        revision = _resolve_one(repository, name)
        if revision.commit in seen:
            continue
        seen.add(revision.commit)
        revisions.append(revision)
    if len(revisions) < 2:
        raise RevisionSeriesError("a revision series needs at least two distinct commits")
    return revisions


def _extract_revision(repository: Path, revision: Revision, destination: Path) -> None:
    destination.mkdir(parents=True)
    archive_path = destination.parent / f"{destination.name}.tar"
    with archive_path.open("wb") as archive:
        completed = subprocess.run(
            ["git", "-C", str(repository), "archive", "--format=tar", revision.commit],
            stdout=archive,
            stderr=subprocess.PIPE,
            check=False,
        )
    if completed.returncode != 0:
        detail = completed.stderr.decode("utf-8", errors="replace").strip()
        raise RevisionSeriesError(f"could not archive {revision.commit}: {detail}")
    with tarfile.open(archive_path) as archive:
        archive.extractall(destination, filter="data")
    archive_path.unlink()


def _tool_version(command: str) -> str:
    completed = subprocess.run(
        [command, "--version"], capture_output=True, text=True, check=False
    )
    if completed.returncode != 0:
        raise RevisionSeriesError(f"{command} --version failed")
    return completed.stdout.strip()


def _build_revisions(
    repository: Path, revisions: Sequence[Revision], workspace: Path
) -> list[measure.Variant]:
    cargo = shutil.which("cargo")
    if cargo is None:
        raise RevisionSeriesError("cargo is required to build revision probes")

    target = workspace / "target"
    binaries = workspace / "binaries"
    binaries.mkdir(parents=True)
    variants: list[measure.Variant] = []
    suffix = ".exe" if os.name == "nt" else ""
    for index, revision in enumerate(revisions):
        name = f"r{index:03d}-{revision.commit[:12]}"
        source = workspace / "sources" / name
        print(
            f"[{index + 1}/{len(revisions)}] archive and build {revision.commit[:12]} "
            f"{revision.subject}",
            file=sys.stderr,
            flush=True,
        )
        _extract_revision(repository, revision, source)
        environment = os.environ.copy()
        environment["CARGO_INCREMENTAL"] = "0"
        completed = subprocess.run(
            [
                cargo,
                "build",
                "--locked",
                "--release",
                "-p",
                "fdu",
                "--example",
                "perf_probe",
                "--no-default-features",
                "--manifest-path",
                str(source / "Cargo.toml"),
                "--target-dir",
                str(target),
            ],
            env=environment,
            check=False,
        )
        if completed.returncode != 0:
            raise RevisionSeriesError(
                f"could not build {revision.commit}; the replay window may predate "
                "the current perf_probe contract"
            )
        built = target / "release" / "examples" / f"perf_probe{suffix}"
        if not built.is_file():
            raise RevisionSeriesError(f"build produced no probe for {revision.commit}")
        immutable = binaries / f"{name}{suffix}"
        shutil.copy2(built, immutable)
        variants.append(
            measure.Variant(
                name=name,
                path=immutable,
                notes=revision.subject,
                source_revision=revision.commit,
            )
        )
    return variants


def _require_outside_tree(root: Path, path: Path, description: str) -> None:
    root_resolved = root.resolve()
    path_resolved = path.resolve()
    if path_resolved == root_resolved or path_resolved.is_relative_to(root_resolved):
        raise RevisionSeriesError(
            f"{description} must stay outside the measured tree: {path_resolved}"
        )


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, default=Path.cwd())
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument(
        "--revision",
        action="append",
        default=[],
        help="commit/tag to benchmark; repeat in measurement order",
    )
    source.add_argument(
        "--range",
        dest="revision_range",
        default="",
        help="first-parent Git range; the left anchor of A..B is included",
    )
    parser.add_argument(
        "--path",
        action="append",
        default=[],
        help="with --range, keep only commits touching this path; repeat as needed",
    )
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--label", required=True)
    parser.add_argument(
        "--job",
        action="append",
        default=[],
        choices=sorted(measure.PROBE_JOBS),
        help="repeat; defaults to the complete index-core-v1 matrix",
    )
    parser.add_argument("--trials", type=int, default=measure.DEFAULT_TRIALS)
    parser.add_argument("--warmups", type=int, default=measure.DEFAULT_WARMUPS)
    parser.add_argument("--baseline-fingerprint", type=Path)
    parser.add_argument("--scratch", type=Path, default=DEFAULT_SCRATCH)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_RESULTS)
    parser.add_argument("--name", default="")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="resolve and print the immutable revision plan without building",
    )
    return parser


def run(argv: Sequence[str]) -> int:
    """Resolve, build, and benchmark one revision series."""
    arguments = _parser().parse_args(list(argv))
    repository = arguments.repository.resolve()
    try:
        revisions = resolve_revisions(
            repository,
            requested=arguments.revision,
            revision_range=arguments.revision_range,
            paths=arguments.path,
        )
        plan = {
            "schema": SERIES_SCHEMA,
            "repository_head": _resolve_one(repository, "HEAD").commit,
            "range": arguments.revision_range or None,
            "paths": list(arguments.path),
            "jobs": list(arguments.job or CHECKPOINT_JOBS),
            "revisions": [revision.as_json() for revision in revisions],
        }
        if arguments.dry_run:
            print(json.dumps(plan, indent=2, sort_keys=True))
            return 0

        _require_outside_tree(arguments.root, arguments.scratch, "scratch directory")
        _require_outside_tree(arguments.root, arguments.output_dir, "result directory")
        baseline = (
            json.loads(arguments.baseline_fingerprint.read_text(encoding="utf-8"))
            if arguments.baseline_fingerprint
            else None
        )
        jobs = [
            measure.PROBE_JOBS[name]
            for name in (arguments.job or CHECKPOINT_JOBS)
        ]
        with tempfile.TemporaryDirectory(prefix="fdu-revision-series-") as directory:
            workspace = Path(directory)
            variants = _build_revisions(repository, revisions, workspace)
            document = measure.run(
                root=arguments.root,
                label=arguments.label,
                variants=variants,
                jobs=jobs,
                trials=arguments.trials,
                warmups=arguments.warmups,
                scratch=arguments.scratch,
                baseline_fingerprint=baseline,
                note="Git revision replay under index-core-v1",
            )
        document["revision_series"] = {
            **plan,
            "build_toolchain": {
                "cargo": _tool_version("cargo"),
                "rustc": _tool_version("rustc"),
                "profile": "release",
                "locked": True,
                "incremental": False,
            },
        }
        slug = arguments.name or time.strftime(
            "revision-series-%Y%m%d-%H%M%S", time.gmtime()
        )
        arguments.output_dir.mkdir(parents=True, exist_ok=True)
        run_path = arguments.output_dir / f"run-{slug}.json"
        report_path = arguments.output_dir / f"run-{slug}.md"
        run_path.write_text(
            json.dumps(document, indent=2, sort_keys=True), encoding="utf-8"
        )
        ledger.write(document, report_path)
        print(f"wrote {run_path}\nwrote {report_path}", file=sys.stderr)
        return 0 if not document["tree_mutated_during_run"] else 2
    except (OSError, ValueError, RevisionSeriesError, json.JSONDecodeError) as error:
        print(f"revision series: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(run(sys.argv[1:]))
