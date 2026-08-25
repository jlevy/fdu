"""End-user smoke test for an installed fdu wheel.

This file intentionally has no third-party imports. CI runs it in a virtual environment
that contains only the built wheel, which catches accidental runtime dependencies and
source-tree imports.
"""

from __future__ import annotations

import ast
import asyncio
import contextlib
import dataclasses
import hashlib
import importlib.util
import json
import os
import re
import subprocess
import sys
import tempfile
import threading
import time
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any, cast

import fdu
from fdu import _native


def _stable(text: str) -> str:
    """Blank the fields that differ between any two runs, and only those."""

    return re.sub(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{9}Z", "[TIME]", text)


def check_watch_reports_its_own_index(root: Path) -> None:
    """A repaint must come from the session's index, not the one it was opened from.

    The aggregates stop being true at the first event, so reporting the opened index
    repaints numbers that never change while claiming to be live -- a display that looks
    like it works and does not (fdu-m66a).
    """

    index = fdu.open(root)
    with index.watch(fdu.WatchOptions(interval=0.2)) as watch:
        live = watch.report()
        # A snapshot: rendering twice gives the same answer both times.
        assert live.render(fdu.Format.TEXT) == live.render(fdu.Format.TEXT)
        assert live.status.source is not None

    # And a change record renders as the CLI streams it, rather than as repr().
    record = fdu.Change(clock=1, path=Path("a.txt"), kind=fdu.ChangeKind.UPSERT)
    line = record.render(fdu.Format.JSONL)
    assert '"schema": "fdu.stream/1"' in line, line
    assert '"op": "upsert"' in line, line
    assert "\t" in record.render(fdu.Format.TEXT)


def check_the_dirty_set_names_every_moved_rollup() -> None:
    """A batch says which directories' roll-ups it may have moved.

    A consumer caching a per-directory answer invalidates exactly these and keeps the
    rest, instead of re-deriving the set from change paths or dropping every cached row.
    The oracle here is computed from the change paths rather than taken from the code
    that produced the set, so the two have to agree independently.
    """

    # Its own tree: this check writes a file while watching, and the shared fixture's
    # entry counts are asserted by other checks.
    own = Path(tempfile.mkdtemp(prefix="fdu-dirty-set-"))
    nested = own / "src" / "deep"
    nested.mkdir(parents=True)
    (own / "seed.txt").write_text("seed", encoding="utf-8")
    index = fdu.open(own)
    with index.watch(fdu.WatchOptions(interval=0.05)) as watch:
        time.sleep(0.5)
        (nested / "leaf.txt").write_text("leaf", encoding="utf-8")
        for batch in watch:
            if not batch.changes:
                continue
            # Compared as paths throughout: the engine's root is the empty path, which
            # Python normalizes to Path("."), so comparing strings would fail on a
            # spelling rather than on the set.
            dirty = set(batch.dirty_rollups)
            expected: set[Path] = set()
            for change in batch.changes:
                parent = change.path.parent
                while True:
                    expected.add(parent)
                    if parent == Path("."):
                        break
                    parent = parent.parent
            assert expected <= dirty, f"ancestors {expected - dirty} missing from {dirty}"
            assert Path(".") in dirty, "the root's totals always move"
            break


def check_a_bounded_tree_says_what_it_withheld() -> None:
    """A truncated tree node carries the aggregate of the rows it dropped.

    "Truncate freely, never silently": the emitted children plus the remainder account
    for every directory beneath the node, so a caller can render an "other" cell without
    a second query. Checked against the same query with the bound lifted rather than
    against a hand-written number, which would agree with a remainder taken from the
    wrong side of the cut.
    """

    # Its own tree: the shared fixture has one directory at the root, so a limit of
    # one would withhold nothing there and the check would pass without testing anything.
    root = Path(tempfile.mkdtemp(prefix="fdu-remainder-"))
    for name, size in (("alpha", 8), ("beta", 4), ("gamma", 2)):
        (root / name).mkdir()
        (root / name / "f.txt").write_text("x" * size, encoding="utf-8")

    index = fdu.open(root)
    full = _tree_section(index.report(fdu.Query(views=(fdu.View.TREE,))))
    assert len(full.children) == 3, full.children
    assert full.remainder is None, "an unbounded tree withheld nothing"

    bounded = _tree_section(
        index.report(fdu.Query(views=(fdu.View.TREE,), selection=fdu.Selection(limit=1)))
    )
    remainder = bounded.remainder
    assert remainder is not None, "one row kept out of several is a truncation"
    assert bounded.truncated is True
    assert len(bounded.children) + remainder.rows == len(full.children)
    assert sum(child.bytes for child in bounded.children) + remainder.bytes == sum(
        child.bytes for child in full.children
    )
    assert sum(child.files for child in bounded.children) + remainder.files == sum(
        child.files for child in full.children
    )
    fdu.clear_cache(root)


def _tree_section(report: fdu.Report) -> fdu.TreeNode:
    for section in report.sections:
        if isinstance(section, fdu.TreeSection):
            return section.tree
    raise AssertionError("the report has no tree section")


def check_empty_is_decidable_from_the_aggregate() -> None:
    """A directory of symlinks weighs nothing and is not nothing.

    Files, directories and bytes are all zero for both of these, so the aggregate could
    not tell them apart until non-file leaves were counted. A listing that greys out
    empty directories was greying out one with contents, or greying out none.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-empty-"))
    (root / "target.txt").write_text("x", encoding="utf-8")
    (root / "hollow").mkdir()

    # Creating a symlink on Windows needs a privilege the runner may not hold, so the
    # half of this that needs one is Unix-only -- the same guard the kind-label check
    # uses. The distinction itself is covered on every platform by the engine test
    # `a_symlink_only_subtree_is_not_an_empty_one`, which builds its tree from
    # observations rather than from a filesystem; what is Unix-only here is whether the
    # count survives a real walk and the binding, not whether it is maintained.
    links_are_testable = os.name != "nt"
    if links_are_testable:
        (root / "links").mkdir()
        (root / "links" / "a").symlink_to(root / "target.txt")
        (root / "links" / "b").symlink_to(root / "target.txt")

    index = fdu.open(root)
    hollow = index.rollup("hollow")
    assert hollow is not None
    assert (hollow.files, hollow.dirs, hollow.bytes) == (0, 0, 0)
    assert hollow.others == 0
    assert hollow.is_empty and hollow.entries == 0

    if links_are_testable:
        links = index.rollup("links")
        assert links is not None
        assert (links.files, links.dirs, links.bytes) == (0, 0, 0), (
            "a symlink weighs nothing, which is exactly why bytes cannot decide this"
        )
        assert links.others == 2, "and is counted anyway"
        assert not links.is_empty

    # And a listing row carries the verdict, decided rather than left to the consumer:
    # deciding it needs the row's provenance as well as its counts.
    page = index.children()
    assert page is not None
    rows = {child.name: child for child in page.rows}
    assert rows["hollow"].empty is True
    assert rows["target.txt"].empty is None, "a file has no subtree to be empty"
    if links_are_testable:
        assert rows["links"].empty is False
        assert rows["links"].totals is not None and rows["links"].totals.entries == 2

    fdu.clear_cache(root)


def check_partial_coverage_says_why() -> None:
    """A complete value carries no reason, and the vocabulary is reachable from Python.

    The engine spells the reason inside the partial variant, so the two cannot disagree;
    what this pins is that the binding surfaces it and that a complete value stays
    reason-free, which is the half a consumer branches on.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-coverage-"))
    (root / "a.txt").write_text("x", encoding="utf-8")

    index = fdu.open(root)
    provenance = index.provenance()
    assert provenance is not None
    assert provenance.status is fdu.Coverage.COMPLETE
    assert provenance.reason is None, "a complete value has nothing to explain"

    # The contract's whole vocabulary is importable, so a consumer can match on it today
    # rather than after the engine learns to produce every member.
    assert {reason.value for reason in fdu.CoverageReason} == {
        "building",
        "budget",
        "cancelled",
        "inaccessible",
        "failed",
    }

    fdu.clear_cache(root)


def check_tags_are_a_named_fact_per_entry() -> None:
    """A tag rides on the entry, and asking about a rule that is off is refused.

    Three things are pinned here that a Rust unit test cannot reach: that enabling is
    Scope (``ScanOptions.tag_rules``) while filtering is Selection, that a listing row and
    a report row agree about the same entry, and that a rule which is not enabled raises
    rather than quietly matching nothing -- a mask of zero being indistinguishable from no
    constraint, so the permissive reading would hand back everything.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-tags-"))
    (root / ".cache").mkdir()
    (root / ".cache" / "blob").write_text("z", encoding="utf-8")
    (root / ".env").write_text("x", encoding="utf-8")
    (root / "main.rs").write_text("y", encoding="utf-8")

    index = fdu.open(root, scan=fdu.ScanOptions(tag_rules=("dotfile",)))
    page = index.children()
    assert page is not None
    rows = {child.name: child for child in page.rows}
    assert rows[".env"].tags == ("dotfile",)
    assert rows[".cache"].tags == ("dotfile",), "a directory is as taggable as a file"
    assert rows["main.rs"].tags == ()

    # The same entry, through the report surface rather than the listing.
    report = index.report(
        fdu.Query(
            views=(fdu.View.FILES,),
            selection=fdu.Selection(tags=("dotfile",), limit=fdu.Bound.ALL),
        )
    )
    tagged = {
        row.path.as_posix(): row.tags
        for section in report.sections
        if isinstance(section, fdu.FilesSection)
        for row in section.files
    }
    assert set(tagged) == {".env", ".cache"}, tagged
    assert all(tags == ("dotfile",) for tags in tagged.values()), tagged

    # A tag is about the entry, never its ancestors: excluding `.cache` keeps what is
    # inside it, which is what separates a tag from scope pruning.
    excluded = index.report(
        fdu.Query(
            views=(fdu.View.FILES,),
            selection=fdu.Selection(not_tags=("dotfile",), limit=fdu.Bound.ALL),
        )
    )
    kept = {
        row.path.as_posix()
        for section in excluded.sections
        if isinstance(section, fdu.FilesSection)
        for row in section.files
    }
    assert kept == {".cache/blob", "main.rs"}, kept

    # An index that never evaluated the rule cannot answer for it, and says so.
    plain = fdu.open(root, cache=fdu.CachePolicy.OFF)
    try:
        plain.report(fdu.Query(selection=fdu.Selection(tags=("dotfile",))))
    except fdu.InvalidArgumentError as error:
        assert "not enabled" in str(error), error
    else:  # pragma: no cover - the failure this guards is a silent one
        raise AssertionError("filtering on a rule that is off must be refused")

    fdu.clear_cache(root)


def check_a_promoted_plane_serves_a_dual_value_listing_from_one_call() -> None:
    """A browser shows two numbers per row, and gets both from one read.

    The listing this exists for is "1.2 GB, 340 MB shown". Reaching the second figure a row
    at a time would take the read guard once per child and re-resolve each path, so the
    plane rides on the page request and every directory row answers with both -- which is
    the part no Rust test can pin, because it is about what one Python call returns.

    Three refusals are pinned beside it, because they need three different next steps:
    promote the rule, enable it, or fix the spelling. The middle one matters most -- an
    unpromoted plane served from the totals would look right on exactly the trees that
    cannot tell the difference.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-plane-"))
    (root / "src").mkdir()
    (root / "src" / "main.rs").write_text("fn main() {}", encoding="utf-8")
    (root / "src" / ".env").write_text("SECRET=1", encoding="utf-8")
    (root / ".cache").mkdir()
    (root / ".cache" / "blob").write_text("cached", encoding="utf-8")

    scan = fdu.ScanOptions(tag_rules=("dotfile",), promote=("dotfile",))
    index = fdu.open(root, scan=scan)

    page = index.children(plane="dotfile")
    assert page is not None
    rows = {child.name: child for child in page.rows}

    src = rows["src"]
    assert src.totals is not None and src.plane is not None
    assert src.totals.files == 2, src.totals
    assert src.plane.files == 1, "the plane holds what does not carry the tag"
    assert src.plane.bytes < src.totals.bytes

    # A tagged directory is outside the plane as an entry, and its untagged contents are
    # still inside it: a tag rides on the entry, never on its ancestors.
    cache = rows[".cache"]
    assert cache.totals is not None and cache.plane is not None
    assert cache.plane.files == 1, "blob is not itself a dotfile"

    # Not asking costs nothing and says nothing.
    plain = index.children()
    assert plain is not None
    assert all(child.plane is None for child in plain.rows)

    # The whole-tree roll-up answers in the plane too, and the complement subtracts.
    whole = index.total()
    plane = index.total(plane="dotfile")
    assert plane.files < whole.files
    assert plane.bytes < whole.bytes

    # And a report answers in the plane while staying on the roll-up tier: the same
    # numbers `not_tags` reaches by re-aggregating the whole index.
    def totals(report: fdu.Report) -> fdu.SummaryRow:
        for section in report.sections:
            if isinstance(section, fdu.SummarySection):
                return section.summary
        raise AssertionError("a default report carries a summary section")

    read = index.report(
        fdu.Query(views=(fdu.View.SUMMARY,), selection=fdu.Selection(plane="dotfile"))
    )
    walked = index.report(
        fdu.Query(views=(fdu.View.SUMMARY,), selection=fdu.Selection(not_tags=("dotfile",)))
    )
    assert totals(read) == totals(walked), (totals(read), totals(walked))

    for name, expected in [
        ("dotfile", None),
        ("gitignore", "not enabled"),
        ("dotfil", "unknown tag rule"),
    ]:
        unpromoted = fdu.open(root, scan=fdu.ScanOptions(tag_rules=("dotfile",)))
        try:
            unpromoted.total(plane=name)
        except fdu.InvalidArgumentError as error:
            wanted = expected if expected is not None else "maintains no plane"
            assert wanted in str(error), (name, str(error))
        else:  # pragma: no cover - the failure this guards is a silent one
            raise AssertionError(f"naming plane {name!r} must be refused")

    fdu.clear_cache(root)


def check_pruned_hidden_paths_are_absent_rather_than_filtered() -> None:
    """Pruning changes what the index holds; a tag changes what a query returns.

    The axis test, from the surface that makes it easiest to state wrongly. A caller who
    reaches for `not_tags=("dotfile",)` gets the same rows and pays for the whole tree to be
    walked; one who sets `hidden="prune"` gets an index that never held them. So the
    assertion is absence -- `children()` has no row, `rollup()` has no path -- rather than a
    smaller number coming back from a report.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-hidden-"))
    (root / "src").mkdir()
    (root / ".git").mkdir()
    (root / ".github").mkdir()
    (root / "src" / "main.rs").write_text("fn main() {}", encoding="utf-8")
    (root / "src" / ".env").write_text("SECRET=1", encoding="utf-8")
    (root / ".git" / "HEAD").write_text("ref: refs/heads/main", encoding="utf-8")
    (root / ".github" / "ci.yml").write_text("on: push", encoding="utf-8")

    kept = fdu.open(root, cache=fdu.CachePolicy.OFF)
    kept_page = kept.children()
    assert kept_page is not None
    assert {child.name for child in kept_page.rows} == {"src", ".git", ".github"}

    pruned = fdu.open(root, cache=fdu.CachePolicy.OFF, scan=fdu.ScanOptions(hidden="prune"))
    page = pruned.children()
    assert page is not None
    assert {child.name for child in page.rows} == {"src"}, [c.name for c in page.rows]
    assert pruned.rollup(".git") is None, "not a path this index has"
    assert pruned.total().files < kept.total().files

    # A hidden entry inside a kept directory goes too, and one hidden directory can be
    # named back in without admitting the rest.
    inner = pruned.children("src")
    assert inner is not None
    assert {child.name for child in inner.rows} == {"main.rs"}

    allowed = fdu.open(
        root,
        cache=fdu.CachePolicy.OFF,
        scan=fdu.ScanOptions(hidden="prune", hidden_allow=(".github",)),
    )
    allowed_page = allowed.children()
    assert allowed_page is not None
    assert {child.name for child in allowed_page.rows} == {"src", ".github"}

    # Refused by the engine, which is the point: the command line and this package reject
    # the same input with the same sentence, because one rule judges it rather than two
    # copies of it. Validating again in `ScanOptions.__post_init__` would fail earlier and
    # say something slightly different, which is how a surface drifts.
    for options, expected in [
        ({"hidden_allow": (".github",)}, "needs hidden pruning"),
        ({"hidden": "sometimes"}, "expected one of keep, prune"),
    ]:
        try:
            fdu.open(root, cache=fdu.CachePolicy.OFF, scan=fdu.ScanOptions(**options))
        except fdu.InvalidArgumentError as error:
            assert expected in str(error), (options, str(error))
        else:  # pragma: no cover - the failure this guards is a silent one
            raise AssertionError(f"{options} must be refused")

    # A bare string where a tuple belongs stays a Python-shape error, because there is no
    # command line spelling of that mistake for it to disagree with.
    try:
        fdu.ScanOptions(hidden_allow=".github")
    except TypeError as error:
        assert "wrap the single value in a tuple" in str(error), error
    else:  # pragma: no cover - the failure this guards is a silent one
        raise AssertionError("a bare string allowlist must be refused")


def check_the_browser_provider_example_holds_the_contract_it_documents() -> None:
    """The shipped embedder example, against the semantics a second engine has to match.

    Four claims, each of which fails silently if it is wrong, and each of which is the
    reason the example exists rather than a paragraph of prose:

    1. One identity string moves when *any* named fingerprint moves. A consumer keying on a
       subset caches an answer across a change that invalidated it, and nothing in the
       answer says so.
    2. A directory row carries both numbers from one call, and the plane is a real
       restriction rather than a copy of the totals.
    3. Symlinks are leaves: counted, never followed, so a cycle is not a hang and a symlink
       farm is not an empty directory.
    4. A `.gitignore` negation beats a broader rule above it, and it still does when the
       control file itself was pruned out of the index.

    The example is loaded from the file that ships, so the tested code and the documented
    code are the same code.
    """

    example = _load_example("browser_provider.py")

    # (1) Identity, and it is two digests rather than one. Scope says what was admitted to
    # the index; semantic says what the rules make of it. A consumer keying on one combined
    # value re-reads for both; one keying on neither serves an answer across a change that
    # invalidated it.
    #
    # The expected bytes are built by hand rather than by calling the example twice, so the
    # test says what the recipe *is*: sorted `[name, value]` **string** pairs, compact JSON,
    # SHA-256. A test that compared the function to itself would have accepted the first
    # draft, which hashed objects with integer values -- internally consistent, and agreeing
    # with nothing.
    def pair_digest(components: dict[str, str]) -> str:
        return hashlib.sha256(
            json.dumps(
                [[name, components[name]] for name in sorted(components)],
                ensure_ascii=True,
                separators=(",", ":"),
            ).encode("utf-8")
        ).hexdigest()

    base = fdu.ScanScope(
        max_depth=None,
        max_files=None,
        follow_symlinks=False,
        one_filesystem=False,
        tag_rules_fingerprint=11,
        type_rules_fingerprint=22,
        reducers_fingerprint=33,
        hidden_fingerprint=44,
    )
    expected_semantic = pair_digest({"reducers": "33", "tag_rules": "11", "type_rules": "22"})
    assert example.semantic_fingerprint(base) == expected_semantic

    options = fdu.ScanOptions(hidden="prune", hidden_allow=(".github", ".cargo"))
    expected_scope = pair_digest(
        {
            "follow_symlinks": "false",
            # A structured value is a compact canonical JSON string inside the outer array.
            "hidden_allowlist": '["' + '","'.join([".cargo", ".github"]) + '"]',
            "max_depth": "null",
            "max_files": "null",
            "stay_on_filesystem": "false",
        }
    )
    assert example.scope_fingerprint(options) == expected_scope

    # The two move independently, which is the whole reason there are two. Every component
    # of each reaches its own digest and neither reaches the other's.
    for field, value in [
        ("tag_rules_fingerprint", 99),
        ("type_rules_fingerprint", 99),
        ("reducers_fingerprint", 99),
    ]:
        moved = dataclasses.replace(base, **{field: value})
        assert example.semantic_fingerprint(moved) != expected_semantic, field

    # The engine's own `hidden_fingerprint` is a cache key, not the consumer's encoding:
    # the scope digest carries the allowlist itself, because a digest of a list is not the
    # list and no second implementation could reproduce it.
    assert (
        example.semantic_fingerprint(dataclasses.replace(base, hidden_fingerprint=99))
        == expected_semantic
    ), "hidden admission is scope identity, not semantic identity"
    assert (
        example.semantic_fingerprint(dataclasses.replace(base, max_files=1000)) == expected_semantic
    ), "a walk budget is scope identity too: it changes which entries exist, not their meaning"

    for changed in [
        dataclasses.replace(options, hidden_allow=(".github",)),
        dataclasses.replace(options, max_depth=3),
        dataclasses.replace(options, max_files=1000),
        dataclasses.replace(options, one_filesystem=True),
    ]:
        assert example.scope_fingerprint(changed) != expected_scope, changed

    # The allowlist is a set, so its written order is not part of its identity.
    assert (
        example.scope_fingerprint(dataclasses.replace(options, hidden_allow=(".cargo", ".github")))
        == expected_scope
    )

    root = Path(tempfile.mkdtemp(prefix="fdu-provider-"))
    (root / "docs").mkdir()
    (root / ".git").mkdir()
    (root / ".github").mkdir()
    (root / ".gitignore").write_text("*.log\n", encoding="utf-8")
    (root / "docs" / ".gitignore").write_text("!keep.log\n", encoding="utf-8")
    (root / "docs" / "keep.log").write_text("kept", encoding="utf-8")
    (root / "docs" / "drop.log").write_text("dropped", encoding="utf-8")
    (root / "docs" / "guide.md").write_text("# guide", encoding="utf-8")
    (root / ".git" / "HEAD").write_text("ref: refs/heads/main", encoding="utf-8")
    (root / ".github" / "ci.yml").write_text("on: push", encoding="utf-8")
    # A symlink to its own parent: followed, this walk does not terminate.
    try:
        (root / "docs" / "loop").symlink_to(root / "docs", target_is_directory=True)
        symlinks = True
    except (OSError, NotImplementedError):  # pragma: no cover - Windows without privilege
        symlinks = False

    index = example.open_tree(root)
    rows = {row.name: row for row in example.listing(index, Path())}

    # (2) and the hidden allowlist: `.git` is gone, `.github` is named back in.
    assert set(rows) == {"docs", ".github"}, sorted(rows)
    docs = rows["docs"]
    assert docs.is_dir and docs.shown_bytes is not None
    assert docs.shown_bytes < docs.bytes, "the plane excludes what git ignores"

    # (4) `drop.log` is ignored and `keep.log` is negated back in, decided by a control
    # file that is not in the index at all -- both `.gitignore` files were pruned.
    inner = {row.name: row for row in example.listing(index, Path("docs"))}
    assert ".gitignore" not in inner, "read, never retained"
    assert inner["drop.log"].tags == ("gitignore",), inner["drop.log"]
    assert inner["keep.log"].tags == (), "a nested negation beats a broader rule above it"
    assert inner["guide.md"].tags == ()

    # (3) A symlink is a leaf. The walk returned, which is most of the claim, and the row
    # is present rather than silently dropped.
    if symlinks:
        assert "loop" in inner, sorted(inner)
        assert not inner["loop"].is_dir, "counted as a leaf, never descended into"

    # (5) The bounded recency slice measures its window from the caller's instant, not from
    # its own. Shown by moving the instant rather than by waiting: `modified_since="1h"`
    # against a reference two hours ahead puts a file written now *below* the threshold,
    # which can only happen if the caller's `as_of` is what the window is measured from.
    #
    # A slice, deliberately not a page: `Selection.limit` bounds the rows and the section
    # reports what it withheld, but there is no cursor to resume from and no version to pin
    # a continuation to, so this cannot be assembled into a complete answer. That gate is
    # `fdu-91ru`, and the example says so rather than looking like it pages.
    def recent_names(as_of: datetime) -> set[str]:
        bundle = example.recent_slice(index, as_of=as_of, limit=50)
        assert bundle.report is not None
        section = bundle.report.sections[0]
        assert isinstance(section, fdu.FilesSection)
        return {row.path.name for row in section.files}

    now = datetime.now(tz=UTC)
    assert "guide.md" in recent_names(now), "written moments ago, inside a one-hour window"
    assert recent_names(now + timedelta(hours=2)) == set(), (
        "the window follows the caller's instant, so nothing is recent as of two hours hence"
    )

    fdu.clear_cache(root)


def _load_example(name: str) -> Any:
    """Import one shipped example by path, so the tested code is the code that ships."""

    path = Path(__file__).resolve().parent.parent / "examples" / name
    spec = importlib.util.spec_from_file_location(f"fdu_example_{path.stem}", path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    # Registered before execution: `@dataclass(slots=True)` rebuilds the class and looks its
    # module up in `sys.modules` to do it, so an unregistered module fails to load.
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def check_a_bundle_answers_a_query_at_the_same_instant_as_its_rows() -> None:
    """A composed page is one read, not several that happen to agree.

    The rows and a "recently changed" panel used to need two calls, and a write landing
    between them left the halves describing different moments -- each individually true.
    Passing a `Query` to `read()` puts both under one guard and one `clock`. What this
    pins from Python is that the report arrives, that it is the same value `report()`
    would return, and that the per-projection costs are separable: a bundle that reports
    only a total says "this read was slow" and never "which part of it".
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-bundle-query-"))
    (root / "src").mkdir()
    for index_of in range(4):
        (root / "src" / f"f{index_of}.txt").write_text("x" * (index_of + 1), encoding="utf-8")

    index = fdu.open(root)
    query = fdu.Query(views=(fdu.View.SUMMARY,))
    bundle = index.read(children_of="src", total=True, query=query)

    assert bundle.report is not None, "a query was passed, so a report must come back"
    summary = next(
        section for section in bundle.report.sections if isinstance(section, fdu.SummarySection)
    )
    assert bundle.total is not None
    assert summary.summary.files == bundle.total.files, (
        "the report and the totals beside it describe one instant"
    )

    # The same answer a standalone report gives, from one guard instead of two.
    standalone = index.report(query).sections[0]
    assert isinstance(standalone, fdu.SummarySection)
    assert summary.summary.files == standalone.summary.files

    # Separable costs. The guard wait is the bundle's alone, because the projections
    # waited together -- which the type now says rather than the values: a projection
    # carries no lock wait to be zero, and its span is named `engine_ns` because it is one
    # phase of the engine read rather than a public call's total.
    parts = bundle.projections
    assert parts.children.rows > 0, "a listing was asked for and returned rows"
    assert parts.report.rows > 0, "the summary section is one row"
    assert not hasattr(parts.children, "lock_wait_ns")
    assert parts.children.engine_ns <= bundle.work.native_ns, (parts, bundle.work)
    counted = parts.children.rows + parts.total.rows + parts.rollups.rows + parts.report.rows
    assert bundle.work.rows == counted, (bundle.work.rows, counted)

    # No query asked for, nothing charged to that projection.
    plain = index.read(total=True)
    assert plain.report is None
    assert plain.projections.report.rows == 0

    fdu.clear_cache(root)


def check_a_listing_pages_and_accounts_for_the_rest() -> None:
    """A wide directory is drawn a page at a time, and the page says what it left out.

    The two facts a partial listing has to carry are different, and conflating them is
    the bug this pins: the remainder is this page's complement in the whole directory,
    so it stays present on the last page, while `next` is what says paging continues.
    A consumer looping on `truncated` would never stop.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-page-"))
    for index_of in range(10):
        (root / f"dir{index_of}").mkdir()
        (root / f"dir{index_of}" / "f.txt").write_text("x" * (index_of + 1), encoding="utf-8")

    index = fdu.open(root)
    whole = index.total()

    seen: list[str] = []
    after: str | None = None
    pages = 0
    while True:
        page = index.children(after=after, limit=3)
        assert page is not None
        pages += 1
        seen.extend(child.name for child in page.rows)
        assert page.truncated, "three rows never cover ten children"

        # Rows plus remainder are the directory, on every page.
        shown = sum(child.totals.bytes for child in page.rows if child.totals)
        assert page.remainder is not None
        assert shown + page.remainder.bytes == whole.bytes
        assert len(page.rows) + page.remainder.rows == 10

        if not page.has_next:
            break
        after = page.next
        assert pages <= 10, "the cursor must make progress"

    assert seen == sorted(seen), "children arrive in name order"
    assert len(seen) == len(set(seen)) == 10, "every child once"
    assert pages == 4, "ten children, three at a time"

    # The whole directory in one call reports neither, by omission.
    everything = index.children()
    assert everything is not None
    assert len(everything.rows) == 10
    assert not everything.truncated and not everything.has_next

    # A file is not a directory, which is distinct from a directory with no children.
    assert index.children("dir0/f.txt") is None
    assert index.children("nope") is None
    empty = index.children("dir0")
    assert empty is not None and len(empty.rows) == 1

    fdu.clear_cache(root)


def check_one_bundle_answers_a_whole_page() -> None:
    """A composed page comes from one read, at one instant, with its own cursor.

    The listing, the totals it is summarised by, and the version to resume from all come
    back together. Read separately, a write can land between them and the page is
    internally inconsistent in a way nothing in it reports.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-bundle-"))
    (root / "src").mkdir()
    (root / "src" / "main.rs").write_text("fn main() {}", encoding="utf-8")
    (root / "docs").mkdir()
    (root / "docs" / "guide.md").write_text("# guide", encoding="utf-8")
    # No extension, so the payload oracle below sees the absent-string case too. It sits
    # under `docs` rather than at the root, where the entries-visited arithmetic is pinned.
    (root / "docs" / "NOTICE").write_text("terms", encoding="utf-8")

    index = fdu.open(root)
    # A second view of the same tree, with content analysis on: the metric sections exist
    # only there, and the payload rule has to hold for them too.
    analyzed = fdu.open(
        root, cache=fdu.CachePolicy.OFF, analysis=fdu.AnalysisOptions(analyze=fdu.Analysis.ALL)
    )
    page = index.read(children_of=".", rollups=("src", "missing"), total=True, extensions=1)

    assert page.children is not None
    assert {child.name for child in page.children.rows} == {"src", "docs"}
    assert page.total is not None
    # The rows and the header describe one instant, so they add up by construction.
    assert (
        sum(child.totals.files for child in page.children.rows if child.totals) == page.total.files
    )

    assert len(page.rollups) == 2
    assert page.rollups[0] is not None and page.rollups[0].files == 1
    assert page.rollups[1] is None, "an absent path is None, not an empty roll-up"

    assert page.clock == index.clock, "the bundle reports the version it read"
    assert page.entries == len(index)
    assert page.status.complete is True
    assert page.status.source is fdu.ReportSource.COLD_SCAN
    assert os.path.samefile(page.root, root)

    # The identity a consumer cache key derives from travels with the read.
    assert page.scope.type_rules_fingerprint == fdu.TypeRegistry.compiled().fingerprint
    assert page.scope.max_depth is None

    # The extension bound reaches every roll-up in the bundle.
    for rollup in (page.total, page.rollups[0]):
        if rollup is not None:
            assert len(rollup.by_extension) <= 1

    # A listing row carries scalars, and its breakdown is a separate projection. The
    # bundle asked for children and roll-ups in one call precisely so a consumer can
    # have both without a second crossing.
    src = next(child for child in page.children.rows if child.name == "src")
    assert src.totals is not None and src.totals.files == 1
    assert page.rollups[0].files == src.totals.files

    # Every bundle says what it cost, beside what it answered, and the number is the
    # sum of what its projections did rather than a fact about the tree. Here: the
    # listing walks the root and its two rows (3), the totals read the root (1), the
    # "src" roll-up walks root and src (2), and the absent path stops at the root (1).
    assert page.work.rows == 2
    assert page.work.entries_visited == 3 + 1 + 2 + 1
    assert page.work.dirs_visited == page.work.entries_visited, "nothing here is a file"

    # The extension bound applies to the rows returned, not to the tallies ranked to
    # choose them -- which is the whole reason the counter reports what it examined.
    assert page.work.tally_rows > len(page.total.by_extension)
    assert page.work.lock_wait_ns <= page.work.wall_ns
    assert page.work.wall_seconds > 0

    # A pinned read returns exactly that version or raises. A name cursor keeps page two
    # from skipping or repeating a row; only a pin keeps page two from describing a
    # different tree than page one, which is what a caller assembling a complete answer
    # from bounded pages needs.
    pinned = index.read(children_of="docs", total=True, expected=page.cursor)
    assert pinned.cursor == page.cursor

    # And the cursor works: what happened after it is what changed since.
    (root / "docs" / "extra.md").write_text("more", encoding="utf-8")
    index.refresh()
    changed = index.since(fdu.Cursor(session=index.cursor().session, clock=page.clock))
    assert not changed.truncated
    assert any(change.path == Path("docs/extra.md") for change in changed.changes)
    assert changed.cursor.clock >= page.clock

    # The tree moved, so the earlier pin has aged out. Failing is the designed answer:
    # only the current version is retained, and a caller restarts a bounded assembly
    # rather than the engine holding history so a stale pin could succeed.
    try:
        index.read(children_of="docs", expected=page.cursor)
    except fdu.FduError:
        pass
    else:
        raise AssertionError("a pin the index has moved past must not be answered")

    # An upper size bound, which a catalog query needs so a consumer stops carrying
    # candidates across the binding only to discard them.
    files_only = fdu.Selection(kinds=(fdu.EntryKind.FILE,), size=fdu.SizeMetric.APPARENT)
    everything = index.report(fdu.Query(views=(fdu.View.FILES,), selection=files_only))
    sizes = sorted(row.bytes for row in everything.sections[0].files)
    assert len(sizes) >= 2, sizes

    # A cap below the largest file admits strictly fewer, which is the whole claim.
    cap = sizes[-1] - 1
    capped = index.report(
        fdu.Query(
            views=(fdu.View.FILES,),
            selection=dataclasses.replace(files_only, max_size=cap),
        )
    )
    assert all(row.bytes <= cap for row in capped.sections[0].files)
    assert len(capped.sections[0].files) < len(sizes), "the cap must exclude the largest"

    # A reversed window admits nothing, rather than quietly preferring one bound.
    windowed = index.report(
        fdu.Query(
            views=(fdu.View.FILES,),
            selection=dataclasses.replace(files_only, min_size=sizes[-1], max_size=sizes[0]),
        )
    )
    assert windowed.sections[0].files == (), "a reversed window admits nothing"

    # A zero page limit is refused: it would be truncated and terminal at once, with a
    # remainder saying there is more and no cursor to ask with.
    try:
        index.read(children_of="docs", limit=0)
    except fdu.FduError:
        pass
    else:
        raise AssertionError("a zero page limit must be refused")

    # The GIL is released for the duration of the native read, through the real public
    # `Index.read`. Threads that merely all finish would prove nothing -- with the GIL held
    # they run one after another and still finish -- so the oracle is whether a second
    # thread runs *during* the call.
    #
    # Forced rather than timed, and the switch interval is what forces it. Earlier versions
    # asked whether a probe happened to be scheduled inside the read, which is a claim
    # about the machine: CI disproved one such threshold on a fast host, where the read
    # finished before the probe was due. Raising the interval makes the interpreter stop
    # handing the GIL away *voluntarily*, so between the two `len()` calls below the main
    # thread cannot yield -- while a GIL the native read actually releases is handed over
    # immediately, interval or not. The count therefore separates the two cases exactly:
    # zero if the read held it, positive if it did not, at any speed.
    wide = Path(tempfile.mkdtemp(prefix="fdu-gil-"))
    for group in range(20):
        bucket = wide / f"d{group}"
        bucket.mkdir()
        for n in range(1000):
            (bucket / f"f{n}.rs").write_text("x", encoding="utf-8")
    wide_index = fdu.open(wide, cache=fdu.CachePolicy.OFF)

    filtered = fdu.Query(
        views=(fdu.View.SUMMARY,),
        selection=fdu.Selection(min_size=1, kinds=(fdu.EntryKind.FILE,)),
    )
    turns: list[int] = []
    running = threading.Event()
    stop = threading.Event()

    def contend() -> None:
        running.set()
        while not stop.is_set():
            turns.append(1)
            # Yields the GIL every iteration, so a long switch interval cannot starve the
            # thread under test while it is still setting up.
            time.sleep(0)

    contender = threading.Thread(target=contend, name="fdu-gil-contender")
    contender.start()
    previous_interval = sys.getswitchinterval()
    sys.setswitchinterval(5.0)
    try:
        assert running.wait(timeout=30), "the contending thread never started"
        before = len(turns)
        measured = wide_index.read(total=True, query=filtered)
        during = len(turns) - before
    finally:
        sys.setswitchinterval(previous_interval)
        stop.set()
        contender.join(timeout=30)

    assert measured.work.entries_visited > 1000, (
        f"the read must do real work for this to mean anything: {measured.work}"
    )
    assert during > 0, (
        "another Python thread must run *while* the native read works, not after it: it "
        f"got {during} turns during a call that visited {measured.work.entries_visited} "
        "entries"
    )

    # The binding's own cost, which no engine-side counter can see. Checked against an
    # independent implementation of the stated rule rather than against a hand-computed
    # figure for one field family: a per-family expectation only covers the family it was
    # written for, and this measurement drifted twice by omitting families nobody had
    # written an expectation for. The oracle covers every field that crosses, including
    # ones added later.
    for shape in (
        {},
        {"total": True},
        {"rollups": [str(Path("src")), str(Path("does-not-exist"))]},
        # A page that withholds rows, and one that does not: the remainder is a dict in the
        # first case and a null in the second, and both are payload.
        {"children_of": "docs", "limit": 1},
        {"children_of": "docs"},
        {"children_of": "."},
        # An extension bound leaves a remainder; without one it is absent. The pair is
        # deliberate -- the over-charge that shipped here was invisible to any shape that
        # never produced the dict.
        {"total": True, "extensions": 1},
        {"children_of": ".", "rollups": [str(Path("src"))], "total": True, "extensions": 1},
        {
            "total": True,
            "report": True,
            "views": ["summary", "extensions", "files", "tree", "groups"],
        },
        # Bounded sections and a bounded tree, so every `bound` and `remainder` is present
        # rather than null.
        {
            "report": True,
            "views": ["extensions", "files", "tree", "groups"],
            "limit_rows": "1",
            "depth": "1",
        },
    ):
        for reader in (index, analyzed):
            raw = reader._native.read(**shape)
            assert _payload_bytes(raw) == raw["work"]["binding_bytes"], (
                f"payload accounting disagrees for {shape}: rule says "
                f"{_payload_bytes(raw)}, the binding charged {raw['work']['binding_bytes']}"
            )
    # The metric families ride on an analysed index and on nothing else, so a shape list
    # over a plain one never reaches `metric_row_dict` at all.
    metrics = analyzed._native.read(report=True, views=["types", "families", "languages"])
    assert _payload_bytes(metrics) == metrics["work"]["binding_bytes"], (
        f"payload accounting disagrees for the metric views: rule says "
        f"{_payload_bytes(metrics)}, the binding charged {metrics['work']['binding_bytes']}"
    )
    # And the shape reached the rows, or the agreement above is about an empty section.
    assert any(
        section.get("metrics", {}).get("rows")
        for section in cast("list[dict[str, Any]]", metrics["report"]["reports"])
    ), "the metric views produced no rows, so this shape proves nothing about them"

    # Every phase is named, and they compose into the one end-to-end figure an embedder
    # should compare providers on -- without overlapping, which is what makes summing them
    # meaningful.
    assert measured.work.native_ns > 0, measured.work
    assert measured.work.conversion_ns > 0, measured.work
    assert measured.work.model_ns > 0, measured.work
    assert (
        measured.work.wall_ns
        >= measured.work.native_ns + measured.work.conversion_ns + measured.work.model_ns
    ), measured.work
    assert measured.work.cpu_ns is None, "CPU is absent rather than inferred"

    # A projection's span is engine-local and says so in its own type. While both were
    # `Work`, `wall_ns` meant the whole public call on one and a span inside the guard on
    # the other, and nothing distinguished them at the point of use.
    assert measured.projections.total.engine_ns <= measured.work.native_ns, measured.projections
    assert not hasattr(measured.projections.total, "wall_ns")

    # An empty request is the constant-work checkpoint: it carries the envelope and reads
    # no entries, which is what lets a caller validate a cached body before paying.
    checkpoint = index.read()
    assert checkpoint.work.entries_visited == 0, checkpoint.work
    assert checkpoint.total is None and checkpoint.children is None
    assert checkpoint.cursor.session == index.cursor().session
    fdu.clear_cache(root)


#: Maps whose keys are drawn from the answer rather than from the schema, so their keys
#: are payload like any other value. A schema key is identical on every result of the same
#: shape and counting it would make the figure grow with the schema instead of the answer.
_DATA_KEYED_MAPS = frozenset({"by_extension", "by_group", "coverage", "sources", "confidence"})

#: Telemetry about the call rather than part of the answer, so it is not payload. It is
#: also built after the count is taken, which is what makes the count self-consistent.
_NOT_PAYLOAD = frozenset({"work", "projections"})


def _payload_bytes(value: object, charge_keys: bool = False) -> int:
    """The documented binding-payload rule, implemented independently of the binding.

    Every string value contributes its own UTF-8 bytes and every fixed-width leaf -- an
    integer, a float, a boolean, a null -- contributes eight. This is deliberately a second
    implementation: agreeing with the binding's own accumulator is the evidence, and a
    helper shared with it would agree by construction and prove nothing.
    """

    if isinstance(value, dict):
        mapping = cast("dict[str, Any]", value)
        total = 0
        for key, item in mapping.items():
            if key in _NOT_PAYLOAD:
                continue
            if charge_keys:
                total += len(str(key).encode())
            total += _payload_bytes(item, str(key) in _DATA_KEYED_MAPS)
        return total
    if isinstance(value, (list, tuple)):
        return sum(_payload_bytes(item) for item in cast("list[Any]", value))
    if isinstance(value, str):
        return len(value.encode())
    return 8


def check_the_envelope_is_typed_and_its_facts_are_independent() -> None:
    """Lifecycle, coverage, and issues each answer a different question.

    A consumer that collapses them gets one of them wrong: an index can be complete and
    fresh while a watch runs beneath it, and partial while nothing runs at all. And an
    issue is a value to branch on -- deciding whether to retry, to prompt for access, or to
    drop a subtree is a decision about a kind, not about the wording of a message.
    """

    with tempfile.TemporaryDirectory(prefix="fdu-envelope-") as raw:
        root = Path(raw)
        (root / "a.txt").write_text("x", encoding="utf-8")
        index = fdu.open(root, cache=fdu.CachePolicy.OFF)

        settled = index.read()
        assert settled.status.phase is fdu.Phase.READY
        assert settled.status.complete is True
        assert settled.status.coverage_reason is None, "complete coverage names no reason"
        assert settled.status.errors == ()

        # A watch attaching is a lifecycle change and nothing else: the rows, their
        # coverage, and their freshness are all exactly what they were.
        before = index.cursor()
        with index.watch(fdu.WatchOptions(interval=0.1)):
            watching = index.read()
            assert watching.status.phase is fdu.Phase.WATCHING
            assert watching.status.complete is settled.status.complete
            assert watching.status.freshness is settled.status.freshness
            # And it reached the change feed, because it changed what a read answers.
            moved = index.since(before)
            assert any(change.transition is fdu.Transition.PHASE for change in moved.transitions), (
                moved.transitions
            )
            # Each transition keeps the commit it landed at. Stamping them with the range's
            # terminal position would say every one of them happened last, which is both
            # false and unorderable against the operations beside them.
            assert all(
                before.clock < change.clock <= moved.cursor.clock for change in moved.transitions
            ), (before, moved.cursor, moved.transitions)

        # Giving the watch back puts the phase back, so the state is a fact rather than a
        # latch: an index that once had a watch does not go on claiming one.
        assert index.read().status.phase is fdu.Phase.READY


def check_state_and_operations_interleave_at_their_own_clocks() -> None:
    """A transition carries the commit it landed at, so it can be placed among the rows.

    A refresh commits three things in order: the sweep announces itself, the rows it finds
    apply, and the sweep records what it verified. Flattened onto one terminal position all
    three would claim to have happened at the end -- and a consumer replaying them would
    apply the rows *before* being told the subtree was under reconciliation, which is the
    ordering the announcement exists to prevent.
    """

    with tempfile.TemporaryDirectory(prefix="fdu-interleave-") as raw:
        root = Path(raw)
        (root / "a.txt").write_text("x", encoding="utf-8")
        index = fdu.open(root, cache=fdu.CachePolicy.OFF)

        before = index.cursor()
        (root / "b.txt").write_text("yy", encoding="utf-8")
        index.refresh()

        moved = index.since(before)
        opened = [
            change
            for change in moved.transitions
            if change.transition is fdu.Transition.FRESHNESS
            and change.freshness is fdu.Freshness.RECONCILING
        ]
        verified = [
            change for change in moved.transitions if change.transition is fdu.Transition.VERIFIED
        ]
        assert opened and verified, moved.transitions
        assert moved.changes, "the new file applied in between"

        # Strictly ordered, and the rows sit between the two ends of the sweep.
        announced = opened[0].clock
        recorded = verified[0].clock
        applied = [change.clock for change in moved.changes]
        assert announced < min(applied) <= max(applied) < recorded, (
            f"announced at {announced}, rows at {applied}, verified at {recorded}"
        )


def check_a_batch_names_which_projections_went_stale() -> None:
    """A consumer holding one projection is told whether *it* moved, not just which paths.

    Absence is the guarantee here. A kind named may be stale; a kind missing is not -- so
    the assertions that matter are the negative ones, because those are what let a consumer
    keep an answer instead of re-reading it.
    """

    with tempfile.TemporaryDirectory(prefix="fdu-kinds-") as raw:
        root = Path(raw)
        (root / "seed.txt").write_text("seed", encoding="utf-8")
        index = fdu.open(root, cache=fdu.CachePolicy.OFF)

        with index.watch(fdu.WatchOptions(interval=0.2)) as watch:
            (root / "added.rs").write_text("fn main() {}", encoding="utf-8")
            # One loop, with the deadline checked on every iteration. A nested loop that
            # breaks only on a dirty batch never comes back to the deadline once the tree
            # goes quiet, so a run that does not see what it is waiting for hangs instead
            # of failing. Each iteration returns within `interval`, so this cannot.
            deadline = time.monotonic() + 30
            kinds: set[fdu.QueryKind] = set()
            for batch in watch:
                kinds.update(batch.dirty_queries)
                if fdu.QueryKind.RECENT in kinds or time.monotonic() > deadline:
                    break

        assert fdu.QueryKind.RECENT in kinds, f"a file arrived, so recency moved: {kinds}"
        assert fdu.QueryKind.CATALOG in kinds, kinds
        assert fdu.QueryKind.ROLLUP in kinds, kinds
        assert fdu.QueryKind.METADATA not in kinds, (
            f"identity facts are fixed for an opened index, so they are never named: {kinds}"
        )


def check_a_file_cap_stops_the_walk_rather_than_the_answer() -> None:
    """`max_files` is scope: entries past the cap are absent, not withheld.

    A projection limit would satisfy every soft assertion here -- fewer rows, partial
    coverage, a typed issue -- while still reading the whole tree, which is the cost the
    cap exists to avoid. So the load-bearing check is that the *index* holds less, and that
    a directory the walk never entered says its coverage is partial rather than reporting an
    empty directory that is not empty.
    """

    with tempfile.TemporaryDirectory(prefix="fdu-budget-") as raw:
        root = Path(raw)
        per_dir = 8
        dirs = 12
        for directory in range(dirs):
            child = root / f"d{directory:02d}"
            child.mkdir()
            for index in range(per_dir):
                (child / f"f{index}.txt").write_text("x", encoding="utf-8")

        full = fdu.open(root, cache=fdu.CachePolicy.OFF)
        full_scope = full.read().scope
        assert full.total().files == dirs * per_dir
        assert full.status.complete is True
        assert full_scope.max_files is None

        capped = fdu.open(
            root,
            cache=fdu.CachePolicy.OFF,
            scan=fdu.ScanOptions(max_files=2 * per_dir, threads=1),
        )
        assert capped.total().files < full.total().files, (
            "the cap has to leave entries out of the index, not out of one answer"
        )
        assert capped.status.complete is False
        assert capped.status.coverage_reason is fdu.CoverageReason.BUDGET
        assert any(issue.kind is fdu.IssueKind.RESOURCE_STOP for issue in capped.status.errors), (
            f"and say so as a typed value: {capped.status.errors}"
        )

        # Scope, so it is part of the identity a consumer keys a cache on.
        capped_scope = capped.read().scope
        assert capped_scope.max_files == 2 * per_dir
        assert capped_scope.hidden_fingerprint == full_scope.hidden_fingerprint, (
            "the cap must not be smuggled into another fingerprint"
        )
        assert capped_scope.tag_rules_fingerprint == full_scope.tag_rules_fingerprint
        assert capped_scope.type_rules_fingerprint == full_scope.type_rules_fingerprint

        # A directory the walk never entered still has a row -- it was seen as a child of a
        # directory that was read -- and must not report its unread zero as a complete one.
        unread = [
            name
            for name in (f"d{directory:02d}" for directory in range(dirs))
            if (rollup := capped.rollup(name)) is not None and rollup.files == 0
        ]
        assert unread, "the fixture must leave some directory unentered"
        for name in unread:
            provenance = capped.provenance(name)
            assert provenance is not None
            assert provenance.reason is fdu.CoverageReason.BUDGET, (
                f"{name} was never read, and says so: {provenance}"
            )

        # Zero is not how an unlimited walk is spelled, and one judge decides that.
        try:
            fdu.ScanOptions(max_files=0)
        except ValueError as error:
            assert "max_files" in str(error), error
        else:  # pragma: no cover - the guard above is the point
            raise AssertionError("a zero cap must be refused")


def check_a_batch_carries_the_state_at_its_own_cursor() -> None:
    """A batch says where the engine ended up, not only what moved on the way.

    `changes` says what moved and `transitions` says what shifted underneath them; neither
    says how far to trust what a consumer is now holding. Answering that with a follow-up
    read is a different instant -- the next commit can land between the two calls, and the
    index keeps only its current image, so there is nothing to ask for the state as of a
    position already passed. Folding the transitions into a consumer-side copy is the other
    way to get it wrong: two authorities for one fact, diverging the first time one is
    dropped, reordered or misapplied, with nothing able to detect it.

    The teeth here are the interleave. A commit lands *after* the batch and before the
    assertions, so an implementation that read the state at assertion time would report the
    later one -- and the two are deliberately made to differ.
    """

    with tempfile.TemporaryDirectory(prefix="fdu-terminal-") as raw:
        root = Path(raw)
        (root / "seed.txt").write_text("seed", encoding="utf-8")
        index = fdu.open(root, cache=fdu.CachePolicy.OFF)

        with index.watch(fdu.WatchOptions(interval=0.2)) as watch:
            (root / "added.rs").write_text("fn main() {}", encoding="utf-8")
            deadline = time.monotonic() + 30
            carried: fdu.WatchBatch | None = None
            for batch in watch:
                if batch.cursor is not None:
                    carried = batch
                    break
                if time.monotonic() > deadline:
                    break

        assert carried is not None, "a write should produce a batch that names a position"
        cursor = carried.cursor
        assert cursor is not None
        assert carried.state is not None, "a batch that names a position names its state"
        assert carried.state.freshness is fdu.Freshness.FRESH, carried.state
        assert carried.state.phase is fdu.Phase.WATCHING, (
            f"the batch came from a watch, so that is the phase it saw: {carried.state}"
        )

        # The commit a follow-up read would have seen and this batch must not. Closing the
        # watch is itself an answer-affecting transition, so the phase has already moved.
        assert index.read().status.phase is fdu.Phase.READY, "the index has moved on"
        assert carried.state.phase is fdu.Phase.WATCHING, (
            "and the batch still describes the instant it was taken"
        )
        assert cursor.clock <= index.cursor().clock, "which is at or behind where the index is"


def check_a_batch_reports_what_it_cost_and_not_what_it_waited() -> None:
    """A batch's wall time is its work, not the interval it sat in.

    The interval is how long a pull may block before returning empty-handed. Folding that
    into the cost would make an idle tree with a long interval report a minute of "work"
    for a batch that did nothing -- and the one figure an embedder compares providers on
    would be measuring its own patience instead of the engine.
    """

    with tempfile.TemporaryDirectory(prefix="fdu-batchwork-") as raw:
        root = Path(raw)
        (root / "seed.txt").write_text("seed", encoding="utf-8")
        index = fdu.open(root, cache=fdu.CachePolicy.OFF)

        # The change has to land *during* the wait, or there is no wait to confuse the work
        # with: writing the file first makes the pull return immediately and a naive
        # whole-call measurement would look correct. A timer puts it a second and a half
        # into a three-second interval.
        interval = 3.0
        delay = 1.5
        writer = threading.Timer(
            delay, lambda: (root / "added.rs").write_text("fn main() {}", encoding="utf-8")
        )
        writer.start()
        try:
            with index.watch(fdu.WatchOptions(interval=interval)) as watch:
                deadline = time.monotonic() + 30
                applied = None
                for batch in watch:
                    # The batch carrying the file, not merely the first dirty one. A
                    # backend that registers its watch after the root already exists
                    # escalates at startup, so on macOS the first dirty batch is that
                    # recovery -- at the root, whose relative path is empty and whose name
                    # bytes are therefore legitimately zero.
                    if any(change.path.name == "added.rs" for change in batch.changes):
                        applied = batch
                        break
                    if time.monotonic() > deadline:
                        break
        finally:
            writer.cancel()

        assert applied is not None, "the created file should have arrived"
        assert applied.work is not None
        assert applied.work.wall_ns < delay * 1_000_000_000 / 3, (
            "a batch reports what it did, not how long it waited to do it: "
            f"{applied.work.wall_ns / 1e9:.3f}s, having waited about {delay}s for the change"
        )
        assert applied.work.rows == len(applied.changes), applied.work
        assert applied.work.name_bytes >= len("added.rs"), (
            f"the row it carries costs at least its own name: {applied.work}"
        )


def check_a_pinned_assembly_pins_its_clock_too() -> None:
    """A version pin fixes the tree. It does not fix a relative recency cutoff.

    ``modified_since="1s"`` is resolved when the query is built, so page two of a
    version-pinned assembly resolved it against a later instant than page one. Membership
    moved while the version stood still, and nothing reported it -- the version genuinely
    had not moved. ``Query.as_of`` is the reference instant a caller carries across the
    pages of one answer.
    """

    with tempfile.TemporaryDirectory(prefix="fdu-asof-") as raw:
        root = Path(raw)
        recent = root / "recent.txt"
        recent.write_text("x", encoding="utf-8")
        index = fdu.open(root, cache=fdu.CachePolicy.OFF)

        # Half a second old, against a one-second window: inside it now, outside it shortly.
        now = datetime.now(tz=UTC)
        stamp = (now - timedelta(seconds=0.5)).timestamp()
        os.utime(recent, (stamp, stamp))
        index.refresh()

        def names(as_of: datetime | None) -> set[str]:
            report = index.report(
                fdu.Query(
                    views=(fdu.View.FILES,),
                    selection=fdu.Selection(modified_since="1s", kinds=(fdu.EntryKind.FILE,)),
                    as_of=as_of,
                )
            )
            section = report.sections[0]
            assert isinstance(section, fdu.FilesSection)
            return {row.path.name for row in section.files}

        pinned_at = datetime.now(tz=UTC)
        assert names(pinned_at) == {"recent.txt"}, "the file is inside the window to begin with"

        # The wall clock moves past the boundary. This is the whole scenario: a second page
        # of the same assembly, taken a moment later.
        time.sleep(1.0)

        assert names(None) == set(), (
            "without a pinned instant the cutoff follows the clock, which is correct for a "
            "one-shot and is exactly what breaks a paged assembly"
        )
        assert names(pinned_at) == {"recent.txt"}, (
            "a page pinned to the assembly's own instant must return what the first page "
            "did; a version pin alone does not do that"
        )

        # And the two pins compose, which is the actual scenario: a second page of a
        # version-pinned assembly, taken after the boundary, asking for the same answer.
        page_two = index.read(
            expected=index.cursor(),
            query=fdu.Query(
                views=(fdu.View.FILES,),
                selection=fdu.Selection(modified_since="1s", kinds=(fdu.EntryKind.FILE,)),
                as_of=pinned_at,
            ),
        )
        assert page_two.report is not None
        section = page_two.report.sections[0]
        assert isinstance(section, fdu.FilesSection)
        assert {row.path.name for row in section.files} == {"recent.txt"}, (
            "the version was pinned and so was the instant, so page two is page one"
        )


def check_a_state_transition_advances_the_version_and_reaches_the_feed() -> None:
    """Coverage, trust, and the run envelope move through commits like any other change.

    A transition delivered outside the clocked commit path lets one cursor name two
    different answers, and nothing in either says which was read. So a refresh that finds
    no difference at all still advances the version -- it verified the subtree, and what
    the rows may be trusted to mean is exactly what moved.
    """

    with tempfile.TemporaryDirectory(prefix="fdu-state-") as raw:
        root = Path(raw)
        (root / "src").mkdir()
        (root / "src" / "main.rs").write_text("fn main() {}", encoding="utf-8")
        index = fdu.open(root, cache=fdu.CachePolicy.OFF)

        before = index.cursor()
        index.refresh()

        after = index.cursor()
        assert after.clock > before.clock, (
            "a sweep that changed no row still changed how far its rows may be trusted"
        )
        changed = index.since(before)
        assert not changed.changes, f"nothing about a path moved: {changed.changes}"
        transitions = {change.transition for change in changed.transitions}
        assert fdu.Transition.VERIFIED in transitions, changed.transitions
        assert fdu.Transition.FRESHNESS in transitions, changed.transitions
        assert changed.cursor == after, "the feed ends where the index stands"

        # And every transition names where it applies, so a consumer can invalidate a
        # subtree rather than everything.
        verified = next(
            change for change in changed.transitions if change.transition is fdu.Transition.VERIFIED
        )
        assert verified.paths == (Path(""),), verified

        # The envelope moves with the rows rather than in a second write nobody is told
        # about. Its watermark is a fact about *this* run -- when the sweep began -- so a
        # repeated sweep over an unchanged tree records a different envelope and commits
        # again. That is the point: an answer's trust window moved, and a consumer holding
        # the previous one has to hear so. Only a genuinely identical envelope is skipped,
        # which the engine's own tests pin because nothing here can produce one.
        assert fdu.Transition.RUN_FACTS in transitions, changed.transitions
        index.refresh()
        repeated = {change.transition for change in index.since(after).transitions}
        assert repeated == transitions, (
            f"a repeated sweep is the same kind of transition: {repeated} vs {transitions}"
        )


def check_the_event_loop_adapter_delivers_the_same_batches() -> None:
    """An asyncio consumer gets the typed batches, without owning the thread handoff.

    The adapter changes when a batch arrives, not what it is. It also owns the affinity
    rule: the watch is created here and touched only by the worker thread, including
    being closed by it, so a caller never has to reason about which thread it is on.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-aio-"))
    (root / "seed.txt").write_text("seed", encoding="utf-8")
    index = fdu.open(root)

    async def collect() -> tuple[fdu.Change, ...]:
        seen: list[fdu.Change] = []

        async def touch() -> None:
            await asyncio.sleep(0.4)
            (root / "async.txt").write_text("hello", encoding="utf-8")

        task = asyncio.ensure_future(touch())
        try:
            async for batch in fdu.aio.watch_batches(index, fdu.WatchOptions(interval=0.1)):
                # Empty batches are filtered by the adapter, so anything yielded is real.
                assert batch.dirty or batch.changes, (
                    "the adapter must not forward batches that observed nothing"
                )
                seen.extend(batch.changes)
                if any(change.path == Path("async.txt") for change in seen):
                    break
        finally:
            await task
        return tuple(seen)

    changes = asyncio.run(asyncio.wait_for(collect(), timeout=30))
    created = next(change for change in changes if change.path == Path("async.txt"))
    assert created.kind is fdu.ChangeKind.UPSERT
    assert created.bytes == 5
    assert created.clock > 0

    # Leaving the loop early stopped the worker; the index is still usable afterwards,
    # which is what proves the adapter closed its own watch rather than the caller's.
    index.refresh()
    assert index.rollup(Path()) is not None

    # Cancellation must be prompt, must not stall the loop, and must actually end the
    # worker. Joining the worker *on* the loop thread deadlocks: its exit path is
    # `run_coroutine_threadsafe(queue.put(None), loop).result()`, so it needs the loop to
    # run while the join stops the loop from running. That resolves by timeout -- every
    # request stalled for seconds, and the worker still alive at the end.
    async def cancel_promptly() -> tuple[float, int, int]:
        beats = 0

        async def heartbeat() -> None:
            nonlocal beats
            while True:
                beats += 1
                await asyncio.sleep(0.01)

        async def follow() -> None:
            # A minute, deliberately. The interval is what the caller wants to hear from an
            # idle tree, and it used to become the native wait the worker parked in -- so
            # cancellation waited out the join timeout and then reported success over a
            # worker that was still inside that wait. An `interval=0.1` test cannot see
            # that at all, because the two bounds are indistinguishable when both are
            # small.
            async for _ in fdu.aio.watch_batches(index, fdu.WatchOptions(interval=60.0)):
                pass

        def watchers() -> int:
            # By name, not by total count: the teardown hands its join to an executor,
            # whose pool thread is expected to outlive the call and would otherwise read
            # as the leak this is looking for.
            return sum(1 for t in threading.enumerate() if t.name == "fdu-watch-aio")

        pulse = asyncio.ensure_future(heartbeat())
        watching = asyncio.ensure_future(follow())
        # Let the worker start and settle into its pull, so the teardown below is the
        # interesting one rather than a race with startup.
        await asyncio.sleep(0.3)

        beats_before = beats
        started = time.monotonic()
        watching.cancel()
        with contextlib.suppress(asyncio.CancelledError):
            await watching
        elapsed = time.monotonic() - started

        # Let an exiting thread finish unwinding before counting it.
        await asyncio.sleep(0.1)
        pulse.cancel()
        return elapsed, beats - beats_before, watchers()

    elapsed, beats, leaked = asyncio.run(asyncio.wait_for(cancel_promptly(), timeout=30))
    assert elapsed < 3.0, f"cancellation must not stall the loop: {elapsed:.2f}s"
    assert beats > 0, "the loop must keep running its other tasks while the watch tears down"
    assert leaked == 0, f"the worker thread must be gone, not merely told to stop: {leaked}"
    # Teardown returned normally, which is a claim: `Thread.join` cannot tell "stopped"
    # from "gave up waiting", so a clean return over a live worker was representable and
    # `WatchTeardownError` is what makes it not.
    assert issubclass(fdu.WatchTeardownError, fdu.FduError)

    fdu.clear_cache(root)


def check_the_sse_example_resumes_or_resyncs() -> None:
    """The shipped SSE example makes the one decision that fails silently if wrong.

    A truncated `ChangeSet` carries real but incomplete changes. Replaying them produces
    a client that believes it is current while missing everything the journal evicted, and
    nothing raises. The example is loaded from the file that ships, so the tested code and
    the documented code are the same code.
    """

    example_path = Path(__file__).resolve().parent.parent / "examples" / "sse_resume.py"
    spec = importlib.util.spec_from_file_location("fdu_sse_example", example_path)
    assert spec is not None and spec.loader is not None
    example = importlib.util.module_from_spec(spec)
    # Registered before execution: `@dataclass(slots=True)` rebuilds the class and looks
    # its module up in `sys.modules` to do it, so an unregistered module fails to load.
    sys.modules[spec.name] = example
    spec.loader.exec_module(example)

    change = fdu.Change(clock=7, path=Path("a.txt"), kind=fdu.ChangeKind.UPSERT)
    at_seven = fdu.Cursor(session=42, clock=7)
    at_nine = fdu.Cursor(session=42, clock=9)

    complete = fdu.ChangeSet(truncated=False, cursor=at_seven, changes=(change,))
    replayed = example.decide(complete, current=at_nine)
    assert replayed.resync is False
    assert replayed.changes == (change,)
    assert replayed.cursor == at_seven

    behind = fdu.ChangeSet(truncated=True, cursor=at_seven, changes=(change,))
    resync = example.decide(behind, current=at_nine)
    assert resync.resync is True
    assert resync.changes == (), "a truncated set must never be replayed"
    assert resync.cursor == at_nine, "and the client is told where it now is"

    # A client-supplied header is input, not a promise. The session half is why a token
    # minted by a previous process cannot pass as a position in this one.
    assert example.parse_last_event_id(None) is None
    assert example.parse_last_event_id("not-a-cursor") is None
    assert example.parse_last_event_id("42--1") is None
    assert example.parse_last_event_id("0-12") is None, "session 0 names no index"
    assert example.parse_last_event_id("42-12") == fdu.Cursor(session=42, clock=12)

    frame = example.sse_event("change", {"path": "a.txt"}, fdu.Cursor(session=42, clock=12))
    assert frame.startswith("id: 42-12\nevent: change\ndata: ")
    assert frame.endswith("\n\n"), "an SSE frame ends with a blank line"


def check_polling_is_selectable_for_filesystems_that_drop_events() -> None:
    """A caller on a network or FUSE mount can ask for polling instead.

    Those filesystems accept a native watch and then deliver nothing -- no error, and an
    index that quietly stops tracking. Only the source of raw events changes: the same
    coalescing and the same stat verification produce the same ops, more slowly.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-poll-"))
    (root / "seed.txt").write_text("seed", encoding="utf-8")
    index = fdu.open(root)

    with index.watch(fdu.WatchOptions(interval=0.1, poll_interval=0.1)) as watch:
        time.sleep(0.3)
        (root / "polled.txt").write_text("hello", encoding="utf-8")
        deadline = time.monotonic() + 30
        seen = None
        for batch in watch:
            for change in batch.changes:
                if change.path == Path("polled.txt"):
                    seen = change
            if seen is not None or time.monotonic() > deadline:
                break
        assert seen is not None, "the poll backend must report the created file"
        assert seen.kind is fdu.ChangeKind.UPSERT
        assert seen.bytes == 5, "a polled event is still verified by stat"

    # A non-positive interval is a busy restat loop, not a faster watch.
    try:
        fdu.WatchOptions(interval=0.1, poll_interval=0)
    except ValueError as error:
        assert "poll_interval" in str(error), error
    else:
        raise AssertionError("a zero poll_interval must be rejected")
    fdu.clear_cache(root)


def check_a_listing_carries_its_own_identity() -> None:
    """A listing row says what it is, so a consumer can drop its own classifier.

    Metadata only: nothing is opened, and the answer is the engine's own -- the same
    verdict the type and group views aggregate. Re-deriving it per row afterwards means
    answering in a second language, against a rule set with no way to stay in step.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-identity-"))
    (root / "src").mkdir()
    (root / "src" / "main.rs").write_text("fn main() {}", encoding="utf-8")
    (root / "notes.md").write_text("# hi", encoding="utf-8")
    (root / "bundle.tar.gz").write_bytes(b"\x1f\x8b" + b"0" * 30)
    (root / "Makefile").write_text("all:\n\ttrue\n", encoding="utf-8")

    index = fdu.open(root)
    listing = index.children()
    assert listing is not None
    children = {child.name: child for child in listing.rows}

    notes = children["notes.md"]
    assert notes.classification is not None
    assert notes.classification.file_type == "markdown"
    assert notes.classification.family is fdu.ContentFamily.PROSE
    assert notes.classification.group == "docs"
    assert notes.classification.source is fdu.DetectionSource.EXTENSION
    assert notes.extension == ".md"

    # A compound extension folds, and the row's key is the one its parent files it under.
    bundle = children["bundle.tar.gz"]
    assert bundle.extension == ".tar.gz"
    assert bundle.classification is not None
    assert bundle.classification.group == "archives"
    assert bundle.extension in index.total().by_extension

    # An exact-filename rule: no extension at all, and still a full verdict.
    make = children["Makefile"]
    assert make.extension is None
    assert make.classification is not None
    assert make.classification.file_type == "make"
    assert make.classification.source is fdu.DetectionSource.EXACT_FILENAME

    # A directory is not a file and has no identity to report.
    assert children["src"].classification is None

    # Files-view rows carry the same verdict, filled after the view's bound.
    report = index.report(
        fdu.Query(views=(fdu.View.FILES,), selection=fdu.Selection(kinds=(fdu.EntryKind.FILE,)))
    )
    rows = next(s for s in report.sections if isinstance(s, fdu.FilesSection)).files
    by_name = {row.path.name: row for row in rows}
    assert by_name["main.rs"].classification is not None
    assert by_name["main.rs"].classification.file_type == "rust"
    assert by_name["main.rs"].classification.group == "code"
    assert by_name["main.rs"].extension == ".rs"
    fdu.clear_cache(root)


def check_groups_answer_the_browsing_question() -> None:
    """A group axis a family axis cannot answer.

    `family` says which analyzer may open a file, so a PDF, an image, and a zip are all
    `binary` under it -- one row, over a directory whose whole point is that they differ.
    Groups answer where a reader would look instead, and the two are maintained side by
    side rather than one derived from the other.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-groups-"))
    (root / "main.rs").write_text("fn main() {}", encoding="utf-8")
    (root / "README.md").write_text("# hi", encoding="utf-8")
    (root / "data.json").write_text("{}", encoding="utf-8")
    (root / "photo.png").write_bytes(b"\x89PNG\r\n\x1a\n" + b"0" * 24)
    # An exact-filename rule: no extension to derive a group from, which is why the
    # engine maintains the tally rather than deriving it.
    (root / "Makefile").write_text("all:\n\ttrue\n", encoding="utf-8")

    index = fdu.open(root)
    total = index.total()
    assert {group: tally.files for group, tally in total.by_group.items()} == {
        "code": 2,
        "docs": 1,
        "data": 1,
        "media": 1,
    }, dict(total.by_group)
    assert sum(tally.files for tally in total.by_group.values()) == total.files

    report = index.report(fdu.Query(views=(fdu.View.GROUPS,)))
    section = next(s for s in report.sections if isinstance(s, fdu.GroupsSection))
    assert {row.id for row in section.groups} == {"code", "docs", "data", "media"}
    assert {row.label for row in section.groups} >= {"Code", "Documentation", "Media"}
    assert sum(row.files for row in section.groups) == total.files

    # A filtered groups view comes from the walk rather than the maintained state, and
    # the two must agree about what they both cover.
    filtered = index.report(
        fdu.Query(views=(fdu.View.GROUPS,), selection=fdu.Selection(include=("*.rs", "*.md")))
    )
    rows = next(s for s in filtered.sections if isinstance(s, fdu.GroupsSection)).groups
    assert {row.id: row.files for row in rows} == {"code": 1, "docs": 1}, rows
    fdu.clear_cache(root)


def check_supplied_type_rules_reach_the_answer() -> None:
    """A caller's own taxonomy classifies, and is a different cache identity.

    The point of the registry: a consumer whose file-type vocabulary differs from fdu's
    supplies it rather than rebuilding the crate or reclassifying in Python. The second
    assertion is the one that would fail silently -- entry counts and byte totals are the
    same under either taxonomy, so a snapshot reused across a rule change looks correct.
    """

    rules = fdu.TypeRegistry.from_manifest(
        '[[kind]]\nid = "notes"\nfamily = "prose"\nextensions = ["rs"]\n'
    )
    assert rules.rule_count == 1
    assert rules.type_ids() == ("notes",)

    mine = rules.classify("main.rs")
    assert mine.file_type == "notes"
    assert mine.family is fdu.ContentFamily.PROSE
    assert mine.source is fdu.DetectionSource.EXTENSION
    assert mine.confidence is fdu.DetectionConfidence.CERTAIN

    shipped = fdu.TypeRegistry.compiled()
    assert shipped.classify("main.rs").file_type == "rust"
    assert shipped.rule_count > rules.rule_count
    assert shipped.fingerprint != rules.fingerprint, (
        "different rules must be a different cache identity"
    )

    # A manifest that would classify ambiguously is rejected, with the parser's message.
    try:
        fdu.TypeRegistry.from_manifest(
            '[[kind]]\nid = "a"\nfamily = "code"\n[[kind]]\nid = "a"\nfamily = "code"\n'
        )
    except fdu.InvalidArgumentError as error:
        assert "duplicate rule id" in str(error), error
        assert "invalid type rules" in str(error), error
    else:
        raise AssertionError("a duplicate id must be rejected")

    # And the rules reach a real scan.
    root = Path(tempfile.mkdtemp(prefix="fdu-type-rules-"))
    (root / "main.rs").write_text("fn main() {}", encoding="utf-8")
    index = fdu.scan(root, scan=fdu.ScanOptions(type_rules=rules))
    report = index.report(fdu.Query(views=(fdu.View.TYPES,)))
    labels = {
        row.id
        for section in report.sections
        if isinstance(section, fdu.MetricsSection)
        for row in section.rows
    }
    assert labels == {"notes"}, labels


def check_bounded_extension_rows_account_for_the_rest() -> None:
    """A listing can ask for a handful of extension rows and still be told the total.

    The same contract as a tree node's remainder, one level down: what the bound keeps
    plus what it reports withheld is what the unbounded call returns. Bounding in Rust is
    the point -- a wide directory multiplies its child count by every child's distinct
    extensions, and a browser showing five rows should not pay to marshal five hundred.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-ext-bound-"))
    sizes = {".rs": 500, ".md": 300, ".txt": 100, ".toml": 10}
    for suffix, size in sizes.items():
        (root / f"file{suffix}").write_text("x" * size, encoding="utf-8")

    index = fdu.open(root)
    everything = index.total()
    assert set(everything.by_extension) == set(sizes)
    assert everything.extension_remainder is None, "an unbounded roll-up withheld nothing"

    bounded = index.total(extensions=2)
    assert set(bounded.by_extension) == {".rs", ".md"}, "the two largest by bytes"
    withheld = bounded.extension_remainder
    assert withheld is not None
    assert withheld.extensions == 2
    assert withheld.files == 2
    kept = sum(tally.bytes for tally in bounded.by_extension.values())
    assert kept + withheld.bytes == sum(tally.bytes for tally in everything.by_extension.values())

    # A bound at or above what is present is not a truncation.
    assert index.total(extensions=len(sizes)).extension_remainder is None

    # And it reaches a listing, which is where it earns its keep: every child's
    # breakdown is bounded, not just the root's.
    nested = root / "sub"
    nested.mkdir()
    for suffix, size in sizes.items():
        (nested / f"file{suffix}").write_text("x" * size, encoding="utf-8")
    index.refresh()

    listing = index.children()
    assert listing is not None
    child = next(item for item in listing.rows if item.name == "sub")
    assert child.totals is not None and child.totals.files == len(sizes)

    # The row does not carry the breakdown at all -- that is the point of the split -- so
    # the bound belongs to the projection that does.
    bounded_child = index.rollup("sub", extensions=1)
    assert bounded_child is not None
    assert len(bounded_child.by_extension) == 1
    assert bounded_child.extension_remainder is not None
    assert bounded_child.extension_remainder.extensions == 3
    fdu.clear_cache(root)


def check_telemetry_measures_the_run_not_the_tree() -> None:
    """Each call reports its own cost, and the numbers are the walk's, not a total.

    An embedder timing its own loop attributes cost to the call it just made. The two
    properties that makes true are that a scan's counts match the tree it read, and that
    a refresh replaces them rather than adding to them -- so a server refreshing on every
    change reads a per-refresh cost instead of a sum that only grows.
    """

    own = Path(tempfile.mkdtemp(prefix="fdu-telemetry-"))
    (own / "src").mkdir()
    (own / "src" / "a.txt").write_text("aaaa", encoding="utf-8")
    (own / "b.txt").write_text("bb", encoding="utf-8")

    index = fdu.scan(own)
    walk = index.telemetry
    assert walk.source is fdu.ReportSource.COLD_SCAN, walk.source
    assert walk.walked_files == 2, walk.walked_files
    assert walk.walked_bytes == 6, walk.walked_bytes
    # No analysis was requested, so nothing was read and nothing was restored.
    assert (walk.fresh_files, walk.bytes_read, walk.cached_files) == (0, 0, 0)
    assert walk.analysis_seconds == walk.analysis_ns / 1e9

    (own / "src" / "c.txt").write_text("ccc", encoding="utf-8")
    index.refresh()
    after = index.telemetry
    assert after.source is fdu.ReportSource.WARM_REVALIDATE, after.source
    # Three files now, and the count is the refresh's own -- not five, which is what a
    # running total across both calls would report.
    assert after.walked_files == 3, after.walked_files
    assert after.walked_bytes == 9, after.walked_bytes

    scoped = fdu.open(own)
    scoped.refresh("src")
    # A scoped refresh reads the subtree, so its telemetry is smaller than the whole
    # tree's. This is the evidence that scoping cost anything at all.
    assert scoped.telemetry.walked_files == 2, scoped.telemetry.walked_files
    fdu.clear_cache(own)


def check_the_one_shot_retains_nothing(root: Path) -> None:
    """`fdu.report` runs the contract the command line runs, not a session.

    `open` retains an index and writes a snapshot, which is right for a caller asking many
    questions and wrong for one asking a single question -- an unfiltered summary is
    answered by a transient tier that retains nothing, so a session cached state the walk
    never saved and a later cache-only read could see it (fdu-4msv).
    """

    report = fdu.report(root, fdu.Query(views=(fdu.View.SUMMARY,)))
    assert report.status.source is not None

    # Rendering twice must not cost a second walk: the handle owns the finished report.
    text = report.render(fdu.Format.TEXT)
    assert text == report.render(fdu.Format.TEXT)
    assert report.render(fdu.Format.JSON) != text

    # An unusable cache is the operation failing, not the caller asking wrongly. Calling it
    # an argument error sent a caller looking in the wrong place, and made the CLI shim
    # exit 2 as a usage error where the command line exits 1.
    try:
        fdu.report(root, fdu.Query(views=(fdu.View.SUMMARY,)), cache=fdu.CachePolicy.ONLY)
    except fdu.InvalidArgumentError as error:  # pragma: no cover - the regression
        raise AssertionError(f"an unusable snapshot is not an argument error: {error}") from None
    except fdu.FduError:
        pass


def check_the_list_grammar_reaches_python(root: Path) -> None:
    """A view spec is parsed by the one grammar, not by whichever surface got it first.

    Duplicate and empty-entry rejection lived in the CLI, so `views="tree,tree"` was a
    typo there and a silent no-op here -- one request meaning two things depending on
    which door it came through.
    """

    index = fdu.scan(root)
    for spec, expected in [
        ("tree,tree", "appears more than once"),
        ("tree,,types", "empty entry in the list"),
        ("full,tree", "cannot be combined"),
        ("bogus", "expected one of"),
    ]:
        try:
            index.report(fdu.Query(views=spec))
        except fdu.InvalidArgumentError as error:
            assert expected in str(error), (spec, str(error))
        else:
            raise AssertionError(f"{spec!r} must be rejected")

    # And a spec the grammar accepts still works, including `full` expansion.
    assert len(index.report(fdu.Query(views="full")).sections) > 1
    assert len(index.report(fdu.Query(views="tree,types")).sections) == 2


def check_render_matches_the_cli(root: Path, binary: str) -> None:
    """The package renders what the command line prints.

    Until this existed a Python caller wanting fdu's own output had to shell out to the
    binary -- the same admission the console script makes, since `fdu:_main` calls
    `_native.main()` and the `fdu` the wheel installs has never exercised a line of the
    Python API.

    The comparison is against the real CLI rather than a recorded string, because a
    recording drifts and the point is that the two agree today.
    """

    index = fdu.scan(str(root))
    for view in (fdu.View.TREE, fdu.View.LARGEST, fdu.View.SUMMARY):
        report = index.report(fdu.Query(views=(view,)))
        for fmt in fdu.Format:
            rendered = report.render(fmt)
            cli = subprocess.run(
                [
                    binary,
                    "--cache",
                    "off",
                    "--color",
                    "never",
                    "--format",
                    str(fmt),
                    "--view",
                    str(view),
                    str(root),
                ],
                capture_output=True,
                text=True,
                check=True,
            ).stdout
            # The CLI appends a performance footer; the schema excludes that telemetry and
            # a Report does not carry the counts behind it, so it is the one difference.
            body = "\n".join(
                line for line in cli.splitlines() if not line.startswith("Performance:")
            ).rstrip()
            # Two separate runs, so the walk timestamps differ. Normalised the same way
            # the golden corpus masks them: the values are unstable, the shape is not.
            assert _stable(rendered.rstrip()) == _stable(body), (
                view,
                fmt,
                rendered[:200],
                body[:200],
            )

    # A report built by hand has no index to render through, and says so.
    from dataclasses import replace as _replace

    detached = _replace(index.report(fdu.Query()), _renderer=None)
    try:
        detached.render()
        raise SystemExit("a report with no index behind it must refuse to render")
    except fdu.InvalidArgumentError as error:
        # Every producer that does bind one, so the message does not send a caller who
        # used fdu.report or Watch.report looking at the wrong call.
        for producer in ("Index.report", "fdu.report", "Watch.report"):
            assert producer in str(error), (producer, str(error))


def check_a_report_is_a_snapshot(root: Path) -> None:
    """One report must not answer differently each time it is asked.

    `Index.report` used to bind a renderer to the *query* and re-project the retained index
    per format, so `as_dict` held the values the call was answered with while `render`
    quietly returned newer ones once the index moved (fdu-4gno). All three producers now
    bind to the finished report.
    """

    index = fdu.open(root)
    report = index.report(fdu.Query(views=(fdu.View.SUMMARY,)))
    before = report.render(fdu.Format.JSON)

    (root / "snapshot-probe.bin").write_bytes(b"x" * 100_000)
    index.refresh()

    assert report.render(fdu.Format.JSON) == before, "render must not follow the live index"
    assert json.loads(before)["reports"][0] == report.as_dict()["reports"][0], (
        "as_dict and render must serialize one value, not two"
    )

    # And the index really did move, or this proves nothing.
    assert index.report(fdu.Query(views=(fdu.View.SUMMARY,))).render(fdu.Format.JSON) != before
    (root / "snapshot-probe.bin").unlink()
    index.refresh()


def check_a_report_states_its_own_omissions(root: Path) -> None:
    """A dropped view must be readable as a value, not only inside rendered text.

    `full` without analyzers cannot answer `documents`. The report says so, and a caller
    reading `sections` needs that as a note rather than having to scrape the text rendering
    to find out why a section is absent (fdu-7wd1).
    """

    report = fdu.report(root, fdu.Query(views=(fdu.View.FULL,)))
    assert report.notes, "a dropped view must be stated on the report"
    assert any("documents" in note for note in report.notes), report.notes

    # Named in this surface's vocabulary: there is no --analyze in Python (fdu-4apt).
    for note in report.notes:
        assert "--analyze" not in note, note
        assert "--view" not in note, note
    assert any("add analyze " in note for note in report.notes), report.notes

    # The same rule as a hard error, in the same vocabulary.
    try:
        fdu.report(root, fdu.Query(views=(fdu.View.DOCUMENTS,)))
        raise SystemExit("documents without analyzers must be rejected")
    except fdu.InvalidArgumentError as error:
        assert str(error).startswith("view documents"), error
        assert "add analyze " in str(error), error
        assert "--analyze" not in str(error), error

    # Nothing dropped, nothing said.
    assert not fdu.report(root, fdu.Query(views=(fdu.View.SUMMARY,))).notes


def check_every_failure_is_an_fdu_error(root: Path) -> None:
    """`except FduError` must be enough to catch what this package raises.

    `Change.render` reached the extension directly and so raised pyo3's bare `ValueError`,
    which that clause does not catch (fdu-dygl).
    """

    change = fdu.Change(path=root / "x", kind=fdu.ChangeKind.UPSERT, clock=1)
    assert json.loads(change.render(fdu.Format.JSONL))["op"] == "upsert"
    try:
        change.render("xml")  # pyright: ignore[reportArgumentType]
        raise SystemExit("an unknown format must be rejected")
    except fdu.FduError as error:
        assert isinstance(error, fdu.InvalidArgumentError), type(error)


def check_the_watch_rule_names_an_instant(root: Path) -> None:
    """A repaint separator must render the instant it was given, exactly.

    A naive datetime names no instant; reading it as local time moved the rule by the
    machine's UTC offset while still printing `Z`. And nanoseconds cannot survive a float,
    so `Change.mtime_ns` -- which is what a caller repainting after a batch actually holds
    -- goes in as an int (fdu-uwv0).
    """

    del root
    aware = datetime(2026, 8, 10, 18, 22, 31, tzinfo=UTC)
    assert fdu.watch_rule(aware) == "──── 2026-08-10T18:22:31.000000000Z ────"

    # An int carries the full nanosecond; a float cannot, and quantized this to ...457024.
    exact_ns = 1_786_386_151_123_456_789
    assert fdu.watch_rule(exact_ns) == "──── 2026-08-10T18:22:31.123456789Z ────"

    try:
        fdu.watch_rule(datetime(2026, 8, 10, 18, 22, 31))
        raise SystemExit("a naive datetime must be refused, not read as local time")
    except fdu.InvalidArgumentError as error:
        assert "aware" in str(error), error


def check_every_view(root: Path) -> None:
    """Every view must reach the typed surface, and say what it bounded.

    This exists because the CLI and the binding held separate view vocabularies: `largest`
    and `recent` shipped, the CLI accepted them, the binding rejected them, and
    `make check` stayed green throughout because nothing in the Python tests named a new
    view. The loop is over `fdu.View` rather than a written list, so a view added later
    cannot be left out of it.
    """

    index = fdu.scan(str(root))
    analyzed = fdu.scan(str(root), analysis=fdu.AnalysisOptions(analyze=fdu.Analysis.ALL))
    for view in fdu.View:
        if view is fdu.View.FULL:
            continue
        # `documents` has no metadata-only projection, so it needs the analysed index.
        source = analyzed if view is fdu.View.DOCUMENTS else index
        report = source.report(fdu.Query(views=(view,)))
        assert len(report.sections) == 1, (view, report.sections)
        assert report.sections[0].view is view, (view, report.sections[0].view)

    # A bounded section reports what it dropped; an unbounded one reports nothing.
    bounded = index.report(fdu.Query(views=(fdu.View.FILES,), selection=fdu.Selection(limit=1)))
    section = bounded.sections[0]
    assert section.bound is not None, "a bounded section must say so"
    assert section.bound.shown == 1, section.bound
    assert section.bound.total > 1, section.bound

    complete = index.report(fdu.Query(views=(fdu.View.FILES,)))
    assert complete.sections[0].bound is None, "an unbounded section reports no bound"

    # Naming no view derives one from the analyzers, exactly as the command line does.
    # Python defaulted to `tree` regardless, so a caller who asked to read every file got
    # a directory tree containing none of the results -- the defect the content axis
    # removed from the CLI, still live here because nothing tested the two together.
    for analyze, expected in (
        (fdu.Analysis.NONE, fdu.View.TREE),
        (fdu.Analysis.LINES, fdu.View.FAMILIES),
        (fdu.Analysis.CODE, fdu.View.LANGUAGES),
        (fdu.Analysis.WORDS, fdu.View.DOCUMENTS),
        (fdu.Analysis.ALL, fdu.View.FAMILIES),
    ):
        derived = fdu.scan(str(root), analysis=fdu.AnalysisOptions(analyze=analyze))
        section = derived.report(fdu.Query()).sections[0]
        assert section.view is expected, (analyze, section.view, expected)

    # `full` is a total the enum offers, so the binding must honour it: it once listed
    # `full` as valid in its own error message while rejecting it.
    full = index.report(fdu.Query(views=(fdu.View.FULL,)))
    produced = {section.view for section in full.sections}
    assert fdu.View.LARGEST in produced and fdu.View.RECENT in produced, produced
    assert fdu.View.FILES not in produced, "an unbounded enumeration is not a summary"


def main() -> None:
    root = Path(tempfile.mkdtemp(prefix="fdu-public-api-"))
    (root / "src").mkdir()
    (root / "src" / "main.rs").write_text("fn main() {}", encoding="utf-8")
    (root / "notes.md").write_text("release notes", encoding="utf-8")

    check_every_view(root)
    check_a_report_is_a_snapshot(root)
    check_a_report_states_its_own_omissions(root)
    check_every_failure_is_an_fdu_error(root)
    check_the_watch_rule_names_an_instant(root)
    check_the_list_grammar_reaches_python(root)
    check_the_one_shot_retains_nothing(root)
    check_watch_reports_its_own_index(root)
    check_the_dirty_set_names_every_moved_rollup()
    check_telemetry_measures_the_run_not_the_tree()
    check_bounded_extension_rows_account_for_the_rest()
    check_supplied_type_rules_reach_the_answer()
    check_groups_answer_the_browsing_question()
    check_a_listing_carries_its_own_identity()
    check_polling_is_selectable_for_filesystems_that_drop_events()
    check_one_bundle_answers_a_whole_page()
    check_a_listing_pages_and_accounts_for_the_rest()
    check_partial_coverage_says_why()
    check_tags_are_a_named_fact_per_entry()
    check_a_promoted_plane_serves_a_dual_value_listing_from_one_call()
    check_pruned_hidden_paths_are_absent_rather_than_filtered()
    check_the_browser_provider_example_holds_the_contract_it_documents()
    check_a_bundle_answers_a_query_at_the_same_instant_as_its_rows()
    check_empty_is_decidable_from_the_aggregate()
    check_the_envelope_is_typed_and_its_facts_are_independent()
    check_state_and_operations_interleave_at_their_own_clocks()
    check_a_batch_carries_the_state_at_its_own_cursor()
    check_a_file_cap_stops_the_walk_rather_than_the_answer()
    check_a_batch_names_which_projections_went_stale()
    check_a_batch_reports_what_it_cost_and_not_what_it_waited()
    check_a_pinned_assembly_pins_its_clock_too()
    check_a_state_transition_advances_the_version_and_reaches_the_feed()
    check_the_event_loop_adapter_delivers_the_same_batches()
    check_the_sse_example_resumes_or_resyncs()
    check_a_bounded_tree_says_what_it_withheld()
    check_render_matches_the_cli(
        root, str(Path(sys.executable).with_name("fdu.exe" if os.name == "nt" else "fdu"))
    )

    index = fdu.scan(root, scan=fdu.ScanOptions(max_depth=3))
    assert os.path.samefile(index.root, root)
    assert index.status.complete is True
    assert index.status.freshness is fdu.Freshness.FRESH
    assert not index.status.errors

    try:
        fdu.scan(root / "missing")
    except fdu.FilesystemError as error:
        assert isinstance(error, OSError)
        assert isinstance(error, fdu.FduError)
        assert Path(error.filename).name == "missing"
    else:
        raise AssertionError("missing roots must raise fdu.FilesystemError")

    total = index.total()
    assert total.files == 2
    assert total.bytes == 25
    assert total.by_extension[".rs"].files == 1

    children = index.children()
    assert children is not None
    assert {child.name for child in children.rows} == {"notes.md", "src"}
    assert index.rollup("src") is not None
    assert index.rollup("missing") is None

    report = index.report(
        fdu.Query(
            views=(fdu.View.SUMMARY, fdu.View.EXTENSIONS, fdu.View.FILES),
            selection=fdu.Selection(size=fdu.SizeMetric.APPARENT),
        )
    )
    assert report.status.complete is True
    assert report.status.freshness is fdu.Freshness.FRESH
    assert [section.view for section in report.sections] == [
        fdu.View.SUMMARY,
        fdu.View.EXTENSIONS,
        fdu.View.FILES,
    ]
    wire = report.as_dict()
    assert wire["schema"] == "fdu.report/4"
    assert wire["generator"] == f"fdu {fdu.__version__}"
    assert json.loads(json.dumps(wire)) == wire

    # Every check in this file is called by name from `main`, which is the readable
    # arrangement and the one with a silent failure mode: a check that is written and never
    # listed passes forever, and looks exactly like a check that passes. This file's own
    # source is the only place that can tell the difference.
    own_source = Path(__file__).read_text(encoding="utf-8")
    tree = ast.parse(own_source)
    written = {
        node.name
        for node in tree.body
        if isinstance(node, ast.FunctionDef) and node.name.startswith("check_")
    }
    listed = {
        call.func.id
        for call in ast.walk(tree)
        if isinstance(call, ast.Call)
        and isinstance(call.func, ast.Name)
        and call.func.id.startswith("check_")
    }
    assert written <= listed, f"written but never run: {sorted(written - listed)}"

    package_dir = Path(fdu.__file__).parent
    public_names = {name for name in dir(fdu) if not name.startswith("_") or name == "__version__"}
    assert public_names == set(fdu.__all__), (public_names, set(fdu.__all__))
    assert (package_dir / "py.typed").is_file()
    stub_path = package_dir / "_native.pyi"
    assert stub_path.is_file()
    stub_tree = ast.parse(stub_path.read_text(encoding="utf-8"))
    stub_exports = {
        node.name for node in stub_tree.body if isinstance(node, (ast.ClassDef, ast.FunctionDef))
    }
    stub_exports.add("__version__")
    runtime_exports = {name for name in dir(_native) if not name.startswith("__")}
    runtime_exports.add("__version__")
    assert runtime_exports == stub_exports, (runtime_exports, stub_exports)
    assert _native.Index.__module__ == "fdu._native"
    assert _native.Watch.__module__ == "fdu._native"
    assert importlib.util.find_spec("fdu_py") is None
    contract = _native.contract()
    assert contract["cache_policies"] == [value.value for value in fdu.CachePolicy]
    assert contract["analysis"] == [value.value for value in fdu.Analysis]
    assert contract["views"] == [value.value for value in fdu.View]
    assert contract["entry_kinds"] == [value.value for value in fdu.EntryKind]
    assert contract["size_metrics"] == [value.value for value in fdu.SizeMetric]
    assert contract["sort_keys"] == [value.value for value in fdu.SortKey]
    assert contract["cache_scopes"] == [value.value for value in fdu.CacheScope]
    assert contract["formats"] == [value.value for value in fdu.Format]

    provenance = index.provenance("src")
    assert provenance is not None
    assert provenance.status is fdu.Coverage.COMPLETE
    assert provenance.source is fdu.ValueSource.SCANNED

    mark = index.cursor()
    (root / "new.txt").write_text("new", encoding="utf-8")
    refresh = index.refresh()
    assert refresh.inserted == 1
    assert refresh.status.complete is True
    changes = index.since(mark)
    assert changes.truncated is False
    assert any(change.path == Path("new.txt") for change in changes.changes)

    # A cursor is a position *in one opened index*. Held across a restart, or taken from
    # another root, it names nothing here -- and saying so is the whole point: an empty
    # change set would read as "you are current" to a consumer that had missed everything.
    assert changes.cursor.session == mark.session
    assert changes.cursor.clock >= mark.clock
    foreign = fdu.Cursor(session=mark.session ^ 0xFFFF, clock=mark.clock)
    try:
        index.since(foreign)
    except fdu.FduError:
        pass
    else:
        raise AssertionError("a cursor from another session must be refused")

    # A scoped refresh sees a change inside its subtree and reaches the same state a
    # whole-tree refresh would. This is the hint-ingestion primitive: a caller running
    # its own watcher pushes each hint through here rather than through a second path
    # into the index, and it pays for the subtree rather than the tree.
    (root / "src" / "scoped.txt").write_text("scoped", encoding="utf-8")
    scoped = index.refresh("src")
    assert scoped.inserted == 1, "a scoped refresh must observe a change inside its scope"
    assert index.rollup("src") is not None
    scoped_total = index.total().files
    # The same tree refreshed whole must agree, so scoping changed the cost and not the
    # answer.
    assert index.refresh().inserted == 0, "the scoped refresh already applied the change"
    assert index.total().files == scoped_total

    # A change outside the scope is not observed by a refresh scoped elsewhere.
    (root / "outside.txt").write_text("outside", encoding="utf-8")
    assert index.refresh("src").inserted == 0, "a scoped refresh must not reach outside it"
    assert index.refresh().inserted == 1, "the whole-tree refresh still finds it"

    # A naive datetime means local time; the facade must resolve it to the explicit
    # offset the engine's time grammar requires rather than letting it be rejected.
    recent = index.report(
        fdu.Query(
            views=(fdu.View.FILES,),
            selection=fdu.Selection(modified_since=datetime.now() - timedelta(hours=1)),
        )
    )
    recent_files = recent.sections[0]
    assert isinstance(recent_files, fdu.FilesSection)
    assert {row.path.name for row in recent_files.files} >= {"new.txt"}, recent_files

    cache_root = Path(tempfile.mkdtemp(prefix="fdu-public-cache-"))
    (cache_root / "cached.txt").write_text("cached", encoding="utf-8")
    fdu.open(cache_root, cache=fdu.CachePolicy.AUTO)
    cached = fdu.open(cache_root, cache=fdu.CachePolicy.ONLY)
    # Coverage and currency are independent: a snapshot can cover the complete scope
    # while remaining deliberately stale until revalidation.
    assert cached.status.complete is True
    assert cached.status.freshness is fdu.Freshness.STALE
    status = fdu.cache_status(cache_root)
    assert status is not None and status.recognized

    entrypoint = Path(sys.executable).with_name("fdu.exe" if os.name == "nt" else "fdu")
    version = subprocess.run([entrypoint, "--version"], check=False, capture_output=True, text=True)
    assert version.returncode == 0, version
    assert version.stdout.startswith(f"fdu {fdu.__version__}"), version.stdout
    assert version.stderr == "", version.stderr

    # Rebuild the API report after the refresh above so both sides observe the same
    # filesystem state. The independently scanned CLI document must otherwise match.
    wire = index.report(
        fdu.Query(
            views=(fdu.View.SUMMARY, fdu.View.EXTENSIONS, fdu.View.FILES),
            selection=fdu.Selection(size=fdu.SizeMetric.APPARENT),
        )
    ).as_dict()
    cli_report = subprocess.run(
        [
            entrypoint,
            "--cache",
            "off",
            "--format",
            "json",
            "--view",
            "summary,extensions,files",
            "--size",
            "apparent",
            str(root),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert cli_report.returncode == 0, cli_report
    cli_wire = json.loads(cli_report.stdout)
    assert wire["source"] == "warm_revalidate"
    assert cli_wire["source"] == "cold_scan"
    for volatile in ("scan_started_at", "generated_at", "source"):
        cli_wire.pop(volatile)
        wire.pop(volatile)
    assert wire == cli_wire, (wire, cli_wire)

    print(f"fdu {fdu.__version__} public API ok")


if __name__ == "__main__":
    main()
