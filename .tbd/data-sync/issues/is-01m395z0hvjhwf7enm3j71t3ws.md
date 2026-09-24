---
type: is
id: is-01m395z0hvjhwf7enm3j71t3ws
title: "PR #123 review R3: workflow-invariant tests must pin the publish condition's structure and the trigger set"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m395wcbdbx8ndh16agfhmrhy
created_at: 2026-09-24T07:44:29.730Z
updated_at: 2026-09-24T07:50:44.029Z
closed_at: 2026-09-24T07:50:44.028Z
close_reason: "Fixed in 15e61d0e: publish condition structure and single-trigger assertions."
resolution: null
duplicate_of: null
---
tests/release/test_metadata.py asserts the four publish-if clauses are present but not conjoined ('||', 'always()' pass) and forbids only 'push:' in the trigger block. Assert no ||/always()/!cancelled() and that the trigger block declares only workflow_dispatch. PR #123 review R3 (Medium).
