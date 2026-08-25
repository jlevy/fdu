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

One more thing this file is careful about: a paged assembly pins its own clock. Page two of
"recently changed" resolved `modified_since="1h"` against a later instant than page one, so
membership drifted while the version stood still. `as_of` is chosen once, by the caller,
and passed with every page.
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

    Not the same thing as "what changed". A file added deep in a tree changes one path and
    invalidates the totals of every directory above it; a filter that admits nothing still
    moves the tallies a tree view reports. `paths` is the first, `rollups` and `queries` are
    the second, and a browser that repainted only `paths` would leave numbers on screen that
    had stopped being true with no event to blame.
    """

    paths: tuple[Path, ...]
    rollups: tuple[Path, ...]
    queries: tuple[str, ...]
    everything: bool
    #: The consumer's own history expired; its cursor is worthless and it must re-read.
    resync: bool


def semantic_fingerprint(scope: fdu.ScanScope) -> str:
    """Combine fdu's named scope fingerprints into one identity a consumer can key on.

    Named components, sorted by name, canonical JSON, SHA-256 hex -- the recipe a second
    implementation has to reproduce byte for byte, which is why every part of it is stated
    rather than left to a default. `sort_keys` and the compact separators are what make two
    encoders agree; `ensure_ascii` keeps the bytes identical where a name is not ASCII.

    Every fingerprint goes in. A consumer keying on a subset caches an answer across a
    change that invalidated it: promote a rule and the plane numbers move, prune hidden
    paths and the totals move, and neither shows up in any field of the answer.
    """

    components = {
        "hidden": scope.hidden_fingerprint,
        "reducers": scope.reducers_fingerprint,
        "tag_rules": scope.tag_rules_fingerprint,
        "type_rules": scope.type_rules_fingerprint,
    }
    payload = json.dumps(
        [{"name": name, "value": components[name]} for name in sorted(components)],
        ensure_ascii=True,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


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

    `reset` is about the consumer's own history rather than about the tree: the journal
    dropped the range this cursor sits in, so there is nothing to replay and the only honest
    move is a full re-read. It is deliberately separate from `all_dirty`, which says the
    tree changed everywhere -- one is "you are lost", the other is "everything moved".
    """

    return Invalidation(
        paths=tuple(change.path for change in batch.changes),
        rollups=batch.dirty_rollups,
        queries=tuple(str(kind) for kind in batch.dirty_queries),
        everything=batch.all_dirty,
        resync=batch.reset,
    )


def recent_page(
    index: fdu.Index,
    *,
    as_of: datetime,
    after: str | None = None,
    limit: int = 20,
) -> fdu.Bundle:
    """One page of "recently changed", with the caller's instant pinned across the whole run.

    `as_of` is the caller's, not this call's. A relative bound resolved afresh per page
    means page two asks about a later window than page one, so an entry can appear twice or
    not at all while the version says nothing moved -- the pin is what makes a paged
    assembly one answer rather than several.
    """

    return index.read(
        children_of=Path(),
        after=after,
        limit=limit,
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
