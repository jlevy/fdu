---
type: is
id: is-01m2pye9p8df0h0rzch4fq37wy
title: "P2.2.2: Use the scalar policy in the existing writers; strict YAML parsing in check-yaml.mjs"
kind: task
status: in_progress
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyea2jz71fks8v1zep7c21
  - type: blocks
    target: is-01m0k512k9a6dq2k51fbfe5xn4
  - type: blocks
    target: is-01m2pj0gs0cyxp178kz5kktpbb
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
hold: null
hold_until: null
created_at: 2026-09-17T05:46:42.247Z
updated_at: 2026-09-20T04:35:01.902Z
started_at: 2026-09-20T04:35:01.902Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 2: The Answer Model and Writers", commit 2. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `emit/emit_scalar.rs` replaces `quote` (`report_format.rs:1277-1295`) and `yaml_scalar` (`:1240-1252`) inside today's writers.
- `scripts/check-yaml.mjs`, reusing the unmerged YAML fixes recorded on fdu-c2ml (branch `claude/release-e2e-fixes`: nested metric rows, `path_raw`/`root_raw`, the stricter scalar rule, and the YAML-equals-JSON check): awkward and non-UTF-8 names, `documents`, strict YAML 1.2 (`strict`, `uniqueKeys`, `intAsBigInt`) and YAML 1.1 parsing of every document kind deep-equal to exactly parsed JSON (the `JSON.parse` reviver's `context.source`) and to reassembled JSON Lines, across views and analyzer sets, cache status, and a watch stream; no raw C1, U+2028, or U+FFFE.

**Tests**

- `report_format.rs`: rewrite the YAML quoting and JSON escaping tests (`:2902`, `:2915`).

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
