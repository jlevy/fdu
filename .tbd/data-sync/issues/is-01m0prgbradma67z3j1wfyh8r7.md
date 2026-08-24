---
type: is
id: is-01m0prgbradma67z3j1wfyh8r7
title: "Spec: fdu for interactive clients — the metabrowser contract"
kind: epic
status: open
priority: 1
version: 47
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
child_order_hints:
  - is-01m0prgyer27hqzdm2pvjx44qg
  - is-01m0prgyv1eq0g0mzgntn1p4n6
  - is-01m0prgz8hvk0rsm1edqgwty5d
  - is-01m0prgznztcx92ypmk6aanszf
  - is-01m0prhbvmd38p7eqffrg08nr6
  - is-01m0prhc835eec71rccdfe50zb
  - is-01m0prhcmhj09n41s1zae35yhm
  - is-01m0prhd1jfqr4gvtaq96hbwd7
  - is-01m0prhpj01wa15ypxm0er2q6s
  - is-01m0prhpzxz5tx9t61nj95aegw
  - is-01m0prhqd27m471dn47yt973k0
  - is-01m0pt934wtpzs87mtmg2hxhsg
  - is-01m0pt93kk5pytsjrb0v5wrweq
  - is-01m0pt9he483bx4et2eykcdp1j
  - is-01m0pt9j1mbn3za7fyym7pyr9t
  - is-01m0ptezmtmkn04mh1f1rwgdxb
  - is-01m0ptvnmg9p0s1qh9174hpcv3
  - is-01m0qs0msk75k8r89b44vqqjnz
  - is-01m0qs0nnjz9z4mkw35ahwydvs
  - is-01m0qs197189wae43fmqxs82bs
  - is-01m0qs19pg77zfmd3s2kg7k905
  - is-01m0racc8tf20x27jjhh35vh5q
  - is-01m0raccjvpde63hx884rkmq5d
  - is-01m0raccwe6ywyac61ezhxk2ws
  - is-01m0racd5dxjfx1g5e0dsfay8q
  - is-01m0rahh7entj80k486sxs5k45
  - is-01m0rw7a5ref8h8b8b17kxccbs
  - is-01m0rw7bvxtw87tgde30emgs56
  - is-01m0rw7cddvwh9vetyxkmgrvsm
  - is-01m0rw7d4h3t49rwvk11cmk5xb
  - is-01m0t5szzjt8kr7yqkzg78cxhm
  - is-01m0t5t1ghzmetfs4qjbrzx44r
  - is-01m0t5t249szzrfqrng85e36me
  - is-01m0t5t2sa2rn3qm3m4dycv7hv
  - is-01m0tdy6k1kfkywsy4f8kga870
  - is-01m0tdy76e3ndzcsdwf8m6j8sq
  - is-01m0tdy7tfq528ftppqfpteypv
  - is-01m0tdy8b6h17fqk7mqge56svh
  - is-01m0tdy8swsdre8d15s96wx4km
  - is-01m0tdy9ceep2byvbtyvwc2vky
  - is-01m0tdy9tx76dachmfcgrq5r3a
  - is-01m0te8vfk0w5tp9337vkth4wy
created_at: 2026-08-23T07:31:34.794Z
updated_at: 2026-08-24T17:49:40.722Z
---
Root epic for the interactive-client integration spec: partitioned tallies (tag planes), the embedder watch contract, the session integration shape, and the adoption proof. Each capability lands engine-first and clears the parity harness. The measured basis and the requirement-by-requirement contract map are in the spec.

## Notes

Delivery is two independent tracks. Metabrowser Phase 1 extracts its Python stack behind
the final sealed InventoryBackend/InventoryHandle contract with NO fdu dependency; Phase 2
implements the same contract against fdu, opening with a small real PyO3 spike (shared
handle, one bundled directory+rollup read returning one version/cursor/state/work record,
convergence after one live mutation, no mirror index). A bad seam there revises both
designs before either surface expands. So fdu sequences its own phases independently.
The conformance packet gates verification of the classification work, not the work
itself. Full record: docs/project/research/research-2026-08-23-interactive-contract-reconciliation.md

## Progress as of 2026-08-24

TWENTY of twenty-six children closed, on PR #47 (stacked on #44). Four of six phases are
substantially complete; two have not started and both are blocked rather than deferred.

DONE. Phase 0 (shared reads, order/threads knobs). Phase 2 (runtime registry, browsing
groups, two extension levels, bounded extension rows). Phase 3 (bundled coherent read,
scalar paged child rows, per-result work counters, roll-up leaf counts). Phase 4 (dirty
roll-up sets, scoped refresh, poll backend, asyncio adapter). Phase 6's classification
identity, walk telemetry and TreeNode remainders. Plus the test machinery the
implementation spec introduced: a scripted watch backend, and counter relations as a
golden-visible cost oracle.

Every one of the reconciliation's "new work" items is built: the batched multi-projection
read under one guard returning clock, cursor, state and fingerprints; per-result work
counters; roll-up leaf counts. Both flipped readiness verdicts are addressed on the
children() side.

NOT STARTED, AND WHY.
- Phase 1 (fdu-mvt3, fdu-7rwf). Scope is now settled smaller than the bead text: hidden
  prunes at scope, gitignore is the sole tag rule. The only real blocker left is the
  `ignore` crate owing the 14-day cool-off.
- Phase 5 (fdu-4o0m and the progress mode fdu-m893 behind it). Blocked on the
  progressive-results epic, which owns the session type these present.
- fdu-vfyw, fdu-ey9q. Blocked on the two above.
- fdu-n4gn. Cannot price a union whose members do not exist, and needs a quiet host.

CARRIED AS EXPLICIT DEBT, not as silence.
- fdu-2ig2: leaf counts shipped on the ancestor-merge path unmeasured. The reconciliation
  warns that a cost acceptable for each member alone can be wrong in combination, and one
  member is now in the hot path ahead of the measurement meant to choose the
  representation.
- fdu-gy3g: the conformance packet cannot usefully be vendored until its cases can tell
  the two extension levels apart.
- fdu-or38: report views still cannot distinguish a symlink-only directory from an empty
  one.
- fdu-plwq closed without per-row tags, which need Phase 1's planes; fdu-7rwf owns
  adding them.
- The resume cursor carries data changes but not trust transitions (fdu-jxs0, raised to
  P1 per the reconciliation). The SSE example says so rather than implying currency it
  does not have.

## Restructure and decisions, 2026-08-24 (late)

The owner directed the tag model be generic -- gitignore one rule among several -- and
delegated the approach. Decisions taken, recorded on the beads and in both specs:

1. `ignore` crate: feature-gated `gitignore`, default-on, notify's precedent. MSRV trap
   found by checking: ignore 0.4.31+ needs Rust 1.88 > MSRV 1.85, so pin =0.4.30 with
   globset held at 0.4.19 (both clear the cool-off). Evidence and pins on fdu-brt0.
2. Genericity applied: fdu-mvt3 re-scoped to the model foundation (tiers, bits,
   tag_rules_fingerprint rename, dotfile reference rule); gitignore -> fdu-brt0;
   promotion/planes -> fdu-pxfz; hidden admission as scope -> fdu-xyvu; flags fold-in
   -> fdu-n7mv (P3); surfaces remain fdu-7rwf, re-scoped.
3. fdu-2ig2: keep `others` (the implemented contract requires leaf counts); measure on
   any quiet host, or ride fdu-n4gn's run.
4. fdu-vrwy: its own change, not PR #47.

Epic now has 34 children: 19 closed, 15 open. Build order: Track A (contract) fdu-5yqb
-> fdu-samw, with fdu-fltq behind fdu-jxs0 and fdu-sgp7 behind fdu-4o0m. Track B (tags)
fdu-mvt3 -> {fdu-brt0, fdu-pxfz} -> fdu-7rwf -> {fdu-vfyw, fdu-n4gn}. Track C
(independent smalls): fdu-or38, fdu-xyvu, fdu-vrwy. The session chain stays behind the
progressive-results epic. Ready right now: fdu-5yqb, fdu-mvt3, fdu-or38, fdu-xyvu,
fdu-vrwy.
