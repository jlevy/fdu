---
type: is
id: is-01m2g94gpx4me1qbw9afawk4vd
title: rollup() looks up un-normalized paths in a map keyed by normalized ones
kind: bug
status: open
priority: 2
version: 1
labels:
  - correctness
dependencies: []
created_at: 2026-09-14T15:38:55.068Z
updated_at: 2026-09-14T15:38:55.068Z
---
`ContentIndex::rollup(path)` does `self.rollups.get(path)` with the caller's raw path,
but every key in that map was inserted from a `PathKey`-normalized path (`merge_ancestors`
is called with `&key.0`). `ContentIndex::file()` normalizes its argument before looking
up; `rollup()` does not.

On unix `normalized()` is the identity, so nothing can go wrong and no test can see it.
On Windows a caller that spells a directory with `/` where the stored key holds `\` gets
`None` from `rollup()` for a directory whose roll-up exists. That is a silently missing
roll-up rather than an error.

Found while reading the map for H103/exp-104, which touched exactly these lines; the
H103 candidate incidentally fixed it by routing `rollup()` through
`path_bytes(&normalized(path))` the way `file()` already does, but that candidate was
rejected on its own merits and reverted, so the one-line fix went with it.

The fix is that one line. What it needs that H103 did not is a Windows-shaped test:
insert roll-ups through the normal path, then look one up with the other separator and
assert it is found. The separator work in 6c7a099/f204abb (exp-070) is the precedent for
how this repo tests that class.

Not a performance change -- it costs one `normalized()` call on a path already in cache
-- so it should not go through the accept rule. Correctness only.
