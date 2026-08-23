#!/usr/bin/env python3
"""Inspect fdu crate, wheel, and source-distribution release artifacts."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import tarfile
import zipfile
from dataclasses import asdict, dataclass
from email.parser import Parser
from pathlib import Path

RELEASE_WHEEL_PLATFORMS = {
    "manylinux x86-64": re.compile(r"manylinux[^-]*_x86_64\.whl$"),
    "manylinux arm64": re.compile(r"manylinux[^-]*_aarch64\.whl$"),
    "macOS x86-64": re.compile(r"macosx_[^-]*_x86_64\.whl$"),
    "macOS arm64": re.compile(r"macosx_[^-]*_arm64\.whl$"),
    "Windows x86-64": re.compile(r"win_amd64\.whl$"),
}


@dataclass(frozen=True, slots=True)
class Artifact:
    """Validated artifact identity retained as release evidence."""

    filename: str
    kind: str
    bytes: int
    sha256: str


def require(condition: bool, message: str) -> None:
    """Raise a stable validation error when an artifact contract is violated."""
    if not condition:
        raise ValueError(message)


def digest(path: Path) -> str:
    """Return one artifact's SHA-256 digest without loading it all into memory."""
    checksum = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            checksum.update(chunk)
    return checksum.hexdigest()


def inspect_wheel(path: Path, version: str) -> None:
    """Validate wheel metadata, typed facade, native placement, license, and SBOM."""
    with zipfile.ZipFile(path) as archive:
        names = set(archive.namelist())
        metadata_names = [name for name in names if name.endswith(".dist-info/METADATA")]
        require(len(metadata_names) == 1, f"{path.name}: expected exactly one METADATA")
        metadata = Parser().parsestr(archive.read(metadata_names[0]).decode())
        require(metadata["Name"] == "fdu", f"{path.name}: wheel project must be fdu")
        require(metadata["Version"] == version, f"{path.name}: wheel version mismatch")
        require(metadata["License-Expression"] == "MIT", f"{path.name}: missing MIT expression")
        require("LICENSE" in metadata.get_all("License-File", []), f"{path.name}: license metadata")
        classifiers = metadata.get_all("Classifier", [])
        require("Typing :: Typed" in classifiers, f"{path.name}: missing typed classifier")
        for member in ("fdu/__init__.py", "fdu/_native.pyi", "fdu/py.typed"):
            require(member in names, f"{path.name}: missing {member}")
        native = [
            name
            for name in names
            if name.startswith("fdu/_native.") and not name.endswith((".py", ".pyi"))
        ]
        require(len(native) == 1, f"{path.name}: expected one private native extension")
        require(not any(name.startswith("fdu_py") for name in names), f"{path.name}: leaked fdu_py")
        require(
            not any("/__pycache__/" in name or name.endswith(".pyc") for name in names),
            f"{path.name}: bytecode cache leaked into wheel",
        )
        require(
            any(name.endswith(".dist-info/licenses/LICENSE") for name in names),
            f"{path.name}: license text missing",
        )
        require(
            any(".dist-info/sboms/" in name and name.endswith(".cyclonedx.json") for name in names),
            f"{path.name}: CycloneDX SBOM missing",
        )


def inspect_sdist(path: Path, version: str) -> None:
    """Validate that a source build has every workspace and Python input it needs."""
    prefix = f"fdu-{version}/"
    with tarfile.open(path, "r:gz") as archive:
        names = set(archive.getnames())
    required = {
        f"{prefix}LICENSE",
        f"{prefix}README.md",
        f"{prefix}pyproject.toml",
        f"{prefix}python/fdu/__init__.py",
        f"{prefix}python/fdu/_native.pyi",
        f"{prefix}python/fdu/py.typed",
        f"{prefix}crates/fdu-py/examples/rollup_adapter.py",
    }
    missing = sorted(required - names)
    require(not missing, f"{path.name}: missing source files: {', '.join(missing)}")
    require(
        any(name.endswith("/crates/fdu-core/src/lib.rs") for name in names),
        f"{path.name}: core Rust workspace source missing",
    )


def inspect_crate(path: Path, package: str, version: str) -> None:
    """Validate one Cargo package's identity and legal/discovery files.

    `build.rs` is required because omitting it from a crate's `include` list is a
    failure that already happened: everything built locally, because the file was on
    disk, and the published source would not compile.
    """
    prefix = f"{package}-{version}/"
    with tarfile.open(path, "r:gz") as archive:
        names = set(archive.getnames())
    for member in (
        "Cargo.toml",
        "Cargo.toml.orig",
        "LICENSE",
        "README.md",
        "build.rs",
        "src/lib.rs",
    ):
        require(f"{prefix}{member}" in names, f"{path.name}: missing {member}")


def inspect_directory(
    directory: Path,
    version: str,
    *,
    require_release_matrix: bool = False,
) -> list[Artifact]:
    """Validate both crates, one sdist, and every wheel in a release directory.

    Both crates, because `fdu` depends on `fdu-core` and cannot be installed without
    it: a release recording only the crate a user names records half an artifact set.
    """
    paths = sorted(path for path in directory.iterdir() if path.is_file())
    wheels = [path for path in paths if path.suffix == ".whl"]
    sdists = [path for path in paths if path.name == f"fdu-{version}.tar.gz"]
    crates = [path for path in paths if path.name == f"fdu-{version}.crate"]
    core_crates = [path for path in paths if path.name == f"fdu-core-{version}.crate"]
    require(bool(wheels), "no wheels found")
    require(len(sdists) == 1, "expected exactly one fdu source distribution")
    require(len(crates) == 1, "expected exactly one fdu crate")
    require(len(core_crates) == 1, "expected exactly one fdu-core crate")
    if require_release_matrix:
        require(len(wheels) == len(RELEASE_WHEEL_PLATFORMS), "expected exactly five wheels")
        for label, pattern in RELEASE_WHEEL_PLATFORMS.items():
            matches = [wheel.name for wheel in wheels if pattern.search(wheel.name)]
            require(len(matches) == 1, f"expected one {label} wheel, found {len(matches)}")
    for wheel in wheels:
        require(
            re.match(rf"fdu-{re.escape(version)}-cp312-abi3-", wheel.name) is not None,
            f"{wheel.name}: expected cp312-abi3 wheel",
        )
        inspect_wheel(wheel, version)
    inspect_sdist(sdists[0], version)
    inspect_crate(core_crates[0], "fdu-core", version)
    inspect_crate(crates[0], "fdu", version)
    artifacts = []
    for path in [*wheels, *sdists, *core_crates, *crates]:
        kind = "wheel" if path.suffix == ".whl" else "sdist" if path in sdists else "crate"
        artifacts.append(Artifact(path.name, kind, path.stat().st_size, digest(path)))
    return artifacts


def parser() -> argparse.ArgumentParser:
    """Build the command-line parser."""
    result = argparse.ArgumentParser()
    result.add_argument("directory", type=Path)
    result.add_argument("--version", required=True)
    result.add_argument("--manifest", type=Path, required=True)
    result.add_argument("--checksums", type=Path, required=True)
    result.add_argument("--require-release-matrix", action="store_true")
    return result


def main() -> None:
    """Inspect artifacts and emit a machine manifest plus checksum file."""
    args = parser().parse_args()
    artifacts = inspect_directory(
        args.directory,
        args.version,
        require_release_matrix=args.require_release_matrix,
    )
    args.manifest.write_text(
        json.dumps(
            {"version": args.version, "artifacts": [asdict(item) for item in artifacts]}, indent=2
        )
        + "\n",
        encoding="utf-8",
    )
    args.checksums.write_text(
        "".join(f"{artifact.sha256}  {artifact.filename}\n" for artifact in artifacts),
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
