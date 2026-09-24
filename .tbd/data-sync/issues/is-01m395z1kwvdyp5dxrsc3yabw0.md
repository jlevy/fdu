---
type: is
id: is-01m395z1kwvdyp5dxrsc3yabw0
title: "PR #123 review R4: name recheck must probe fdu_core (crates.io -/_ collision rule)"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m395wcbdbx8ndh16agfhmrhy
created_at: 2026-09-24T07:44:30.838Z
updated_at: 2026-09-24T07:50:44.615Z
closed_at: 2026-09-24T07:50:44.615Z
close_reason: "Fixed in 3902f7cd: fdu_core probe added to the pre-tag name recheck."
resolution: null
duplicate_of: null
---
docs/project/guides/release-process.md, Tag the Release Commit step 2: crates.io refuses a new crate whose name differs from an existing one only by - versus _, so fdu_core existing blocks fdu-core. Add the probe. PR #123 review R4 (Low).
