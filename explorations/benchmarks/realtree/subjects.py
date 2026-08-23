"""The nominated real-tree subject set, and the policy a set has to satisfy.

    python -m benchmarks.realtree subjects --nominations <file> --out <document>

Campaign 2 requires at least one nominated real tree in the paired set behind any
accept decision, and treats generated corpora as screening only. That rule exists
because a uniform corpus does not merely add noise -- it moves results, in one
direction, by more than the accept gate. The floor report measured a generated tree
hiding about 15 points of fdu's distance from the syscall floor, and the same effect
inverted a peer ranking: fdu led ripgrep's walker by 12-26% on four generated trees and
trailed by 11.8% on ``/usr``.

Nothing in the harness could express that rule before this module, because a run names
one ``--root`` and the record kept no notion of which trees are fit to decide anything.

**Why a set and not a tree.** One real tree retires a generated-corpus ranking and
cannot establish a real-tree one; the floor report says so about its own evidence. Real
trees differ from each other along the axes that decided every transfer failure on the
record -- name length and shape, directory width, depth, file density -- so a set spread
across characters is what makes "this transfers" a statement about more than one
accident.

**Why the paths are not here.** A subject is somebody's working tree, and the loop
treats it as confidential: an artifact records ``root_id``, the SHA-256 of the path,
never the path. The same split applies to nomination. The *nominations* file is local
and gitignored -- it says where the trees are on this machine. The *document* this
module emits is redacted and committable: labels, characters, shapes and ``root_id``s,
which is everything a reader needs to know what a claim rests on and nothing that says
where it lives.

**Why nomination is per host.** ``root_id`` is a hash of an absolute path, so a set is
a fact about one machine. The campaign wants real trees on more than one host; each
host nominates its own and commits its own document.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Mapping, Sequence

from benchmarks.realtree import measure
from benchmarks.realtree import tree as reference_tree

#: Schema of the redacted document this module emits.
DOCUMENT_SCHEMA = "fdu-nominated-subjects-v1"

#: The characters a set is judged against, and what each one is evidence about.
#:
#: These are not a taxonomy of filesystems; they are the shapes that have actually moved
#: a result on this record. A source checkout carries long real names and wide
#: ``node_modules`` fans; a package cache is deep, narrow and enormously repetitive; a
#: system prefix is the shape `/usr` has, which is where the peer ranking inverted; a
#: media tree is few large files, which is the only character under which a per-entry
#: cost stops dominating at all.
CHARACTERS: Dict[str, str] = {
    "source-checkout": "long real names, wide dependency fans, live mtimes",
    "package-cache": "deep, narrow, highly repetitive, mostly small files",
    "system-prefix": "the shape /usr has, where the peer ranking inverted",
    "media-tree": "few large files, where per-entry cost stops dominating",
}

#: How many distinct characters a set needs before it can decide anything.
#:
#: Three rather than four because the fourth, ``media-tree``, is the one many hosts
#: genuinely do not have, and refusing to let such a host decide anything would make the
#: rule unusable rather than strict. Three is also the number of recorded generalisation
#: failures this rule exists to prevent, which is a coincidence but a useful mnemonic.
MINIMUM_CHARACTERS = 3

#: Apparent-to-allocated ratio past which a tree is reported as sparse, and so is not a
#: real subject however it was obtained. Shared with the ledger's rendering rule.
SPARSE_RATIO_LIMIT = 2.0


class SubjectError(RuntimeError):
    """A nomination could not be read, or a nominated tree could not be observed."""


def load_nominations(path: Path) -> List[Dict[str, Any]]:
    """Read the local, gitignored nominations file.

    Shape, deliberately small::

        [
          {"label": "...", "character": "source-checkout", "path": "/...",
           "provenance": "...", "reconstructible": false}
        ]

    ``provenance`` is required for the same reason `perf-record` requires it: a subject
    whose origin nobody wrote down is one nobody else can obtain, and that is a fact
    about the evidence rather than a gap in the paperwork.
    """
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as error:
        raise SubjectError(
            f"no nominations file at {path}. Create one (it is gitignored) listing the "
            "real trees this host offers; see the module docstring for the shape."
        ) from error
    except json.JSONDecodeError as error:
        raise SubjectError(f"{path} is not valid JSON: {error}") from error
    if not isinstance(raw, list) or not raw:
        raise SubjectError(f"{path} must hold a non-empty list of nominations")

    nominations: List[Dict[str, Any]] = []
    seen: set = set()
    for index, entry in enumerate(raw):
        if not isinstance(entry, Mapping):
            raise SubjectError(f"nomination {index} is not an object")
        missing = [key for key in ("label", "character", "path", "provenance") if not entry.get(key)]
        if missing:
            raise SubjectError(f"nomination {index} is missing {', '.join(missing)}")
        character = str(entry["character"])
        if character not in CHARACTERS:
            raise SubjectError(
                f"nomination {entry['label']!r} has unknown character {character!r}; "
                f"known: {', '.join(sorted(CHARACTERS))}"
            )
        label = str(entry["label"])
        if label in seen:
            raise SubjectError(f"two nominations claim the label {label!r}")
        seen.add(label)
        nominations.append(
            {
                "label": label,
                "character": character,
                "path": Path(str(entry["path"])).expanduser(),
                "provenance": str(entry["provenance"]),
                "reconstructible": bool(entry.get("reconstructible", False)),
            }
        )
    return nominations


def observe(nominations: Sequence[Mapping[str, Any]]) -> Dict[str, Any]:
    """Fingerprint every nominated tree and return the redacted document.

    One full walk per subject, through the same fingerprint the loop uses for its own
    oracle, so a nomination is pinned by content and the paths never leave this process.
    """
    subjects: List[Dict[str, Any]] = []
    for nomination in nominations:
        path = Path(nomination["path"])
        try:
            observed = reference_tree.fingerprint(path, label=str(nomination["label"]))
        except (OSError, reference_tree.ReferenceTreeError) as error:
            raise SubjectError(f"cannot observe {nomination['label']!r}: {error}") from error
        subjects.append(_redact(nomination, observed))
    subjects.sort(key=lambda subject: subject["label"])
    return {
        "schema": DOCUMENT_SCHEMA,
        "characters_required": MINIMUM_CHARACTERS,
        "host": host_class(),
        "subjects": subjects,
    }


def host_class() -> Dict[str, Any]:
    """Which machine nominated this set, to the precision a reader needs.

    A set is a fact about one host -- ``root_id`` hashes an absolute path -- and the
    campaign wants real trees on more than one. Enough here to tell two hosts apart and
    to say what a cold sample on this one could mean; no hostname, no user, no paths.
    """
    facts = measure.host_facts()
    return {
        "system": facts.get("system") or "",
        "release": facts.get("release") or "",
        "arch": facts.get("arch") or "",
        "virtualization": facts.get("virtualization") or "",
    }


def _redact(nomination: Mapping[str, Any], fingerprint: Mapping[str, Any]) -> Dict[str, Any]:
    """Everything a reader needs about a subject, and nothing that locates it."""
    sizes = fingerprint["sizes"]
    counts = fingerprint["counts"]
    apparent = int(sizes["apparent_bytes"])
    allocated = int(sizes["allocated_bytes"])
    return {
        "label": str(nomination["label"]),
        "character": str(nomination["character"]),
        "provenance": str(nomination["provenance"]),
        "reconstructible": bool(nomination["reconstructible"]),
        "root_id": fingerprint["root_id"],
        "engine_digest": fingerprint["engine_digest"],
        "entries": int(counts["total"]),
        "directories": int(counts["directories"]),
        "files": int(counts["files"]),
        "symlinks": int(counts["symlinks"]),
        "apparent_bytes": apparent,
        "allocated_bytes": allocated,
        "max_depth": int(fingerprint["max_depth"]),
        "sparse_ratio": round(apparent / allocated, 2) if allocated else None,
    }


def policy_gaps(document: Mapping[str, Any]) -> List[str]:
    """Why this set cannot yet decide an accept, or an empty list when it can.

    Reported rather than raised: a host part-way through nominating should be told what
    is missing and still be able to use what it has for screening.
    """
    subjects = list(document.get("subjects") or [])
    gaps: List[str] = []
    if not subjects:
        return ["the set is empty"]

    characters = {str(subject.get("character")) for subject in subjects}
    if len(characters) < MINIMUM_CHARACTERS:
        absent = sorted(set(CHARACTERS) - characters)
        gaps.append(
            f"{len(characters)} of {MINIMUM_CHARACTERS} required characters present; "
            f"still to nominate one of: {', '.join(absent)}"
        )

    for subject in subjects:
        label = subject.get("label")
        ratio = subject.get("sparse_ratio")
        # A sparse tree is a generated tree wearing a real tree's clothes, whatever its
        # provenance says: reading a hole costs nothing, which is exactly the property
        # that flattered exp-064's cold figure by eleven points.
        if ratio is not None and ratio >= SPARSE_RATIO_LIMIT:
            gaps.append(
                f"{label} is {ratio}x sparse, so it screens but cannot decide: "
                "reading a hole costs nothing and per-file work looks larger than it is"
            )
        if not str(subject.get("provenance") or "").strip():
            gaps.append(f"{label} records no provenance")
    return gaps


def drift(document: Mapping[str, Any], observed: Mapping[str, Any]) -> List[str]:
    """What changed between a committed set document and the trees as they are now.

    A nominated tree is somebody's live working directory, so it *will* drift, and that
    is not automatically a failure -- it is a fact a reader has to be told before they
    compare a number taken last month with one taken today. Reported per subject.
    """
    was = {str(subject["label"]): subject for subject in document.get("subjects") or []}
    now = {str(subject["label"]): subject for subject in observed.get("subjects") or []}

    reasons: List[str] = []
    for label in sorted(set(was) - set(now)):
        reasons.append(f"{label} is nominated but was not observed")
    for label in sorted(set(now) - set(was)):
        reasons.append(f"{label} was observed but is not in the committed set")
    for label in sorted(set(was) & set(now)):
        before, after = was[label], now[label]
        if before.get("root_id") != after.get("root_id"):
            reasons.append(f"{label} now lives at a different path")
            continue
        if before.get("engine_digest") != after.get("engine_digest"):
            moved = [
                f"{field} {before.get(field)!r} -> {after.get(field)!r}"
                for field in ("entries", "directories", "files", "apparent_bytes", "max_depth")
                if before.get(field) != after.get(field)
            ]
            detail = "; ".join(moved) if moved else "same shape, different content"
            reasons.append(f"{label} changed: {detail}")
    return reasons


def render(document: Mapping[str, Any]) -> str:
    """A short human summary; the JSON document stays the machine-readable form."""
    lines = [f"nominated subjects ({len(document.get('subjects') or [])}):"]
    for subject in document.get("subjects") or []:
        ratio = subject.get("sparse_ratio")
        density = f", {ratio}x sparse" if ratio is not None and ratio >= SPARSE_RATIO_LIMIT else ""
        lines.append(
            f"  {subject['label']:24s} {subject['character']:16s} "
            f"{subject['entries']:>9,} entries, depth {subject['max_depth']}{density}"
        )
    gaps = policy_gaps(document)
    if gaps:
        lines.append("")
        lines.append("this set screens but cannot decide an accept:")
        lines.extend(f"  - {gap}" for gap in gaps)
    else:
        lines.append("")
        lines.append("this set satisfies the accept rule's real-subject requirement.")
    return "\n".join(lines)


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(
        prog="benchmarks.realtree subjects", description=__doc__
    )
    parser.add_argument(
        "--nominations",
        type=Path,
        default=Path("explorations/benchmarks/subjects.local.json"),
        help="local, gitignored file naming this host's real trees",
    )
    parser.add_argument("--out", type=Path, help="write the redacted document here")
    parser.add_argument(
        "--check",
        type=Path,
        help="compare a committed document against the trees as they are now",
    )
    arguments = parser.parse_args(list(argv))

    try:
        nominations = load_nominations(arguments.nominations)
        observed = observe(nominations)
    except SubjectError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    if arguments.check is not None:
        try:
            committed = json.loads(arguments.check.read_text(encoding="utf-8"))
        except OSError as error:
            print(f"error: cannot read {arguments.check}: {error}", file=sys.stderr)
            return 1
        reasons = drift(committed, observed)
        if reasons:
            print("the nominated subjects have drifted:", file=sys.stderr)
            for reason in reasons:
                print(f"  - {reason}", file=sys.stderr)
            return 1
        print("nominated subjects match the committed set", file=sys.stderr)
        return 0

    print(render(observed))
    if arguments.out is not None:
        arguments.out.parent.mkdir(parents=True, exist_ok=True)
        arguments.out.write_text(
            json.dumps(observed, indent=1, sort_keys=True) + "\n", encoding="utf-8"
        )
        print(f"wrote {arguments.out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
