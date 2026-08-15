"""Verify an isolated installed fdu command and its effective shell resolution."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, Mapping, Sequence

from benchmarks.realtree import compare_tools, provenance, tree


SCHEMA = "fdu-installed-command-v1"


class InstallationError(RuntimeError):
    """An installed command cannot support the declared release surface."""


def capture(
    *,
    executable: Path,
    artifact_label: str,
    kind: str,
    root: Path,
    provenance_document: Mapping[str, Any],
    shells: Sequence[str] = ("bash", "zsh"),
) -> Dict[str, Any]:
    """Prove direct execution, clean-shell lookup, and a real cache-off scan."""
    _validate_provenance_envelope(provenance_document, require_claim_grade=False)
    resolved = executable.resolve(strict=True)
    if resolved.name != "fdu":
        raise InstallationError("the installed command must be named fdu")
    artifact = (provenance_document.get("artifacts") or {}).get(artifact_label)
    if not isinstance(artifact, Mapping):
        raise InstallationError("provenance does not contain the installed artifact")
    expected_kind = {"native-cargo": "native-fdu", "python-wheel": "python-fdu"}.get(kind)
    if expected_kind is None or artifact.get("kind") != expected_kind:
        raise InstallationError("installed surface kind disagrees with provenance")
    expected_file = next(
        (
            entry
            for entry in artifact.get("files", [])
            if isinstance(entry, Mapping) and entry.get("name") == resolved.name
        ),
        None,
    )
    executable_sha256 = provenance._sha256_file(resolved)
    if not isinstance(expected_file, Mapping) or expected_file.get("sha256") != executable_sha256:
        raise InstallationError("installed command hash disagrees with provenance")

    before = tree.fingerprint(root, label="installed-command-smoke")
    stdout, stderr, exit_code = _run(
        [
            str(resolved),
            "--cache",
            "off",
            "--view",
            "summary",
            "--format",
            "json",
            "--color",
            "never",
            str(root.resolve(strict=True)),
        ]
    )
    semantic_sha256, summary, semantic_error = compare_tools._summary_semantics(stdout)
    reasons = []
    if provenance_document.get("claim_grade") is not True:
        provenance_reasons = provenance_document.get("invalidation_reasons") or [
            "provenance is not claim-grade"
        ]
        reasons.extend(f"provenance: {reason}" for reason in provenance_reasons)
    if exit_code != 0:
        reasons.append(f"cache-off smoke command exited with {exit_code}")
    if stderr.strip():
        reasons.append("cache-off smoke command emitted stderr")
    if semantic_error is not None:
        reasons.append(semantic_error)
    elif summary is not None:
        oracle_error = compare_tools._summary_oracle_error(summary, before)
        if oracle_error is not None:
            reasons.append(oracle_error)
    after = tree.fingerprint(root, label="installed-command-smoke")
    mutation = tree.compare(after, before)
    if mutation:
        reasons.append("smoke scan subject changed during verification")

    shell_results: Dict[str, Any] = {}
    for shell in shells:
        shell_result = _shell_resolution(shell, resolved)
        shell_results[shell] = shell_result
        if shell_result.get("matches") is not True:
            reasons.append(f"{shell} clean noninteractive resolution did not select the artifact")
        diagnostic = shell_result.get("diagnostic_interactive")
        if not isinstance(diagnostic, Mapping) or diagnostic.get("matches") is not True:
            reasons.append(f"{shell} interactive login resolution shadows the artifact")

    document: Dict[str, Any] = {
        "artifact_label": artifact_label,
        "claim_grade": not reasons,
        "executable": {
            "name": resolved.name,
            "sha256": executable_sha256,
            "size_bytes": resolved.stat().st_size,
            "version": provenance._version(resolved),
        },
        "invalidation_reasons": reasons,
        "kind": kind,
        "provenance_manifest_id": provenance_document.get("manifest_id"),
        "schema": SCHEMA,
        "shells": shell_results,
        "smoke": {
            "engine_digest": before["engine_digest"],
            "root_id": before["root_id"],
            "semantic_sha256": semantic_sha256,
            "tree_mutation": mutation,
        },
    }
    document["attestation_id"] = _attestation_id(document)
    return document


def verify(
    document: Mapping[str, Any],
    *,
    executable: Path,
    root: Path,
    provenance_document: Mapping[str, Any],
    require_claim_grade: bool = True,
) -> Dict[str, Any]:
    _validate_provenance_envelope(provenance_document, require_claim_grade=require_claim_grade)
    if document.get("schema") != SCHEMA:
        raise InstallationError(f"unexpected installation schema {document.get('schema')!r}")
    if document.get("attestation_id") != _attestation_id(document):
        raise InstallationError("installation attestation identity does not match")
    if require_claim_grade and document.get("claim_grade") is not True:
        raise InstallationError(
            "installation attestation is exploratory: "
            + "; ".join(map(str, document.get("invalidation_reasons") or []))
        )
    if document.get("provenance_manifest_id") != provenance_document.get("manifest_id"):
        raise InstallationError("installation and provenance manifests disagree")
    artifact_label = document.get("artifact_label")
    artifact = (provenance_document.get("artifacts") or {}).get(artifact_label)
    expected_kind = {
        "native-cargo": "native-fdu",
        "python-wheel": "python-fdu",
    }.get(document.get("kind"))
    if not isinstance(artifact, Mapping) or artifact.get("kind") != expected_kind:
        raise InstallationError("installation surface no longer matches its provenance artifact")
    resolved = executable.resolve(strict=True)
    recorded = document.get("executable") or {}
    if recorded.get("sha256") != provenance._sha256_file(resolved):
        raise InstallationError("installed command hash no longer matches")
    if recorded.get("version") != provenance._version(resolved):
        raise InstallationError("installed command version no longer matches")
    fingerprint = tree.fingerprint(root, label="installed-command-verify")
    smoke = document.get("smoke") or {}
    if fingerprint["root_id"] != smoke.get("root_id"):
        raise InstallationError("installed-command subject root changed")
    if fingerprint["engine_digest"] != smoke.get("engine_digest"):
        raise InstallationError("installed-command subject contents changed")
    shells = document.get("shells")
    if not isinstance(shells, Mapping) or not shells:
        raise InstallationError("installed-command shell evidence is missing")
    for shell_name in shells:
        current = _shell_resolution(str(shell_name), resolved)
        if current.get("matches") is not True:
            raise InstallationError(f"{shell_name} clean shell no longer selects the artifact")
        diagnostic = current.get("diagnostic_interactive")
        if not isinstance(diagnostic, Mapping) or diagnostic.get("matches") is not True:
            raise InstallationError(f"{shell_name} interactive login shell shadows the artifact")
    return dict(document)


def _validate_provenance_envelope(
    document: Mapping[str, Any], *, require_claim_grade: bool
) -> None:
    if document.get("schema") != provenance.SCHEMA:
        raise InstallationError("installation requires the versioned provenance contract")
    if document.get("manifest_id") != provenance._manifest_id(document):
        raise InstallationError("installation provenance identity does not match")
    if require_claim_grade and document.get("claim_grade") is not True:
        raise InstallationError("installation requires claim-grade provenance")


def load_and_verify(
    path: Path,
    *,
    executable: Path,
    root: Path,
    provenance_document: Mapping[str, Any],
    require_claim_grade: bool = True,
) -> Dict[str, Any]:
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise InstallationError(f"cannot read installation attestation: {error}") from error
    if not isinstance(document, dict):
        raise InstallationError("installation attestation is not a JSON object")
    return verify(
        document,
        executable=executable,
        root=root,
        provenance_document=provenance_document,
        require_claim_grade=require_claim_grade,
    )


def _shell_resolution(shell_name: str, executable: Path) -> Dict[str, Any]:
    shell = shutil.which(shell_name)
    if shell is None:
        return {
            "available": False,
            "matches": None,
            "reason": f"{shell_name} is unavailable",
        }
    standard_path = os.pathsep.join([str(executable.parent), "/usr/local/bin", "/usr/bin", "/bin"])
    if shell_name == "bash":
        clean_argv = [shell, "--noprofile", "--norc", "-c", "command -v fdu"]
    elif shell_name == "zsh":
        clean_argv = [shell, "-f", "-c", "command -v fdu"]
    else:
        clean_argv = [shell, "-c", "command -v fdu"]
    completed = subprocess.run(
        clean_argv,
        stdin=subprocess.DEVNULL,
        capture_output=True,
        env={"HOME": "/nonexistent", "LANG": "C", "PATH": standard_path},
        check=False,
    )
    output = completed.stdout.decode("utf-8", errors="replace").strip()
    selected = Path(output).resolve(strict=False) if output.startswith("/") else None
    matches = completed.returncode == 0 and selected == executable

    diagnostic_environment = dict(os.environ)
    diagnostic_environment["PATH"] = os.pathsep.join(
        [str(executable.parent), diagnostic_environment.get("PATH", "")]
    )
    marker = "__FDU_EFFECTIVE_RESOLUTION__="
    try:
        diagnostic = subprocess.run(
            [
                shell,
                "-lic",
                f'resolution=$(command -v fdu); printf "{marker}%s\\n" "$resolution"',
            ],
            stdin=subprocess.DEVNULL,
            capture_output=True,
            check=False,
            timeout=10,
            env=diagnostic_environment,
        )
        diagnostic_bytes = diagnostic.stdout + diagnostic.stderr
        resolutions = [
            line.removeprefix(marker)
            for line in diagnostic.stdout.decode("utf-8", errors="replace").splitlines()
            if line.startswith(marker)
        ]
        diagnostic_selected = (
            Path(resolutions[0]).resolve(strict=False)
            if len(resolutions) == 1 and resolutions[0].startswith("/")
            else None
        )
        diagnostic_matches = diagnostic.returncode == 0 and diagnostic_selected == executable
        diagnostic_exit_code = diagnostic.returncode
        diagnostic_reason = (
            None
            if diagnostic_matches
            else "interactive login shell selected an alias, function, or different path"
        )
    except subprocess.TimeoutExpired as error:
        diagnostic_bytes = (error.stdout or b"") + (error.stderr or b"")
        diagnostic_matches = False
        diagnostic_exit_code = None
        diagnostic_reason = "interactive login shell resolution timed out"
    return {
        "available": True,
        "clean_mode": "noninteractive-no-startup-files",
        "diagnostic_interactive": {
            "bytes": len(diagnostic_bytes),
            "exit_code": diagnostic_exit_code,
            "matches": diagnostic_matches,
            "reason": diagnostic_reason,
            "sha256": hashlib.sha256(diagnostic_bytes).hexdigest(),
        },
        "matches": matches,
        "reason": None if matches else "clean shell selected a different command",
        "resolved_sha256": provenance._sha256_file(selected)
        if matches and selected is not None
        else None,
    }


def _run(argv: Sequence[str]) -> tuple[bytes, str, int]:
    completed = subprocess.run(
        list(argv),
        stdin=subprocess.DEVNULL,
        capture_output=True,
        env={
            "HOME": "/nonexistent",
            "LANG": "C",
            "NO_COLOR": "1",
            "PATH": "/usr/local/bin:/usr/bin:/bin",
        },
        check=False,
        timeout=600,
    )
    return (
        completed.stdout,
        completed.stderr.decode("utf-8", errors="replace"),
        completed.returncode,
    )


def _attestation_id(document: Mapping[str, Any]) -> str:
    return provenance._sha256_json(
        {key: value for key, value in document.items() if key != "attestation_id"}
    )


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(prog="benchmarks.realtree.installed_command")
    subparsers = parser.add_subparsers(dest="command", required=True)
    captured = subparsers.add_parser("capture")
    captured.add_argument("--executable", type=Path, required=True)
    captured.add_argument("--artifact-label", required=True)
    captured.add_argument("--kind", choices=("native-cargo", "python-wheel"), required=True)
    captured.add_argument("--root", type=Path, required=True)
    captured.add_argument("--provenance", type=Path, required=True)
    captured.add_argument("--output", type=Path, required=True)
    captured.add_argument("--shell", action="append", default=[])
    captured.add_argument("--require-claim-grade", action="store_true")

    checked = subparsers.add_parser("verify")
    checked.add_argument("--executable", type=Path, required=True)
    checked.add_argument("--root", type=Path, required=True)
    checked.add_argument("--provenance", type=Path, required=True)
    checked.add_argument("--attestation", type=Path, required=True)
    arguments = parser.parse_args(list(argv))
    try:
        provenance_document = json.loads(arguments.provenance.read_text(encoding="utf-8"))
        if arguments.command == "capture":
            document = capture(
                executable=arguments.executable,
                artifact_label=arguments.artifact_label,
                kind=arguments.kind,
                root=arguments.root,
                provenance_document=provenance_document,
                shells=arguments.shell or ("bash", "zsh"),
            )
            if arguments.require_claim_grade and not document["claim_grade"]:
                raise InstallationError(
                    "installation is exploratory: " + "; ".join(document["invalidation_reasons"])
                )
            arguments.output.parent.mkdir(parents=True, exist_ok=True)
            arguments.output.write_text(
                json.dumps(document, indent=2, sort_keys=True), encoding="utf-8"
            )
        else:
            document = load_and_verify(
                arguments.attestation,
                executable=arguments.executable,
                root=arguments.root,
                provenance_document=provenance_document,
            )
    except (
        InstallationError,
        OSError,
        provenance.ProvenanceError,
        tree.ReferenceTreeError,
        json.JSONDecodeError,
    ) as error:
        parser.error(str(error))
    print(
        json.dumps(
            {
                "attestation_id": document["attestation_id"],
                "claim_grade": document["claim_grade"],
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
