---
type: is
id: is-01m0prgbradma67z3j1wfyh8r7
title: "Spec: fdu for interactive clients — the metabrowser contract"
kind: epic
status: open
priority: 1
version: 29
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
updated_at: 2026-08-23T22:06:01.965Z
---
Root epic for the interactive-client integration spec: partitioned tallies (tag planes), the embedder watch contract, the session integration shape, and the adoption proof. Each capability lands engine-first and clears the parity harness. The measured basis and the requirement-by-requirement contract map are in the spec.

## Notes

Delivery is two independent tracks. Metabrowser Phase 1 extracts its Python stack behind the final sealed InventoryBackend/InventoryHandle contract with NO fdu dependency; Phase 2 implements the same contract against fdu, opening with a small real PyO3 spike (shared handle, one bundled directory+rollup read returning one version/cursor/state/work record, convergence after one live mutation, no mirror index). A bad seam there revises both designs before either surface expands. So fdu sequences its own phases independently; the earlier three-spike wording is metabrowser's Phase 2 evidence loop, not a joint delivery plan. Land fdu-gav9 and the bundled read early since the coupled spike needs them. The conformance packet gates verification of the classification work, not the work itself. Full record: docs/project/research/research-2026-08-23-interactive-contract-reconciliation.md
