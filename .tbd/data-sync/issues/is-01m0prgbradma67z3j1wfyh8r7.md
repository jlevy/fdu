---
type: is
id: is-01m0prgbradma67z3j1wfyh8r7
title: "Spec: fdu for interactive clients — the metabrowser contract"
kind: epic
status: open
priority: 1
version: 30
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
created_at: 2026-08-23T07:31:34.794Z
updated_at: 2026-08-24T00:54:04.284Z
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
