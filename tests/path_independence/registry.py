"""The known-violation registry: every difference the engine is known to have, and why.

The registry is reviewed like a golden. A run fails on:

- an unregistered difference;
- a registered case whose generalized diff paths changed;
- a registered case that now matches, so its entry is stale;
- a class with no entries, or an entry naming a class the table does not define;
- an entry still marked `unclassified`;
- a run with zero cases or zero parseable cold answers.

A class is emptied only by a full run, because only the full matrix executes every
case; a subset run checks the entries it executed and nothing else.

Format:

    [classes.content-containment]
    description = "A narrower analysis request served from records of a wider one"
    clears_with = "Phase 1 item 2: content identity and equality serve"
    bead = "fdu-gija"

    [[violation]]
    class = "content-containment"
    paths = ["analysis.analyze[]"]
    keys = ["warm/cli-report/auto/W_all/-/a_lines"]

A violation may carry `platforms = ["win32"]` (values of `sys.platform`) when it occurs
only there; that is itself a finding to explain. Entries sharing a class, path set, and
platforms are grouped under one `keys` list.
"""

from __future__ import annotations

import tomllib
from collections import defaultdict
from collections.abc import Iterable
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

DEFAULT_PATH = Path(__file__).resolve().parent / "known-violations.toml"
UNCLASSIFIED = "unclassified"

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
    """One registered violation."""

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
    entries: dict[str, Entry] = field(default_factory=dict)


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
    unknown = set(data) - {"classes", "violation"}
    if unknown:
        raise ValueError(f"unknown registry tables: {sorted(unknown)}")
    registry = Registry()
    for name, table in data.get("classes", {}).items():
        registry.classes[name] = ViolationClass(
            name, table["description"], table["clears_with"], table["bead"]
        )
    for group in data.get("violation", []):
        keys = group.get("keys", [])
        if "key" in group:
            keys = [group["key"], *keys]
        platforms = tuple(group["platforms"]) if "platforms" in group else None
        for key in keys:
            if key in registry.entries:
                raise ValueError(f"key registered twice: {key}")
            registry.entries[key] = Entry(key, group["class"], tuple(group["paths"]), platforms)
    return registry


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

    for entry in registry.entries.values():
        if entry.klass == UNCLASSIFIED:
            failures.append(Failure("entry is unclassified", entry.key))
        elif entry.klass not in registry.classes:
            failures.append(Failure("entry names an undefined class", entry.key, entry.klass))

    for key, allowed, paths in cases:
        entry = registry.entries.get(key)
        if entry is not None and not entry.applies_on(platform):
            entry = None
        if allowed:
            if entry is not None:
                failures.append(Failure("registered case now conforms; remove its entry", key))
        elif entry is None:
            failures.append(Failure("unregistered difference", key, ", ".join(paths)))
        elif tuple(sorted(paths)) != tuple(sorted(entry.paths)):
            detail = f"registered {list(entry.paths)}, observed {list(paths)}"
            failures.append(Failure("difference changed shape", key, detail))

    if full:
        used = {entry.klass for entry in registry.entries.values()}
        for name in sorted(set(registry.classes) - used):
            failures.append(Failure("class has no entries; remove it", None, name))
        executed = {key for key, _, _ in cases}
        for entry in registry.entries.values():
            if entry.applies_on(platform) and entry.key not in executed:
                failures.append(Failure("registered case is not in the full matrix", entry.key))
    return failures


def record(registry: Registry, judged: Iterable[Judged], *, platform: str) -> Registry:
    """The registry rewritten to match `judged`.

    Classes and platforms of retained keys are kept; a new key is `unclassified`, so the
    run fails until someone reads the difference and names its cause. Keys this run did
    not execute are kept unchanged.
    """
    updated = Registry(classes=dict(registry.classes))
    executed: set[str] = set()
    for key, allowed, paths in judged:
        executed.add(key)
        if allowed:
            continue
        previous = registry.entries.get(key)
        if previous is not None and previous.applies_on(platform):
            updated.entries[key] = Entry(
                key, previous.klass, tuple(sorted(paths)), previous.platforms
            )
        else:
            updated.entries[key] = Entry(key, UNCLASSIFIED, tuple(sorted(paths)))
    for key, entry in registry.entries.items():
        if key not in executed or not entry.applies_on(platform):
            updated.entries.setdefault(key, entry)
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
    for entry in registry.entries.values():
        groups[(entry.klass, entry.paths, entry.platforms)].append(entry.key)
    for (klass, paths, platforms), keys in sorted(groups.items(), key=_group_order):
        lines += ["", "[[violation]]", f"class = {_string(klass)}"]
        if platforms is not None:
            lines.append(f"platforms = {_array(platforms)}")
        lines.append(f"paths = {_array(paths)}")
        lines.append(f"keys = {_array(sorted(keys))}")
    return "\n".join(lines) + "\n"


def _group_order(item: tuple[tuple[str, tuple[str, ...], Any], list[str]]) -> tuple[str, str]:
    (klass, _, _), keys = item
    return (klass, min(keys))


def _bare_or_quoted(name: str) -> str:
    if name and all(char.isascii() and (char.isalnum() or char in "-_") for char in name):
        return name
    return _string(name)


def _string(value: str) -> str:
    escaped = []
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
