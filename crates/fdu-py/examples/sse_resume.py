"""Resume a Server-Sent Events change feed from `Last-Event-ID`, or resync honestly.

An SSE client that drops its connection reconnects with `Last-Event-ID`, and expects to
be told everything it missed. fdu answers that directly: every applied change carries a
cursor, and `Index.since(cursor)` returns what happened after it.

The cursor is `{session, clock}`, and the session half is what makes a reconnect safe
across a *restart*. A bare clock cannot say which opened index it came from: two runs over
the same tree both count from zero, so a `Last-Event-ID` minted by a previous process
compares as a perfectly ordinary position in the new one, and `since` would answer with an
empty set -- "you are up to date" about a place this index has never been. Carrying the
session turns that into a refusal the feed can act on, which is why `resume_cursor` below
returns `None` for a token this index does not recognize.

The part worth getting right is the other branch. The journal is bounded, so a client that
was away long enough has fallen further behind than fdu can replay -- and `ChangeSet`
says so with `truncated`. There is exactly one correct response, and it is not to send
the ops anyway: the client must throw its state away and re-read. Ignoring `truncated`
produces a client that believes it is current while silently missing everything evicted
from the journal, which no error will ever surface.

`decide` is a pure function over the `ChangeSet` so both branches are testable without
having to evict 64k ops from a journal to reach the interesting one.

**What this clock does not yet carry.** `since()` replays *data* changes. Provenance and
trust transitions -- a subtree moving from cached to verified, say -- do not ride this
clock today, so a client resuming from one is current on what changed and not on how far
to trust it. The interactive-client contract records that the resume cursor is not
complete for a production SSE feed until those transitions share the clock, which is
`fdu-jxs0`. Until then a feed built on this either omits trust from its envelope or
re-reads provenance on reconnect; what it must not do is imply currency it does not have.
"""

from __future__ import annotations

import asyncio
import json
from collections.abc import AsyncIterator
from dataclasses import dataclass

import fdu


@dataclass(frozen=True, slots=True)
class Resume:
    """What to send a reconnecting client."""

    resync: bool
    """True when the client must discard its state and re-read from scratch."""

    changes: tuple[fdu.Change, ...]
    """The changes to replay; empty when `resync` is set."""

    cursor: fdu.Cursor
    """The position the client should report next time."""


def decide(changeset: fdu.ChangeSet, current: fdu.Cursor) -> Resume:
    """Replay what the client missed, or tell it to start over.

    `truncated` is the whole decision. The changes a truncated set carries are real but
    incomplete, and a client that applied them would be wrong in a way it cannot detect.
    """

    if changeset.truncated:
        return Resume(resync=True, changes=(), cursor=current)
    return Resume(resync=False, changes=changeset.changes, cursor=changeset.cursor)


def format_event_id(cursor: fdu.Cursor) -> str:
    """The `id:` a client sends back, carrying both halves of the position."""

    return f"{cursor.session}-{cursor.clock}"


def parse_last_event_id(header: str | None) -> fdu.Cursor | None:
    """Read the position a reconnecting client reports, rejecting anything else.

    A header is client-controlled input. A malformed one means "I have no position",
    which is the same as a first connection -- not an error, and not a reason to guess.
    A token from a previous process parses fine here and is rejected one step later, by
    the engine, which is the only thing that knows which session is serving.
    """

    if header is None:
        return None
    session, _, clock = header.partition("-")
    try:
        parsed = fdu.Cursor(session=int(session), clock=int(clock))
    except ValueError:
        return None
    return parsed if parsed.session > 0 and parsed.clock >= 0 else None


def sse_event(name: str, payload: object, event_id: fdu.Cursor) -> str:
    """One SSE frame. `id:` is what comes back as `Last-Event-ID`."""

    return f"id: {format_event_id(event_id)}\nevent: {name}\ndata: {json.dumps(payload)}\n\n"


async def feed(
    index: fdu.Index,
    last_event_id: str | None,
    options: fdu.WatchOptions | None = None,
) -> AsyncIterator[str]:
    """The SSE body: catch the client up, then stream.

    Catch-up happens before the first live batch is awaited, so a client that reconnects
    during a burst does not miss the changes that landed while it was gone.
    """

    current = index.cursor()
    resumed = parse_last_event_id(last_event_id)
    if resumed is None:
        # A first connection has nothing to catch up on, and is told where it starts.
        yield sse_event("resync", {"reason": "first-connection"}, current)
    else:
        try:
            changeset = index.since(resumed)
        except fdu.FduError:
            # The engine did not recognize the position: another process opened this
            # tree, or this one reopened it. Both are a resync, and both used to be
            # invisible -- a stale token returned an empty set that read as "current".
            yield sse_event("resync", {"reason": "unknown-session"}, current)
            return
        decision = decide(changeset, current)
        if decision.resync:
            yield sse_event(
                "resync",
                {
                    "reason": "journal-truncated",
                    "behind_by": current.clock - resumed.clock,
                },
                decision.cursor,
            )
        else:
            for change in decision.changes:
                yield sse_event("change", _wire(change), _at(current, change.clock))

    async for batch in fdu.aio.watch_batches(index, options):
        # A batch that observed something but carries no admitted changes still moved the
        # totals; a feed that renders aggregates re-reads on `batch.dirty`. This example
        # streams entry changes only, so it forwards what it has.
        for change in batch.changes:
            yield sse_event("change", _wire(change), _at(current, change.clock))


def _at(cursor: fdu.Cursor, clock: int) -> fdu.Cursor:
    """One change's position within the session serving this connection."""

    return fdu.Cursor(session=cursor.session, clock=clock)


def _wire(change: fdu.Change) -> dict[str, object]:
    """This application's own wire shape, deliberately not an fdu model."""

    return {
        "path": str(change.path),
        "kind": str(change.kind),
        "bytes": change.bytes,
        "clock": change.clock,
    }


async def main() -> None:
    """Print the first few frames a fresh client would receive."""

    index = fdu.open(".")
    frames = 0
    async for frame in feed(index, None, fdu.WatchOptions(interval=0.2)):
        print(frame, end="")
        frames += 1
        if frames >= 3:
            break


if __name__ == "__main__":
    asyncio.run(main())
