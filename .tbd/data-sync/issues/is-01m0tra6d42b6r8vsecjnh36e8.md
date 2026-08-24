---
type: is
id: is-01m0tra6d42b6r8vsecjnh36e8
title: "[feature] Selection.max_size: the upper size bound the catalog contract needs"
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T20:45:10.436Z
updated_at: 2026-08-24T20:45:42.768Z
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
