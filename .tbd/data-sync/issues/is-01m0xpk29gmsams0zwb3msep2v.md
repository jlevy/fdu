---
type: is
id: is-01m0xpk29gmsams0zwb3msep2v
title: Replace surgical parsing in golden sessions with full stable output
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#discussion_r3858495113
    at: 2026-08-26T00:12:51.903Z
labels:
  - pr47-review
dependencies: []
parent_id: is-01m0xns5sa6dgxxa5y4a1ts8xv
created_at: 2026-08-26T00:12:47.267Z
updated_at: 2026-08-26T00:28:12.880Z
closed_at: 2026-08-26T00:28:12.879Z
close_reason: Concurrent review work created fdu-9tdm first for the same exact GitHub finding. Its notes now preserve this bead’s full audit scope and acceptance criteria.
resolution: duplicate
duplicate_of: is-01m0xp3y8tpxjcgvmbxc7r3ykb
---
PR #47 review finding from https://github.com/jlevy/fdu/pull/47#discussion_r3858495113 at exact head 0558c7eff1b91a1dca052d4259dbe3751f6ffcd0. Several tryscript goldens defeat transparent-box coverage by spawning Node to parse FDU JSON/counter output and emit only hand-picked booleans or fields. The clearest cases are all three scenarios in tests/golden/cli-cost.tryscript.md and the report-extraction scenarios in tests/golden/cli-content.tryscript.md; inspect every golden, distinguishing fixture setup from surgical result extraction. Apply golden-testing-guidelines: direct golden commands should show complete stable product output; move relational/invariant checks that cannot be represented stably into ordinary Rust/integration tests layered beside the golden; shrink fixtures or split scenarios when full output is too large; use patterns only for genuinely unstable values. Do not replace the one-line parsers with a shared opaque parser or another narrow projection. Acceptance: every result-parsing script is removed or justified by the documented too-large/capture-value exception, the remaining goldens expose broad reviewable state, critical cost/cache/content relations have direct domain assertions, focused golden tests and full CI pass, and the originating review thread receives a disposition map.
