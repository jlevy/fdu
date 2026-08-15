"""Path-redacted, machine-verified provenance for claim-grade performance runs."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import plistlib
import shutil
import subprocess
import sys
import urllib.parse
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Mapping, Optional, Sequence, Tuple

from benchmarks.realtree import measure


SCHEMA = "fdu-claim-provenance-v1"
PROJECT_ROOT = Path(__file__).resolve().parents[2]
ARTIFACT_KINDS = {
    "fdu-perf-probe",
    "homebrew-dust",
    "native-fdu",
    "python-fdu",
}


class ProvenanceError(RuntimeError):
    """A provenance manifest is incomplete, contradictory, or no longer current."""


@dataclass(frozen=True)
class ArtifactSpec:
    """One executable and the files needed to identify its native payload."""

    label: str
    kind: str
    executable: Path
    supporting_files: Tuple[Path, ...] = ()
    build_argv: Tuple[str, ...] = ()


def capture(
    *,
    source_root: Path,
    subject_root: Path,
    artifacts: Sequence[ArtifactSpec],
) -> Dict[str, Any]:
    """Capture an honest manifest; dirty or incomplete inputs remain exploratory."""
    if not artifacts:
        raise ProvenanceError("at least one artifact is required")
    labels = [artifact.label for artifact in artifacts]
    if len(labels) != len(set(labels)):
        raise ProvenanceError("artifact labels must be unique")

    source = _source_facts(source_root)
    host = _host_facts()
    filesystem = _filesystem_facts(subject_root)
    collectors = _collector_facts()
    reasons = []
    captured: Dict[str, Any] = {}
    for artifact in artifacts:
        if artifact.kind not in ARTIFACT_KINDS:
            raise ProvenanceError(f"unsupported artifact kind {artifact.kind!r}")
        entry, entry_reasons = _artifact_facts(artifact, source_root, source)
        captured[artifact.label] = entry
        reasons.extend(f"{artifact.label}: {reason}" for reason in entry_reasons)

    if not source["clean"] and any(
        artifact.kind in {"fdu-perf-probe", "native-fdu", "python-fdu"} for artifact in artifacts
    ):
        reasons.append("fdu source tree is dirty")
    if filesystem.get("type") in {None, ""}:
        reasons.append("measured filesystem type is unavailable")
    if filesystem.get("solid_state") is not True:
        reasons.append("solid-state storage is not established")
    for field in ("arch", "cpu_count", "memory_bytes", "release", "system"):
        if host.get(field) in {None, ""}:
            reasons.append(f"host field {field} is unavailable")
    if host.get("system") == "Darwin":
        for field in ("cpu_model", "performance_cores", "efficiency_cores"):
            if host.get(field) in {None, "", 0}:
                reasons.append(f"Apple host field {field} is unavailable")
        if (host.get("power") or {}).get("source") != "AC":
            reasons.append("AC power is not established")
        if (host.get("thermal") or {}).get("pressure") != "normal":
            reasons.append("normal thermal pressure is not established")
    process_rusage = collectors.get("process_rusage") or {}
    if process_rusage.get("supported") is not True:
        reasons.append("per-process resource collection is unavailable")

    document: Dict[str, Any] = {
        "artifacts": captured,
        "claim_grade": not reasons,
        "collectors": collectors,
        "filesystem": filesystem,
        "host": host,
        "invalidation_reasons": sorted(set(reasons)),
        "schema": SCHEMA,
        "source": source,
    }
    document["manifest_id"] = _manifest_id(document)
    return document


def verify(
    document: Mapping[str, Any],
    *,
    source_root: Path,
    subject_root: Path,
    artifacts: Mapping[str, Path],
    supporting_files: Optional[Mapping[str, Sequence[Path]]] = None,
    require_claim_grade: bool = True,
) -> Dict[str, Any]:
    """Recheck a manifest against current source, host, filesystem, and binaries."""
    if document.get("schema") != SCHEMA:
        raise ProvenanceError(f"unexpected provenance schema {document.get('schema')!r}")
    if document.get("manifest_id") != _manifest_id(document):
        raise ProvenanceError("provenance manifest identity does not match its contents")
    if require_claim_grade and document.get("claim_grade") is not True:
        reasons = document.get("invalidation_reasons") or ["unspecified reason"]
        raise ProvenanceError("provenance is exploratory: " + "; ".join(map(str, reasons)))

    recorded_artifacts = document.get("artifacts")
    if not isinstance(recorded_artifacts, Mapping):
        raise ProvenanceError("provenance artifacts are missing")
    if not set(artifacts).issubset(recorded_artifacts):
        raise ProvenanceError("provided artifact labels are absent from the manifest")
    supporting_files = supporting_files or {}
    for label, executable in artifacts.items():
        recorded = recorded_artifacts[label]
        if not isinstance(recorded, Mapping):
            raise ProvenanceError(f"artifact {label!r} is malformed")
        files = [Path(executable), *map(Path, supporting_files.get(label, ()))]
        observed_hashes = _file_identities(files)
        if observed_hashes != recorded.get("files"):
            raise ProvenanceError(f"artifact {label!r} hashes no longer match")
        if _version(Path(executable)) != recorded.get("version"):
            raise ProvenanceError(f"artifact {label!r} version no longer matches")

    recorded_source = document.get("source")
    current_source = _source_facts(source_root)
    if not isinstance(recorded_source, Mapping):
        raise ProvenanceError("source provenance is missing")
    for field in (
        "commit",
        "remote",
        "cargo_lock_sha256",
        "rust_toolchain",
        "tags_at_commit",
    ):
        if recorded_source.get(field) != current_source.get(field):
            raise ProvenanceError(f"source provenance field {field} no longer matches")
    if recorded_source.get("clean") is not current_source.get("clean"):
        raise ProvenanceError("source provenance cleanliness no longer matches")
    if require_claim_grade and current_source["clean"] is not True:
        raise ProvenanceError("source tree is no longer clean")

    current_host = _host_facts()
    if (document.get("host") or {}).get("class_id") != current_host["class_id"]:
        raise ProvenanceError("anonymous host class no longer matches")
    if require_claim_grade:
        scope_reasons = apple_silicon_apfs_scope_reasons({**dict(document), "host": current_host})
        if scope_reasons:
            raise ProvenanceError(
                "current Apple Silicon/APFS scope is not established: " + "; ".join(scope_reasons)
            )
    current_filesystem = _filesystem_facts(subject_root)
    recorded_filesystem = document.get("filesystem") or {}
    for field in ("type", "block_size", "fragment_size", "flags", "solid_state"):
        if recorded_filesystem.get(field) != current_filesystem.get(field):
            raise ProvenanceError(f"measured filesystem field {field} no longer matches")
    if document.get("collectors") != _collector_facts():
        raise ProvenanceError("collector capabilities or identities no longer match")
    return dict(document)


def load_and_verify(
    path: Path,
    *,
    source_root: Path,
    subject_root: Path,
    artifacts: Mapping[str, Path],
    supporting_files: Optional[Mapping[str, Sequence[Path]]] = None,
    require_claim_grade: bool = True,
) -> Dict[str, Any]:
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ProvenanceError(f"cannot read provenance manifest: {error}") from error
    if not isinstance(document, dict):
        raise ProvenanceError("provenance manifest is not a JSON object")
    return verify(
        document,
        source_root=source_root,
        subject_root=subject_root,
        artifacts=artifacts,
        supporting_files=supporting_files,
        require_claim_grade=require_claim_grade,
    )


def _artifact_facts(
    artifact: ArtifactSpec, source_root: Path, source: Mapping[str, Any]
) -> Tuple[Dict[str, Any], list[str]]:
    executable = artifact.executable.resolve(strict=True)
    supporting = tuple(path.resolve(strict=True) for path in artifact.supporting_files)
    reasons: list[str] = []
    if not os.access(executable, os.X_OK):
        reasons.append("executable bit is not set")
    if any(Path(argument).is_absolute() for argument in artifact.build_argv):
        reasons.append("build argv contains an absolute path instead of a redacted placeholder")
    if not artifact.build_argv:
        reasons.append("exact build argv is missing")
    if artifact.kind == "python-fdu" and not supporting:
        reasons.append("Python console script has no hashed native extension payload")

    version = _version(executable)
    if artifact.kind == "homebrew-dust":
        origin, origin_reasons = _homebrew_dust_facts(executable)
        reasons.extend(origin_reasons)
    else:
        origin = {
            "cargo_lock_sha256": source["cargo_lock_sha256"],
            "license": "MIT",
            "source_clean": source["clean"],
            "source_revision": source["commit"],
            "source_url": source["remote"],
            "target": source["rust_target"],
            "toolchain": source["rust_toolchain"],
        }
        if artifact.kind == "python-fdu":
            uv_lock = source_root / "crates" / "fdu-py" / "uv.lock"
            origin["python_lock_sha256"] = _sha256_file(uv_lock)
        if "tags_at_commit" in source:
            reasons.extend(_fdu_revision_reasons(version, source))
    for field in ("license", "source_revision", "source_url", "target", "toolchain"):
        if origin.get(field) in {None, ""}:
            reasons.append(f"origin field {field} is missing")

    return (
        {
            "build_argv": list(artifact.build_argv),
            "files": _file_identities([executable, *supporting]),
            "kind": artifact.kind,
            "origin": origin,
            "profile": "release",
            "version": version,
        },
        reasons,
    )


def _source_facts(source_root: Path) -> Dict[str, Any]:
    root = source_root.resolve(strict=True)
    if not (root / ".git").exists() and not _git(root, "rev-parse", "--git-dir"):
        raise ProvenanceError("source root is not a Git worktree")
    commit = _git(root, "rev-parse", "HEAD")
    status = _git(root, "status", "--porcelain=v1", "--untracked-files=all")
    remote = _git(root, "remote", "get-url", "origin")
    rust_toolchain = _command(["rustc", "-vV"])
    rust_target = next(
        (
            line.split(":", 1)[1].strip()
            for line in rust_toolchain.splitlines()
            if line.startswith("host:")
        ),
        "",
    )
    return {
        "cargo_lock_sha256": _sha256_file(root / "Cargo.lock"),
        "clean": status == "",
        "commit": commit,
        "remote": _normalize_remote(remote),
        "rust_target": rust_target,
        "rust_toolchain": rust_toolchain,
        "tags_at_commit": sorted(
            tag for tag in _git(root, "tag", "--points-at", "HEAD").splitlines() if tag
        ),
    }


def _fdu_revision_reasons(version: str, source: Mapping[str, Any]) -> list[str]:
    """Bind a development binary to HEAD, or a release binary to an exact tag."""
    commit = source.get("commit")
    tags = source.get("tags_at_commit")
    if not isinstance(commit, str) or len(commit) < 9 or not isinstance(tags, list):
        return ["source revision cannot be matched to the executable version"]
    if f"g{commit[:9]}" in version:
        return []
    version_token = version.rsplit(maxsplit=1)[-1] if version else ""
    if version_token and f"v{version_token}" in tags:
        return []
    return ["executable version does not identify the current source revision"]


def _homebrew_dust_facts(executable: Path) -> Tuple[Dict[str, Any], list[str]]:
    reasons: list[str] = []
    brew = shutil.which("brew")
    if brew is None:
        return {}, ["Homebrew is unavailable, so dust origin cannot be verified"]
    try:
        document = json.loads(_command([brew, "info", "--json=v2", "dust"]))
        formula = document["formulae"][0]
        stable = formula["urls"]["stable"]
        installed = formula["installed"]
        bottle_files = formula["bottle"]["stable"]["files"]
    except (KeyError, IndexError, TypeError, json.JSONDecodeError) as error:
        return {}, [f"Homebrew dust metadata is malformed: {error}"]
    prefix = _command([brew, "--prefix", "dust"], required=False)
    expected_executable = (Path(prefix) / "bin" / "dust").resolve(strict=False) if prefix else None
    if expected_executable != executable.resolve(strict=False):
        reasons.append("dust executable is not the Homebrew formula's installed command")
    if (
        not installed
        or not installed[0].get("built_as_bottle")
        or not installed[0].get("poured_from_bottle")
    ):
        reasons.append("dust was not installed from a pinned Homebrew bottle")
    version = str(installed[0].get("version", "")) if installed else ""
    if version and version not in _version(executable):
        reasons.append("dust executable version disagrees with Homebrew metadata")
    bottle_tag, bottle = next(iter(bottle_files.items()), (None, {}))
    if not bottle.get("sha256"):
        reasons.append("Homebrew bottle checksum is unavailable")
    required = {
        "source archive checksum": stable.get("checksum"),
        "formula checksum": (formula.get("ruby_source_checksum") or {}).get("sha256"),
        "formula revision": formula.get("tap_git_head"),
        "bottle tag": bottle_tag,
        "bottle URL": bottle.get("url"),
    }
    reasons.extend(
        f"Homebrew {field} is unavailable"
        for field, value in required.items()
        if value in {None, ""}
    )
    return (
        {
            "archive_sha256": stable.get("checksum"),
            "bottle_sha256": bottle.get("sha256"),
            "bottle_tag": bottle_tag,
            "bottle_url": bottle.get("url"),
            "build_recipe": "homebrew/core Formula/d/dust.rb bottle",
            "formula_sha256": (formula.get("ruby_source_checksum") or {}).get("sha256"),
            "formula_revision": formula.get("tap_git_head"),
            "license": formula.get("license"),
            "source_clean": True,
            "source_revision": f"v{version}",
            "source_url": stable.get("url"),
            "target": platform.machine() + "-apple-darwin",
            "toolchain": "Homebrew bottle; compiler version encoded by bottle provenance",
        },
        reasons,
    )


def _host_facts() -> Dict[str, Any]:
    facts = measure.host_facts()
    class_fields = {
        key: facts.get(key)
        for key in (
            "arch",
            "cpu_count",
            "cpu_model",
            "efficiency_cores",
            "memory_bytes",
            "performance_cores",
            "release",
            "system",
        )
    }
    return {
        **facts,
        "class_id": _sha256_json(class_fields),
        "power": _power_facts(),
        "thermal": _thermal_facts(),
    }


def _filesystem_facts(root: Path) -> Dict[str, Any]:
    resolved = root.resolve(strict=True)
    statvfs = os.statvfs(resolved)
    filesystem_type: Optional[str] = None
    solid_state: Optional[bool] = None
    if sys.platform == "darwin":
        diskutil = shutil.which("diskutil")
        device = _darwin_mount_device(resolved)
        if diskutil is not None and device is not None:
            completed = subprocess.run(
                [diskutil, "info", "-plist", device], capture_output=True, check=False
            )
            if completed.returncode == 0:
                try:
                    info = plistlib.loads(completed.stdout)
                    filesystem_type = info.get("FilesystemType")
                    solid_state = info.get("SolidState")
                except plistlib.InvalidFileException:
                    pass
    return {
        "block_size": int(statvfs.f_bsize),
        "flags": int(statvfs.f_flag),
        "fragment_size": int(statvfs.f_frsize),
        "solid_state": solid_state,
        "type": filesystem_type,
    }


def _darwin_mount_device(root: Path) -> Optional[str]:
    """Resolve a path to its mounted device without retaining either private value."""
    df = shutil.which("df")
    if df is None:
        return None
    completed = subprocess.run([df, "-P", str(root)], capture_output=True, check=False)
    if completed.returncode != 0:
        return None
    lines = completed.stdout.decode("utf-8", errors="replace").splitlines()
    if len(lines) != 2:
        return None
    fields = lines[1].split()
    if not fields or not fields[0].startswith("/dev/"):
        return None
    return fields[0]


def _collector_facts() -> Dict[str, Any]:
    return {
        "macos_sample": _command_identity(Path("/usr/bin/sample")),
        "process_rusage": {
            "reason": None if hasattr(os, "wait4") else "os.wait4 is unavailable",
            "supported": hasattr(os, "wait4"),
        },
        "time": _command_identity(Path("/usr/bin/time")),
    }


def _power_facts() -> Dict[str, Any]:
    source = measure._darwin_power_source()
    if sys.platform != "darwin":
        return {"available": False, "reason": "power source is macOS-only", "source": None}
    return {
        "available": source is not None,
        "reason": None if source is not None else "power source was not parseable",
        "source": source,
    }


def _thermal_facts() -> Dict[str, Any]:
    pressure = measure._darwin_thermal_pressure()
    if sys.platform != "darwin":
        return {
            "available": False,
            "pressure": None,
            "reason": "thermal pressure is macOS-only",
        }
    return {
        "available": pressure is not None,
        "pressure": pressure,
        "reason": None if pressure is not None else "current thermal pressure was not parseable",
    }


def apple_silicon_apfs_scope_reasons(
    document: Mapping[str, Any],
) -> list[str]:
    """Return why a provenance document cannot support this campaign's target claim."""
    reasons: list[str] = []
    host = document.get("host")
    filesystem = document.get("filesystem")
    collectors = document.get("collectors")
    if not isinstance(host, Mapping):
        return ["host facts are missing"]
    if host.get("system") != "Darwin" or host.get("arch") != "arm64":
        reasons.append("host is not arm64 Darwin")
    for field in ("performance_cores", "efficiency_cores"):
        if not isinstance(host.get(field), int) or host.get(field) <= 0:
            reasons.append(f"{field} topology is unavailable")
    power = host.get("power")
    if not isinstance(power, Mapping) or power.get("source") != "AC":
        reasons.append("AC power is not established")
    thermal = host.get("thermal")
    if not isinstance(thermal, Mapping) or thermal.get("pressure") != "normal":
        reasons.append("normal thermal pressure is not established")
    if not isinstance(filesystem, Mapping) or str(filesystem.get("type", "")).lower() != "apfs":
        reasons.append("subject filesystem is not APFS")
    if not isinstance(filesystem, Mapping) or filesystem.get("solid_state") is not True:
        reasons.append("solid-state storage is not established")
    process_rusage = collectors.get("process_rusage") if isinstance(collectors, Mapping) else None
    if not isinstance(process_rusage, Mapping) or process_rusage.get("supported") is not True:
        reasons.append("per-process resource collection is unavailable")
    return reasons


def _file_identities(paths: Sequence[Path]) -> list[Dict[str, Any]]:
    identities = []
    seen_names: set[str] = set()
    for path in paths:
        resolved = path.resolve(strict=True)
        name = resolved.name
        if name in seen_names:
            raise ProvenanceError(f"artifact files repeat the redacted name {name!r}")
        seen_names.add(name)
        identities.append(
            {
                "name": name,
                "sha256": _sha256_file(resolved),
                "size_bytes": resolved.stat().st_size,
            }
        )
    return sorted(identities, key=lambda item: item["name"])


def _version(executable: Path) -> str:
    output = _command([str(executable.resolve(strict=True)), "--version"])
    version = " ".join(output.split())[:300]
    if not version:
        raise ProvenanceError(f"{executable.name} produced no version")
    return version


def _command_identity(path: Path) -> Dict[str, Any]:
    if not path.is_file():
        return {"available": False, "reason": "command is unavailable"}
    return {
        "available": True,
        "name": path.name,
        "sha256": _sha256_file(path),
        "size_bytes": path.stat().st_size,
    }


def _git(root: Path, *arguments: str) -> str:
    return _command(["git", *arguments], cwd=root)


def _command(argv: Sequence[str], *, cwd: Optional[Path] = None, required: bool = True) -> str:
    completed = subprocess.run(
        list(argv),
        cwd=cwd,
        stdin=subprocess.DEVNULL,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        if required:
            error = completed.stderr.decode("utf-8", errors="replace").strip()
            raise ProvenanceError(f"command {Path(argv[0]).name} failed: {error[:300]}")
        return ""
    return completed.stdout.decode("utf-8", errors="replace").strip()


def _normalize_remote(remote: str) -> str:
    if remote.startswith("git@github.com:"):
        remote = "https://github.com/" + remote.removeprefix("git@github.com:")
    parsed = urllib.parse.urlsplit(remote)
    if parsed.scheme and parsed.hostname:
        remote = urllib.parse.urlunsplit((parsed.scheme, parsed.hostname, parsed.path, "", ""))
    return remote.removesuffix(".git")


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _sha256_json(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _manifest_id(document: Mapping[str, Any]) -> str:
    return _sha256_json({key: value for key, value in document.items() if key != "manifest_id"})


def _parse_artifact(value: str) -> ArtifactSpec:
    descriptor, separator, raw_path = value.partition("=")
    label, kind_separator, kind = descriptor.partition(":")
    if not separator or not kind_separator or not label or kind not in ARTIFACT_KINDS:
        raise ProvenanceError("artifact must be LABEL:KIND=PATH")
    return ArtifactSpec(label=label, kind=kind, executable=Path(raw_path))


def _parse_labeled(value: str) -> Tuple[str, str]:
    label, separator, raw = value.partition("=")
    if not separator or not label or not raw:
        raise ProvenanceError("labeled value must be LABEL=VALUE")
    return label, raw


def _parse_build_argv(raw: str) -> Tuple[str, ...]:
    value = json.loads(raw)
    if (
        not isinstance(value, list)
        or not value
        or not all(isinstance(argument, str) and argument for argument in value)
    ):
        raise ProvenanceError("build argv must be a non-empty JSON string array")
    return tuple(value)


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(prog="benchmarks.realtree.provenance")
    subparsers = parser.add_subparsers(dest="command", required=True)
    captured = subparsers.add_parser("capture")
    captured.add_argument("--source-root", type=Path, default=PROJECT_ROOT)
    captured.add_argument("--subject-root", type=Path, required=True)
    captured.add_argument("--artifact", action="append", required=True)
    captured.add_argument("--supporting", action="append", default=[])
    captured.add_argument("--build-argv", action="append", default=[])
    captured.add_argument("--output", type=Path, required=True)
    captured.add_argument("--require-claim-grade", action="store_true")

    checked = subparsers.add_parser("verify")
    checked.add_argument("--source-root", type=Path, default=PROJECT_ROOT)
    checked.add_argument("--subject-root", type=Path, required=True)
    checked.add_argument("--artifact", action="append", required=True)
    checked.add_argument("--supporting", action="append", default=[])
    checked.add_argument("--manifest", type=Path, required=True)
    checked.add_argument("--allow-exploratory", action="store_true")
    arguments = parser.parse_args(list(argv))

    try:
        parsed = [_parse_artifact(value) for value in arguments.artifact]
        supporting: Dict[str, list[Path]] = {}
        for value in arguments.supporting:
            label, raw = _parse_labeled(value)
            supporting.setdefault(label, []).append(Path(raw))
        if arguments.command == "capture":
            build_argv: Dict[str, Tuple[str, ...]] = {}
            for label, raw in map(_parse_labeled, arguments.build_argv):
                if label in build_argv:
                    raise ProvenanceError(f"build argv repeats artifact label {label!r}")
                build_argv[label] = _parse_build_argv(raw)
            specs = [
                ArtifactSpec(
                    label=artifact.label,
                    kind=artifact.kind,
                    executable=artifact.executable,
                    supporting_files=tuple(supporting.get(artifact.label, ())),
                    build_argv=build_argv.get(artifact.label, ()),
                )
                for artifact in parsed
            ]
            document = capture(
                source_root=arguments.source_root,
                subject_root=arguments.subject_root,
                artifacts=specs,
            )
            if arguments.require_claim_grade and not document["claim_grade"]:
                raise ProvenanceError(
                    "capture is exploratory: " + "; ".join(document["invalidation_reasons"])
                )
            arguments.output.parent.mkdir(parents=True, exist_ok=True)
            arguments.output.write_text(
                json.dumps(document, indent=2, sort_keys=True), encoding="utf-8"
            )
        else:
            document = load_and_verify(
                arguments.manifest,
                source_root=arguments.source_root,
                subject_root=arguments.subject_root,
                artifacts={artifact.label: artifact.executable for artifact in parsed},
                supporting_files=supporting,
                require_claim_grade=not arguments.allow_exploratory,
            )
    except (OSError, ProvenanceError, TypeError, ValueError, json.JSONDecodeError) as error:
        parser.error(str(error))

    print(
        json.dumps(
            {
                "claim_grade": document["claim_grade"],
                "manifest_id": document["manifest_id"],
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
