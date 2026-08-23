"""Command-line interface for deterministic performance corpus operations."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence

from benchmarks.corpus import (
    CORPUS_NAME,
    MANIFEST_NAME,
    CorpusError,
    apply_transition,
    cleanup_run_directory,
    create_corpus,
    load_recipe,
    reserve_run_directory,
    verify_corpus,
)


def build_parser() -> argparse.ArgumentParser:
    """Build the corpus command parser."""
    parser = argparse.ArgumentParser(
        prog="python -m benchmarks.generate",
        description=(
            "Create and verify deterministic fdu performance corpora. "
            "All output is structured JSON."
        ),
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    create = subparsers.add_parser(
        "create", help="reserve a marked run directory and create one corpus"
    )
    create.add_argument("--recipe", required=True, help="committed recipe identifier")
    create.add_argument(
        "--work-dir",
        required=True,
        type=Path,
        help="existing or creatable parent for the unique run directory",
    )
    create.add_argument(
        "--entries",
        type=_positive_int,
        help="override the recipe's required descendant count",
    )
    create.add_argument("--seed", help="override the deterministic recipe seed")

    verify = subparsers.add_parser(
        "verify", help="verify a marked run against its stored semantic manifest"
    )
    verify.add_argument("--run-root", required=True, type=Path)

    mutate = subparsers.add_parser(
        "mutate", help="apply the next declared mutation transition"
    )
    mutate.add_argument("--run-root", required=True, type=Path)
    mutate.add_argument("--transition", required=True)

    cleanup = subparsers.add_parser(
        "cleanup", help="remove exactly one validated marked run directory"
    )
    cleanup.add_argument("--run-root", required=True, type=Path)
    return parser


def main(arguments: Optional[Sequence[str]] = None) -> int:
    """Run one corpus operation and emit a single JSON result."""
    parser = build_parser()
    namespace = parser.parse_args(arguments)
    try:
        result = _run(namespace)
    except CorpusError as error:
        _write_json(
            sys.stderr,
            {
                "action": namespace.command,
                "error": str(error),
                "error_type": "corpus_error",
                "ok": False,
            },
        )
        return 2
    except OSError as error:
        _write_json(
            sys.stderr,
            {
                "action": namespace.command,
                "errno": error.errno,
                "error": error.strerror or str(error),
                "error_type": "filesystem_error",
                "ok": False,
            },
        )
        return 2
    _write_json(sys.stdout, result)
    return 0


def _run(namespace: argparse.Namespace) -> Dict[str, Any]:
    if namespace.command == "create":
        load_recipe(
            namespace.recipe,
            target_entries=namespace.entries,
            seed=namespace.seed,
        )
        run_root = reserve_run_directory(namespace.work_dir)
        manifest = create_corpus(
            run_root,
            namespace.recipe,
            target_entries=namespace.entries,
            seed=namespace.seed,
        )
        return _manifest_result("create", run_root, manifest)
    if namespace.command == "verify":
        manifest = verify_corpus(namespace.run_root)
        return _manifest_result("verify", namespace.run_root.resolve(), manifest)
    if namespace.command == "mutate":
        manifest = apply_transition(namespace.run_root, namespace.transition)
        return _manifest_result("mutate", namespace.run_root.resolve(), manifest)
    if namespace.command == "cleanup":
        resolved = namespace.run_root.resolve(strict=True)
        cleanup_run_directory(namespace.run_root)
        return {"action": "cleanup", "ok": True, "removed_run_root": str(resolved)}
    raise CorpusError(f"unknown corpus command: {namespace.command!r}")


def _manifest_result(
    action: str, run_root: Path, manifest: Dict[str, Any]
) -> Dict[str, Any]:
    return {
        "action": action,
        "corpus_root": str(run_root / CORPUS_NAME),
        "counts": manifest["counts"],
        "manifest_hash": manifest["manifest_hash"],
        "manifest_path": str(run_root / MANIFEST_NAME),
        "ok": True,
        "recipe_id": manifest["recipe_id"],
        "run_root": str(run_root),
        "semantic_digest": manifest["semantic_digest"],
        "state": manifest["state"],
    }


def _positive_int(value: str) -> int:
    try:
        parsed = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("must be an integer") from error
    if parsed < 1:
        raise argparse.ArgumentTypeError("must be greater than zero")
    return parsed


def _write_json(output: Any, value: Dict[str, Any]) -> None:
    json.dump(value, output, ensure_ascii=True, sort_keys=True)
    output.write("\n")


if __name__ == "__main__":
    raise SystemExit(main())
