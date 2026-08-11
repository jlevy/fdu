"""Archive a path-redacted, content-addressed real-tree measurement run.

Raw paired samples are the evidence behind an experiment. They belong beside the
experiment record, not in the ignored machine-local results directory. Archiving
changes only the operator-chosen tree label and the root-path hash; timings, sample
order, oracle verdicts, binary identities, and statistics remain byte-for-byte values
from the run document.
"""

from __future__ import annotations

import hashlib
import json
import re
from copy import deepcopy
from pathlib import Path
from typing import Any, Dict, Iterator


class EvidenceError(RuntimeError):
    """A run contains data that is unsafe or incomplete to archive."""


def archive_run(
    document: Dict[str, Any], *, destination: Path, tree_label: str
) -> tuple[Dict[str, Any], str]:
    """Write a sanitized immutable run and return it with its content digest."""
    if not tree_label or any(character in tree_label for character in ("/", "\\")):
        raise EvidenceError("the archived tree label must be a non-path identifier")
    archived = deepcopy(document)
    tree = archived.get("tree")
    if not isinstance(tree, dict) or not tree.get("engine_digest"):
        raise EvidenceError("the run has no engine-digested reference tree")
    tree["label"] = tree_label
    tree["root_id"] = hashlib.sha256(
        b"fdu-archived-tree-v1\0" + str(tree["engine_digest"]).encode("ascii")
    ).hexdigest()

    for value in _strings(archived):
        if _contains_absolute_path(value):
            raise EvidenceError(f"archived evidence contains an absolute path: {value!r}")

    destination.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(archived, indent=2, sort_keys=True) + "\n"
    if destination.exists():
        existing = destination.read_text(encoding="utf-8")
        if existing != text:
            raise EvidenceError(
                f"immutable evidence destination already contains different data: {destination}"
            )
        return archived, hashlib.sha256(text.encode("utf-8")).hexdigest()
    destination.write_text(text, encoding="utf-8")
    return archived, hashlib.sha256(text.encode("utf-8")).hexdigest()


_ABSOLUTE_PATH = re.compile(
    r"(?:^|[\s='\"])(?:~[/\\]|/(?!/)|[A-Za-z]:[/\\])"
)


def _contains_absolute_path(value: str) -> bool:
    return bool(_ABSOLUTE_PATH.search(value))


def _strings(value: Any) -> Iterator[str]:
    if isinstance(value, dict):
        for item in value.values():
            yield from _strings(item)
    elif isinstance(value, list):
        for item in value:
            yield from _strings(item)
    elif isinstance(value, str):
        yield value


__all__ = ["EvidenceError", "archive_run"]
