---
type: is
id: is-01kzqmzewkph9n0w5rzn2a9hyg
title: "Spec: composable CLI and query surface (five axes)"
kind: epic
status: open
priority: 1
version: 24
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
child_order_hints:
  - is-01kzqmzxg6ne1qt2n6xtcz8k9x
  - is-01kzqn07pd0n9fvf00r6ate71f
  - is-01kzqn0ev2c1d4d5aa5098957r
  - is-01kzqn0ryk5bywq86c1f4k50fe
  - is-01kzqn1e57thy4skv9yjtcpp2h
  - is-01kzqn1shjfrhncb9bhebyqx73
  - is-01kzqn23etqsjxe0pnn1hx1jng
  - is-01kzqn2fn6p7qcmp31j87qesak
  - is-01kzqn2s3rwkxhb8ag9v4e6t24
  - is-01kzqn33mgtc6j3rk6tns1bawg
  - is-01kzqn3c33pyf3vh7070ehnfss
  - is-01kzqn3mqbn1wy0ms32gmm4nh6
  - is-01kzqn3xvxg58z2dcwjevgd439
  - is-01kzqn44q2r4r04yjsweznvyxe
  - is-01kzqn4eh461jy13mvs25bmwvn
  - is-01kzqn4rdq9vy4qvcve073rfhf
  - is-01kzqn502680awzhvddzntq32d
  - is-01kzqn5atxakb84p364hjfhg1p
  - is-01kzqn5jbrqef88q43pdd0pa71
  - is-01kzqn5vw0t83yh77s92f6njf9
  - is-01kzqn66p0pmck4yg6pexhww2z
  - is-01kzqscchfxr2p8rnk9csrq8w3
  - is-01kzqtb7a0va7ce09caacgd8s5
created_at: 2026-08-11T05:33:27.826Z
updated_at: 2026-08-11T07:07:16.159Z
---
Reshape CLI and query layer around five orthogonal axes (scope, selection, view, format, mode) per the spec. One scan serves many views; views are pure readers over the index; the CLI parses flags into library types (Query, CachePolicy, Report) and invents nothing; formats are schema-versioned serializations; watch is the same query repeated, event-driven never polling. Four phases, one PR per phase; Phase 1 is the breaking-rename PR. Golden discipline per golden-testing-guidelines: every new surface gets tryscript coverage (sandbox, fixtures, pinned env, patterns for unstable fields) plus schema-bump tests.
