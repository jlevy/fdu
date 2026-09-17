"""The known-violation registry: every difference the engine is known to have, and why.

The registry is reviewed like a golden. A run fails on:

- an unregistered difference;
- a registered case whose generalized diff paths changed;
- a registered case that now matches, so its entry is stale;
- a class with no entries, or an entry naming a class the table does not define;
- an entry still marked `unclassified`;
- a run with zero cases or zero parseable cold answers.

A class is emptied only by a run that executed every case, because a subset run checks
the entries it executed and nothing else.

Format:

    [classes.content-containment]
    description = "A narrower analysis request served from records of a wider one"
    clears_with = "Phase 1 item 2: content identity and equality serve"
    bead = "fdu-gija"

    [[violation]]
    class = "content-containment"
    paths = ["analysis.analyze[]"]
    keys = ["warm/cli-report/auto/W_all/-/a_lines"]

Entries sharing a class, path set, and platforms are grouped under one `keys` list.
A class is assigned per case, not per path: a case with two causes carries the class of
the one that clears last, and shows as a changed shape when the first cause is fixed.

A violation may carry `platforms = ["win32"]` (values of `sys.platform`) when it occurs,
or takes its shape, only there; that is itself a finding to explain. One key may appear
in several groups only when every such group names its platforms and no platform is
named twice.
"""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from collections import defaultdict
from collections.abc import Iterable
from dataclasses import dataclass, field
from pathlib import Path

DEFAULT_PATH = Path(__file__).resolve().parent / "known-violations.toml"
UNCLASSIFIED = "unclassified"
KEY_SEGMENTS = 6
# The platforms the registry describes: the values of `sys.platform` CI runs on. An entry
# naming no platforms applies to all of them.
PLATFORMS = ("darwin", "linux", "win32")

CLASS_FIELDS = frozenset({"description", "clears_with", "bead"})
VIOLATION_FIELDS = frozenset({"class", "paths", "keys", "platforms"})

# One judged case: its key, whether its verdict is an allowed outcome, and the
# generalized paths that differ when it is not.
Judged = tuple[str, bool, tuple[str, ...]]


@dataclass(frozen=True)
class ViolationClass:
    """Why a group of violations exists and what removes it."""

    name: str
    description: str
    clears_with: str
    bead: str


@dataclass(frozen=True)
class Entry:
    """One registered violation, on every platform or on the named ones."""

    key: str
    klass: str
    paths: tuple[str, ...]
    platforms: tuple[str, ...] | None = None

    def applies_on(self, platform: str) -> bool:
        return self.platforms is None or platform in self.platforms


@dataclass
class Registry:
    """The classes table and every registered violation, by case key."""

    classes: dict[str, ViolationClass] = field(default_factory=dict)
    entries: dict[str, list[Entry]] = field(default_factory=dict)

    def entry_for(self, key: str, platform: str) -> Entry | None:
        """The entry that applies to `key` on `platform`, if any."""
        return next((e for e in self.entries.get(key, []) if e.applies_on(platform)), None)

    def add(self, entry: Entry) -> None:
        """Register `entry`, refusing a platform overlap with another entry for its key."""
        existing = self.entries.setdefault(entry.key, [])
        if existing and (
            entry.platforms is None
            or any(other.platforms is None for other in existing)
            or any(set(entry.platforms) & set(other.platforms or ()) for other in existing)
        ):
            raise ValueError(f"key registered twice for the same platform: {entry.key}")
        existing.append(entry)

    def all_entries(self) -> list[Entry]:
        return [entry for entries in self.entries.values() for entry in entries]


@dataclass(frozen=True)
class Failure:
    """One reason a run does not conform to the registry."""

    reason: str
    key: str | None = None
    detail: str = ""

    def __str__(self) -> str:
        parts = [self.reason]
        if self.key:
            parts.append(self.key)
        if self.detail:
            parts.append(self.detail if len(self.detail) <= 160 else self.detail[:157] + "...")
        return ": ".join(parts)


def load(path: Path) -> Registry:
    """Read a registry, or an empty one when the file does not exist."""
    if not path.exists():
        return Registry()
    return parse(path.read_text(encoding="utf-8"))


def parse(text: str) -> Registry:
    data = tomllib.loads(text)
    _refuse_unknown("registry", set(data), frozenset({"classes", "violation"}))
    registry = Registry()
    for name, table in data.get("classes", {}).items():
        _refuse_unknown(f"class {name}", set(table), CLASS_FIELDS, required=CLASS_FIELDS)
        registry.classes[name] = ViolationClass(
            name, table["description"], table["clears_with"], table["bead"]
        )
    for group in data.get("violation", []):
        _refuse_unknown(
            "violation", set(group), VIOLATION_FIELDS, required={"class", "paths", "keys"}
        )
        if not group["paths"]:
            raise ValueError(f"violation with no paths: {group['keys'][:1]}")
        platforms = tuple(group["platforms"]) if "platforms" in group else None
        for key in group["keys"]:
            if len(key.split("/")) != KEY_SEGMENTS:
                raise ValueError(f"key must have {KEY_SEGMENTS} segments: {key}")
            registry.add(Entry(key, group["class"], tuple(group["paths"]), platforms))
    return registry


def _refuse_unknown(
    where: str, fields: set[str], allowed: frozenset[str], required: Iterable[str] = ()
) -> None:
    unknown = fields - allowed
    missing = set(required) - fields
    if unknown or missing:
        raise ValueError(f"{where}: unknown fields {sorted(unknown)}, missing {sorted(missing)}")


def verify(
    registry: Registry,
    judged: Iterable[Judged],
    *,
    full: bool,
    platform: str,
    cold_answers: int,
) -> list[Failure]:
    """Every way `judged` and the registry disagree."""
    failures: list[Failure] = []
    cases = list(judged)
    if not cases:
        failures.append(Failure("run executed no cases"))
    if cold_answers == 0:
        failures.append(Failure("run parsed no cold answers"))

    for entry in registry.all_entries():
        if entry.klass == UNCLASSIFIED:
            failures.append(Failure("entry is unclassified", entry.key))
        elif entry.klass not in registry.classes:
            failures.append(Failure("entry names an undefined class", entry.key, entry.klass))

    for key, allowed, paths in cases:
        entry = registry.entry_for(key, platform)
        if allowed:
            if entry is not None:
                failures.append(Failure("registered case now conforms; remove its entry", key))
        elif entry is None:
            failures.append(Failure("unregistered difference", key, ", ".join(paths)))
        elif tuple(sorted(paths)) != tuple(sorted(entry.paths)):
            detail = f"registered {list(entry.paths)}, observed {list(paths)}"
            failures.append(Failure("difference changed shape", key, detail))

    if full:
        used = {entry.klass for entry in registry.all_entries()}
        for name in sorted(set(registry.classes) - used):
            failures.append(Failure("class has no entries; remove it", None, name))
        executed = {key for key, _, _ in cases}
        for entry in registry.all_entries():
            if entry.applies_on(platform) and entry.key not in executed:
                failures.append(Failure("registered case is not in the full matrix", entry.key))
    return failures


def record(registry: Registry, judged: Iterable[Judged], *, platform: str) -> Registry:
    """The registry rewritten to match what `judged` observed on `platform`."""
    return merge(registry, {platform: judged})


def merge(registry: Registry, judged_by_platform: dict[str, Iterable[Judged]]) -> Registry:
    """The registry rewritten to match what each platform's run observed.

    For every case a platform executed, that platform's entry becomes what it observed:
    the difference's shape, or nothing when the case conforms. A platform that did not
    execute a case keeps its entry, so a local run never erases what CI recorded
    elsewhere. A case keeps its class; a case new to the registry is `unclassified`, so
    the run fails until someone reads the difference and names its cause. Platforms
    sharing a class and shape share one entry, which names no platforms when it covers
    all of them.
    """
    unknown = set(judged_by_platform) - set(PLATFORMS)
    if unknown:
        raise ValueError(f"record on {', '.join(PLATFORMS)}, not {sorted(unknown)}")
    assigned: dict[str, dict[str, tuple[str, tuple[str, ...]]]] = defaultdict(dict)
    for entry in registry.all_entries():
        for target in entry.platforms or PLATFORMS:
            assigned[entry.key][target] = (entry.klass, entry.paths)
    for platform, judged in judged_by_platform.items():
        for key, allowed, paths in judged:
            by_platform = assigned[key]
            if allowed:
                by_platform.pop(platform, None)
                continue
            klass = next((k for k, _ in by_platform.values()), UNCLASSIFIED)
            by_platform[platform] = (klass, tuple(sorted(paths)))

    updated = Registry(classes=dict(registry.classes))
    for key, by_platform in assigned.items():
        groups: dict[tuple[str, tuple[str, ...]], list[str]] = defaultdict(list)
        for target, shape in by_platform.items():
            groups[shape].append(target)
        for (klass, paths), targets in groups.items():
            platforms = None if set(targets) == set(PLATFORMS) else tuple(sorted(targets))
            updated.add(Entry(key, klass, paths, platforms))
    return updated


def dump(registry: Registry) -> str:
    """TOML text for `registry`, grouped and sorted so a re-record diffs cleanly."""
    lines = [
        "# Known path-independence violations. Reviewed like a golden: see registry.py.",
        "# Regenerate with `make path-independence-record`, then classify new entries.",
    ]
    for name in sorted(registry.classes):
        klass = registry.classes[name]
        lines += [
            "",
            f"[classes.{_bare_or_quoted(name)}]",
            f"description = {_string(klass.description)}",
            f"clears_with = {_string(klass.clears_with)}",
            f"bead = {_string(klass.bead)}",
        ]
    groups: dict[tuple[str, tuple[str, ...], tuple[str, ...] | None], list[str]] = defaultdict(list)
    for entry in registry.all_entries():
        groups[(entry.klass, entry.paths, entry.platforms)].append(entry.key)
    for (klass, paths, platforms), keys in sorted(groups.items(), key=_group_order):
        lines += ["", "[[violation]]", f"class = {_string(klass)}"]
        if platforms is not None:
            lines.append(f"platforms = {_array(platforms)}")
        lines.append(f"paths = {_array(paths)}")
        lines.append(f"keys = {_array(sorted(keys))}")
    return "\n".join(lines) + "\n"


def _group_order(
    item: tuple[tuple[str, tuple[str, ...], tuple[str, ...] | None], list[str]],
) -> tuple[str, tuple[str, ...], str]:
    (klass, _, platforms), keys = item
    return (klass, platforms or (), min(keys))


def _bare_or_quoted(name: str) -> str:
    if name and all(char.isascii() and (char.isalnum() or char in "-_") for char in name):
        return name
    return _string(name)


def _string(value: str) -> str:
    escaped: list[str] = []
    for char in value:
        if char in '"\\':
            escaped.append("\\" + char)
        elif ord(char) < 0x20 or ord(char) == 0x7F:
            escaped.append(f"\\u{ord(char):04X}")
        else:
            escaped.append(char)
    return '"' + "".join(escaped) + '"'


def _array(values: Iterable[str]) -> str:
    items = [_string(value) for value in values]
    if not items:
        return "[]"
    return "[\n" + "".join(f"  {item},\n" for item in items) + "]"


def write_judged(path: Path, platform: str, judged: Iterable[Judged]) -> None:
    """Write one run's judged cases, so runs on several platforms can be merged."""
    cases = [[key, allowed, list(paths)] for key, allowed, paths in judged]
    path.write_text(json.dumps({"platform": platform, "cases": cases}), encoding="utf-8")


def read_judged(path: Path) -> tuple[str, list[Judged]]:
    data = json.loads(path.read_text(encoding="utf-8"))
    return data["platform"], [(key, allowed, tuple(paths)) for key, allowed, paths in data["cases"]]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Merge judged runs from several platforms into the committed registry."
    )
    parser.add_argument("judged", nargs="+", type=Path, help="judged JSON from runner.py --judged")
    parser.add_argument("--registry", type=Path, default=DEFAULT_PATH)
    args = parser.parse_args(argv)
    runs: dict[str, list[Judged]] = {}
    for path in args.judged:
        platform, cases = read_judged(path)
        if platform in runs:
            raise SystemExit(f"two runs from {platform}: {path}")
        runs[platform] = cases
    merged = merge(load(args.registry), runs)
    args.registry.write_text(dump(merged), encoding="utf-8")
    unclassified = sum(1 for entry in merged.all_entries() if entry.klass == UNCLASSIFIED)
    print(f"merged {sorted(runs)}: {len(merged.all_entries())} entries, {unclassified} unclassified")
    return 0


if __name__ == "__main__":
    sys.exit(main())
