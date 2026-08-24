---
type: is
id: is-01m0tra6d42b6r8vsecjnh36e8
title: "[feature] Selection.max_size: the upper size bound the catalog contract needs"
kind: feature
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T20:45:10.436Z
updated_at: 2026-08-24T23:08:45.497Z
closed_at: 2026-08-24T23:08:45.496Z
close_reason: |
  Shipped with `fdu-662n` in one commit. `make check` green, parity holds.

  `Selection.max_size: Option<u64>`, inclusive, mirroring `min_size`. Present on every
  surface, because a capability reachable one way has to be reachable the others:
  `--max-size` on the command line, `max_size=` on `fdu.Selection`, and the field in
  `Selection` itself. `is_unfiltered` accounts for it, so a capped query correctly takes the
  re-aggregating tier rather than reading a maintained roll-up that does not know about the
  bound.

  INCLUSIVE ON PURPOSE. "At most 1M" is what a person means by a maximum, and keeping it
  symmetric with `min_size` is what makes the pair read as a window. MetaBrowser's
  `CatalogQuery.size_less_than` is *exclusive*; that translation (`max_size = n - 1`, with
  zero meaning an empty selection) belongs in the adapter, not in the engine's vocabulary.
  Recorded in the field's own doc comment so the next reader does not have to rediscover
  which side the asymmetry lives on.

  WHY IT EXISTS. Activity discovery filters candidates by size so a native provider returns
  only what matters across the binding. Without an upper bound the adapter would carry every
  candidate over and discard it in Python, which is precisely the cost a native provider is
  supposed to remove.

  TESTS. `max_size_bounds_from_above_the_way_min_size_bounds_from_below` covers the
  inclusive edge, the window, a reversed window admitting nothing, and that it follows the
  selected size metric like every other size predicate. A golden session on both surfaces,
  recorded by the parity harness as an exact match. The Python check derives its cap from
  the fixture's own largest file rather than hard-coding a number, so it stays meaningful if
  the fixture changes.

  Unblocks `fdu-vfyw`: the acceptance slice's catalog query can now run native-side.
resolution: null
duplicate_of: null
---
Found by the 2026-08-24 design review, reading MetaBrowser's sealed provider contract
(arch-inventory-provider.md at b4be2d0) against fdu's Selection. `CatalogQuery` carries a
`size_less_than` predicate -- an exclusive upper byte bound -- and activity discovery
filters by it so a native provider returns only candidates across the binding. fdu's
`Selection` has `min_size` only; there is no upper bound, so the adapter would have to
over-fetch and filter in Python, which is exactly the boundary cost the contract exists
to avoid.

DESIGN. Add `max_size: Option<u64>` to `Selection`, inclusive, mirroring `min_size`'s
inclusive-lower semantics (candidate below min is out; candidate above max is out). The
adapter translates the contract's exclusive `size_less_than: N` as
`max_size = N.checked_sub(1)`, with `N == 0` short-circuiting to an empty selection.
Keeping fdu's own semantics inclusive keeps `--min-size`/`--max-size` symmetric on the
command line, where "at most 1M" is what a person means by a maximum; the
exclusive-bound translation is contract-specific and belongs in the adapter.

Follows min_size everywhere: `is_unfiltered`, the admit path (same SizeMetric the
selection already carries), CLI `--max-size` on the Selection axis, Python
`max_size=`, goldens for both surfaces, and the parity corpus. Small change; the whole
point is that the engine grows the capability and every surface presents it.

Blocks fdu-vfyw: the acceptance slice's catalog query cannot run native-side without it.
