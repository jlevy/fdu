"""A browser's inventory provider, built on fdu: identity, dual-plane rows, live changes.

The seam a file browser actually needs, in one file, because the four pieces are wrong
apart. A browser boots against a tree, draws a directory with two numbers per row, follows
the tree as it changes, and reconnects without re-reading everything -- and each of those
has one decision that fails silently if it is made naively.

**Identity is one string, and it must combine every fingerprint.** fdu reports its scope as
several named fingerprints because they answer different questions -- which tag rules ran,
which taxonomy classified, which reducers were maintained, which entries were admitted at
all. A consumer's cache key has to move when *any* of them does, so this composes them into
one value the way a consumer can reproduce: named components, sorted by name, canonical JSON,
SHA-256. Sorted because a caller listing the same components in another order has not asked
a different question; canonical because two encoders must agree byte for byte; and one
combined value because a consumer that keys on a subset caches an answer across a change
that invalidated it, which no field of the answer reveals.

**Two numbers per row come from one call.** A browser shows "1.2 GB, 340 MB shown" -- the
subtree, and the subtree excluding what git ignores. Asking for the second per row would
take the read guard once per child and re-resolve each path, so the plane rides on the page
request and every directory row answers with both. That needs the rule promoted at scan
time, which is what makes the read a roll-up lookup rather than a walk.

**A batch is a delta plus a verdict about what it invalidated.** `changes` is what moved;
`dirty_rollups`, `dirty_queries` and `all_dirty` are what a consumer must throw away. A
browser that repainted only the rows in `changes` would leave stale totals on every ancestor
of a change, so this maps the batch to the invalidation a view layer can act on.

**Resuming is a decision, not a replay.** The journal is bounded. A client away long enough
has fallen further behind than fdu can replay, and `ChangeSet.truncated` says so; replaying
its ops anyway produces a client that believes it is current while missing everything
evicted. See `sse_resume.py`, which is about that branch alone.

One more thing this file is careful about: a bounded answer measures its window from the
caller's instant. `modified_since="1h"` resolved afresh per request asks about a later
window each time, so membership drifts while the version stands still. `as_of` is chosen
once, by the caller.

**What this example cannot do yet, and says so where it matters.** A filtered report is
bounded but has no continuation, so `recent_slice` is a slice rather than a page
(`fdu-91ru`). A watch batch materialises entry rows whether or not a consumer reads them,
and does not carry terminal provider state, so `invalidation` is written to want nothing
from `changes` (`fdu-vfx7`). Both are engine gaps; neither is worked around here, because
an adapter that papered over them would be the mirror this design exists to avoid.
"""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

import fdu

#: The tag rule this provider promotes, so a plane read stays a roll-up read.
#:
#: One rule rather than a set: promotion multiplies the ancestor-merge path on every
#: mutation whether or not anyone reads the plane, so a provider promotes what it draws
#: and nothing else.
PLANE_RULE = "gitignore"

#: Hidden entries a browser keeps despite pruning the rest.
#:
#: Exact names. `.git` is the reason pruning exists at all -- it is routinely most of a
#: working tree and no browser shows it -- while `.github` is content a repository's own
#: authors wrote and expect to see.
HIDDEN_ALLOW = (".github",)


@dataclass(frozen=True, slots=True)
class Row:
    """One directory listing row, carrying both numbers a browser draws."""

    name: str
    is_dir: bool
    bytes: int
    #: Bytes excluding entries the promoted rule tags, or `None` for a non-directory.
    shown_bytes: int | None
    tags: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class Invalidation:
    """What a view layer must discard after one batch.

    Not the same thing as "what changed", and deliberately not derived from it. A file added
    deep in a tree changes one path and invalidates the totals of every directory above it;
    a filter that admits nothing still moves the tallies a tree view reports and yields no
    changes at all. A browser repainting the changed rows would leave numbers on screen that
    had stopped being true, with no event to blame.
    """

    rollups: tuple[Path, ...]
    queries: tuple[str, ...]
    everything: bool
    #: The consumer's own history expired; its cursor is worthless and it must re-read.
    resync: bool


def _pair_digest(components: dict[str, str]) -> str:
    """SHA-256 over the UTF-8 compact JSON array of sorted `[name, value]` string pairs.

    The consumer's encoding, spelled out rather than left to a default, because two
    implementations have to produce the same bytes. Pairs rather than objects; string
    values rather than native ones; sorted by name, since a caller listing the same
    components in another order has not asked a different question; and compact separators,
    because a space is a different digest.

    The first version of this hashed objects with integer values and was internally
    consistent, which is exactly what a test comparing a function to itself would have
    accepted. It agreed with nothing.
    """

    payload = json.dumps(
        [[name, components[name]] for name in sorted(components)],
        ensure_ascii=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def scope_fingerprint(options: fdu.ScanOptions) -> str:
    """Identity of what was admitted to the index: budgets, visibility, boundaries.

    Built from the options the adapter opened with rather than from `ScanScope`, and that
    is deliberate. The engine reports its own `hidden_fingerprint` -- a digest it uses as a
    cache key -- while the consumer's encoding wants the *allowlist*, as a compact
    canonical JSON string inside the outer array. A digest of a list is not the list, and
    hashing it here would produce a value no second implementation could reproduce.

    `hidden="keep"` has no counterpart on the consuming side, whose model always prunes
    hidden names except an allowlist. A provider adapter therefore prunes; the mode exists
    for fdu's own command line, which counts what is there.
    """

    return _pair_digest(
        {
            "follow_symlinks": "false",
            "hidden_allowlist": json.dumps(
                sorted(options.hidden_allow), ensure_ascii=True, separators=(",", ":")
            ),
            "max_depth": "null" if options.max_depth is None else str(options.max_depth),
            "stay_on_filesystem": "true" if options.one_filesystem else "false",
        }
    )


def semantic_fingerprint(scope: fdu.ScanScope) -> str:
    """Identity of every non-scope rule that can change a complete answer.

    Separate from the scope digest because they answer different questions and invalidate
    different things: a changed allowlist means the index holds different entries, while a
    changed taxonomy means the same entries are labelled differently. A consumer keying on
    one combined value would re-read for both, and a consumer keying on neither would serve
    an answer across a change that invalidated it.

    A provider with one native fingerprint returns it directly; fdu has three, so they are
    combined with the same pair encoding. All three go in -- promote a tag rule and the
    plane numbers move, change the registry and every group row does, and neither shows up
    in any field of the answer.

    Which components belong here is the part a second implementation cannot infer, and it
    is what the shared conformance fixture has to pin.
    """

    return _pair_digest(
        {
            "reducers": str(scope.reducers_fingerprint),
            "tag_rules": str(scope.tag_rules_fingerprint),
            "type_rules": str(scope.type_rules_fingerprint),
        }
    )


def open_tree(root: Path) -> fdu.Index:
    """Open an index a browser can serve from: promoted plane, hidden paths pruned."""

    return fdu.open(
        root,
        scan=fdu.ScanOptions(
            tag_rules=(PLANE_RULE,),
            promote=(PLANE_RULE,),
            hidden="prune",
            hidden_allow=HIDDEN_ALLOW,
        ),
    )


def listing(index: fdu.Index, path: Path, limit: int | None = None) -> tuple[Row, ...]:
    """One directory's rows, both numbers each, from one call."""

    page = index.children(path, limit=limit, plane=PLANE_RULE)
    if page is None:
        return ()
    return tuple(
        Row(
            name=child.name,
            is_dir=child.totals is not None,
            bytes=child.totals.bytes if child.totals is not None else (child.bytes or 0),
            shown_bytes=child.plane.bytes if child.plane is not None else None,
            tags=child.tags,
        )
        for child in page.rows
    )


def invalidation(batch: fdu.WatchBatch) -> Invalidation:
    """Translate one batch into what a view layer discards.

    Read off the bounded fields the batch already carries -- `dirty_rollups`, `dirty_queries`
    and `all_dirty` -- rather than reconstructed from `changes`. The distinction is not
    cosmetic: `changes` is unbounded in the size of the mutation, so building the
    invalidation from it makes an adapter's cost scale with what moved rather than with what
    it has to repaint, and a mutation the selection filters out changes the totals while
    yielding no changes at all.

    `reset` is about the consumer's own history rather than about the tree: the journal
    dropped the range this cursor sits in, so there is nothing to replay and the only honest
    move is a full re-read. It is deliberately separate from `all_dirty`, which says the
    tree changed everywhere -- one is "you are lost", the other is "everything moved".

    **What this cannot do yet.** The entry rows still cross the boundary: there is no
    invalidations-only interest mode, so `changes` is materialised whether or not a consumer
    reads it, and the batch does not carry the terminal provider state a consumer would
    otherwise re-read for. Both are open on `fdu-vfx7`. This function is written to want
    nothing from `changes` so that the mode, when it lands, changes the engine rather than
    this adapter.
    """

    return Invalidation(
        rollups=batch.dirty_rollups,
        queries=tuple(str(kind) for kind in batch.dirty_queries),
        everything=batch.all_dirty,
        resync=batch.reset,
    )


def recent_slice(index: fdu.Index, *, as_of: datetime, limit: int = 20) -> fdu.Bundle:
    """One bounded slice of "recently changed", measured from the caller's own instant.

    A **slice**, not a page, and the name is the honest one. `Selection.limit` bounds the
    rows a filtered report returns and the section says how many it withheld, but there is
    no continuation: no cursor to resume from, no `expected` version to pin the next request
    to, and no exact remaining-row count conserved to a terminal page. A consumer needing
    the whole answer cannot assemble it from bounded requests, which is what a provider
    contract requires. Native filtered-tree and catalog paging is open on `fdu-91ru`.

    The earlier draft of this passed `after` and `limit` to `index.read(children_of=...)`
    and called itself a page. Those arguments bound the *child listing*, an unrelated
    projection; the report was truncated independently and continued nowhere. It looked
    like paging and paged nothing.

    What is real here is the pin. `as_of` is the caller's, not this call's: a relative bound
    resolved afresh per request means the second one asks about a later window than the
    first, so an entry can appear twice or not at all while the version says nothing moved.
    """

    return index.read(
        query=fdu.Query(
            views=(fdu.View.FILES,),
            selection=fdu.Selection(
                modified_since="1h",
                sort=fdu.SortKey.MTIME,
                reverse=True,
                limit=limit,
            ),
            as_of=as_of,
        ),
    )


def main() -> None:
    """Boot, print the identity and one listing, then follow the tree for a few batches."""

    index = open_tree(Path())
    # The scope arrives on a bundle rather than on the index, because it is a fact about
    # one coherent read: an index that rebinds its rules mid-session has a scope that moved,
    # and a value cached off the handle would keep saying the old one.
    print(json.dumps({"identity": semantic_fingerprint(index.read().scope)}))
    for row in listing(index, Path(), limit=10):
        print(json.dumps({"name": row.name, "bytes": row.bytes, "shown": row.shown_bytes}))

    with index.watch(fdu.WatchOptions(interval=1.0)) as watch:
        for batch in watch:
            if not (batch.changes or batch.dirty):
                continue
            discard = invalidation(batch)
            print(json.dumps({"repaint": [str(path) for path in discard.rollups]}))
            break


if __name__ == "__main__":
    main()
