"""Event-loop adapters for fdu's blocking surfaces.

`Watch` is a thread-affine blocking iterator, and stays one: a change feed is a pull, the
native layer releases the GIL for the duration of each pull, and an iterator that blocks
is the honest shape for something that waits. What an asyncio server needs is not a
different watcher but the handoff -- a worker thread draining that iterator into the event
loop -- and shipping the handoff here is the point. Every consumer would otherwise write
it again, around the same GIL, with the same three things to get wrong: cancellation,
backpressure, and thread affinity.

Thread affinity, precisely: a `Watch` belongs to the thread that created it and is not
shareable; the binding marks it unsendable so Python is told that at the boundary rather
than left to discover it. :func:`watch_batches` therefore opens the watch *on* its worker
thread, and that thread is the only one that ever touches it -- including closing it. An
`Index`, by contrast, is shared freely: it serves concurrent readers during a write, which
is what lets the consumer keep querying while the worker streams.

Batches are the same typed values `Watch` yields. This adapter changes when you receive
them, not what they are.
"""

from __future__ import annotations

import asyncio
import threading
from collections.abc import AsyncIterator

from ._api import Index
from ._models import WatchBatch, WatchOptions

__all__ = ["watch_batches"]

#: How long cancellation waits for the worker to notice and exit.
#:
#: The drain that precedes the join unblocks it, so this bound is a guard against a bug
#: rather than a normal path -- a worker that is still alive after it has been told to
#: stop and had its queue emptied is not waiting on anything this code controls.
_WORKER_JOIN_TIMEOUT = 5.0

#: Batches held for a slow consumer before the worker stops pulling.
#:
#: Bounded on purpose. An unbounded queue turns a consumer that cannot keep up into memory
#: growth that ends the process, which is a worse failure than falling behind: the engine
#: already coalesces, so waiting costs latency rather than fidelity.
DEFAULT_QUEUE_SIZE = 64


async def watch_batches(
    index: Index,
    options: WatchOptions | None = None,
    *,
    queue_size: int = DEFAULT_QUEUE_SIZE,
) -> AsyncIterator[WatchBatch]:
    """Yield `index.watch(options)`'s batches on the running event loop.

    A live UI should set `WatchOptions.interval` near its frame budget. The interval bounds
    how long one pull blocks before returning empty-handed, so it is the floor on how
    quickly a stop is noticed -- not on how quickly a change is seen. Detection is
    event-driven and an idle tree costs nothing between events.

    Empty batches are not yielded. They exist so a blocking iterator can return; an async
    consumer has no such need, and forwarding them would make every caller filter them.

    Cancelling the iterating task, or leaving the `async for` early, stops the worker,
    which closes the watch. Both happen within one `interval` -- the time an outstanding
    pull can still be blocked.

    An exception from the underlying watch is re-raised here, on the consumer's task,
    rather than lost on a background thread.
    """

    selected = options if options is not None else WatchOptions()
    loop = asyncio.get_running_loop()
    queue: asyncio.Queue[WatchBatch | BaseException | None] = asyncio.Queue(
        maxsize=max(1, queue_size)
    )
    stop = threading.Event()

    def drain() -> None:
        """Open, pull, and close the watch, all on the worker thread.

        Opening it here rather than handing one in is what keeps the affinity rule intact
        rather than merely documented: the watch never crosses a thread boundary at all.
        A failure to open -- a scan scope a watcher cannot narrow, say -- is relayed like
        any other, so the consumer sees it on its own task rather than losing it here.

        `run_coroutine_threadsafe(...).result()` is what makes backpressure real: the
        worker blocks while the queue is full instead of racing ahead of a consumer that
        cannot keep up. Blocking this thread costs nothing -- the pull it would otherwise
        be doing releases the GIL anyway.
        """

        def hand_over(item: WatchBatch | BaseException | None) -> None:
            # The loop can already be gone -- a cancelled task, a closing process -- and
            # there is nothing to report at that point: nobody is reading. The coroutine
            # is closed explicitly on that path, because one that was created and never
            # scheduled warns at collection time and the warning would point here rather
            # than at the shutdown that caused it.
            pending = queue.put(item)
            try:
                asyncio.run_coroutine_threadsafe(pending, loop).result()
            except BaseException:
                pending.close()

        try:
            with index.watch(selected) as watch:
                for batch in watch:
                    if stop.is_set():
                        break
                    # Every batch that observed something, not only those carrying
                    # changes. A mutation the selection filters out still moves the totals
                    # a tree view reports, and `if batch.changes` would discard the only
                    # signal saying so -- the async consumer would go on rendering numbers
                    # that had stopped being true, with no event to blame.
                    if batch.dirty or batch.changes:
                        hand_over(batch)
        except BaseException as error:
            hand_over(error)
        finally:
            hand_over(None)

    worker = threading.Thread(target=drain, name="fdu-watch-aio", daemon=True)
    worker.start()
    try:
        while True:
            item = await queue.get()
            if item is None:
                return
            if isinstance(item, BaseException):
                raise item
            yield item
    finally:
        stop.set()
        # Drain what is queued so a worker blocked on a full queue can complete its put,
        # see the stop, and exit. Without this a consumer that leaves early would leave
        # the thread parked on a future nothing will ever await.
        while True:
            try:
                queue.get_nowait()
            except asyncio.QueueEmpty:
                break
        # And wait for it to actually be gone. Setting a flag says "please stop"; joining
        # is what makes cancellation mean the watch registration is released by the time
        # this returns, rather than at some point afterwards. The drain above is what
        # guarantees the join finishes: a worker blocked on a full queue can now complete
        # its put and exit.
        worker.join(timeout=_WORKER_JOIN_TIMEOUT)
