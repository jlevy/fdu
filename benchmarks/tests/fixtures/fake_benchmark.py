from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Dict, Optional, Sequence


def main(arguments: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument(
        "--mode",
        required=True,
        choices=(
            "child-sleep",
            "environment",
            "invalid-json",
            "mutate-corpus",
            "nonzero",
            "prepare-cache",
            "prepare-snapshot",
            "raw-output",
            "sleep",
            "stderr",
            "success",
            "wrong-digest",
        ),
    )
    parser.add_argument("--corpus", type=Path)
    parser.add_argument("--cache-marker", type=Path)
    parser.add_argument("--require-file", action="append", type=Path, default=[])
    parser.add_argument("--snapshot", type=Path)
    parser.add_argument("--output-bytes", type=int, default=0)
    namespace = parser.parse_args(arguments)
    manifest: Dict[str, Any] = json.loads(
        namespace.manifest.read_text(encoding="utf-8")
    )

    if namespace.mode == "prepare-cache":
        if namespace.cache_marker is None:
            parser.error("--cache-marker is required for prepare-cache")
        namespace.cache_marker.write_text("ready\n", encoding="utf-8")
        return 0
    if namespace.mode == "prepare-snapshot":
        if namespace.snapshot is None:
            parser.error("--snapshot is required for prepare-snapshot")
        namespace.snapshot.write_text(
            json.dumps({"semantic_digest": manifest["semantic_digest"]}),
            encoding="utf-8",
        )
        return 0

    if namespace.mode == "child-sleep":
        child = subprocess.Popen(
            [sys.executable, "-c", "import time; time.sleep(60)"],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        sys.stdout.write(f"{child.pid}\n")
        sys.stdout.flush()
        time.sleep(60)
        return 0
    if namespace.mode == "sleep":
        time.sleep(60)
        return 0
    if namespace.mode == "nonzero":
        return 7
    if namespace.mode == "invalid-json":
        sys.stdout.write("not json\n")
        return 0
    if namespace.mode == "raw-output":
        sys.stdout.write("x" * namespace.output_bytes)
        return 0
    if namespace.mode == "stderr":
        sys.stderr.write("unexpected diagnostic\n")
    if namespace.mode == "mutate-corpus":
        if namespace.corpus is None:
            parser.error("--corpus is required for mutate-corpus")
        (namespace.corpus / "unexpected.txt").write_text("changed", encoding="utf-8")

    semantic_digest = manifest["semantic_digest"]
    if namespace.mode == "wrong-digest":
        semantic_digest = "0" * 64
    output = {
        "entry_count": manifest["counts"]["total"],
        "ok": True,
        "semantic_digest": semantic_digest,
        "sentinel": os.environ.get("FDU_SENTINEL"),
        "required_files": sum(path.is_file() for path in namespace.require_file),
    }
    if namespace.snapshot is not None:
        snapshot = json.loads(namespace.snapshot.read_text(encoding="utf-8"))
        output["snapshot_matches_manifest"] = (
            snapshot["semantic_digest"] == manifest["semantic_digest"]
        )
    json.dump(output, sys.stdout, sort_keys=True)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
