---
type: is
id: is-01m2h3ctbd3p1jss1qa0f181sz
title: "PR #56 review PR56B-JRN-1: MIN_JOURNAL_CAPACITY_BYTES admits budgets that retain no real commit"
kind: bug
status: in_progress
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2h3ches22p7k9x4kka6bq6q
created_at: 2026-09-14T23:17:50.060Z
updated_at: 2026-09-14T23:53:44.014Z
---
Delta review 5203772881 on PR #56. engine_contract.rs:1566-1573; index.rs:2406-2416; opened.rs:2957-2995 at 8d2eb7f. The 512-byte floor equals a root-only commit's cost; the smallest commit a real tree produces costs 642, so 512..=641 retain nothing, and an item count such as 1024 or 4096 passed as bytes still resets on nearly every commit. Fix: raise the floor to a value that rejects plausible item counts and guarantees useful retention, justified from the cost model in the constant's doc; update the error message, the Python mapping, the CHANGELOG, docs, and the slow-consumer test that opens at the minimum.
