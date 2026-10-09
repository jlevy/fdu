---
type: is
id: is-01m4fxa1r9zx7yxe0a8phqm82x
title: "fdu 0.5.0: several paths in one report, and an age column in the tree"
kind: epic
status: open
priority: 1
version: 19
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
child_order_hints:
  - is-01m4fxammtqwc8mb1362jgtmhn
  - is-01m4fxankw8s2qrmnnkn3ejs75
  - is-01m4fxapwnccxwmbfezf7gh0tk
  - is-01m4fxar5gf3qrax2k0rwfczn0
  - is-01m4fxasm0nae6c2tbjnqhd6rz
  - is-01m4fxattv1jb2w21ae6814ex9
  - is-01m4fxaw4tnmt77j099fhnk8j6
  - is-01m4fxax3s3274rk55njmnrksk
  - is-01m4fxay0x5wxrjarhc892gfhp
  - is-01m4fxayv9jpnd3nnh65f2q1nw
  - is-01m4fxazfga262r6aj9xfrcqfc
  - is-01m4fyjg32yetj9va263jqezc2
  - is-01m4gc336c85bew881nh38n3xp
  - is-01m4gc33xdg0re1zed4cjdtrmd
  - is-01m4gc34r0v9f5aj7ymepcsacv
  - is-01m4ghgje0a1raa8fwqy6m085k
  - is-01m4ghh5h0wmyx09t533pfm7zx
  - is-01m4gkdgh3n1dxknsxzfxa6avh
created_at: 2026-10-09T08:43:37.094Z
updated_at: 2026-10-09T15:09:59.202Z
---
Implements the plan spec. Part 1: the tree shows each subtree's age (the list-row activity definition, unknown when incomplete) in text and RFC 3339 + exact ns in machine formats; fdu.report/11. Part 2: PATH... over one or more disjoint roots, merged in the engine before bounds, as if each were run and summed. Answers fdu-onoo and fdu-khu8 open question 2.
