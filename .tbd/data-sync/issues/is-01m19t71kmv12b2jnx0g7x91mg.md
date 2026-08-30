---
type: is
id: is-01m19t71kmv12b2jnx0g7x91mg
title: "Opening an escaped entry: the contract path is an identity, not an address"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-30T17:07:00.851Z
updated_at: 2026-08-30T17:07:00.851Z
---
Deferred deliberately, because the case is rare and the alternative was complexity
everywhere for it.

The contract path is now canonical: the provider escapes at `_semantic_entry`, the single
outbound boundary, so rows crossing to a consumer carry `x%FF.txt` where the filesystem
holds a byte-0xFF name. The retained store still holds the platform name, so nothing is
stored twice.

About seven sites treat a contract path as a filesystem *address* rather than an identity:

- `walk.py:143` `abs_path = root if path == "" else root / path`
- `active_tracker.py:121,150,219` `root / record.path`
- `tree.py:704` `root_abs / entry.path`
- `inventory_engine/runtime.py:76` `root / relative_path`
- `providers/python_inventory.py:2130` `root / rel`

For an escaped entry those look for a file literally named `x%FF.txt`, which does not
exist. So such an entry is listable, orderable, and consistent across providers -- and may
fail to open.

That is a deliberate trade. Before this work, one undecodable name made the entire
directory unlistable, because ordering encoded the name to UTF-8 and encoding a surrogate
raises. Degrading to "you can see it but may not be able to open it" is strictly better,
and it costs nothing to the common case.

## What closing it would take

The encoding is injective by construction, so the inverse exists: parse `%XX` back to a
byte and re-encode with `surrogateescape` on POSIX, or back to a UTF-16 code unit on
Windows. One `platform_path(canonical)` helper, a no-op when `"%" not in path`, applied at
the sites above.

## Why not now

Non-UTF-8 names cannot exist on APFS, so this is unreachable on macOS and untestable
end-to-end there. It is reachable on Linux and uncommon in the trees people browse. Adding
a second path field to every row, or a decoder to every addressing site, is a permanent
cost paid by every file for a case most working trees never contain.

Worth doing when someone reports a file they can see and cannot open. The bounded, honest
version of the failure is what makes deferring it defensible.
