---
type: is
id: is-01kzrrhx6sw5kswb26efdz2sne
title: "PR #4 review R1: reject missing selected comparison cleanly"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01kzrrhm8bnv31bc5rqdex8yfp
created_at: 2026-08-11T15:55:12.472Z
updated_at: 2026-08-11T16:00:02.634Z
closed_at: 2026-08-11T16:00:02.633Z
close_reason: Fixed non-baseline recording to raise a clear ValueError when the primary job has no selected comparison; added a red-green regression test and passed make check.
---
PR #4 thread PRRT_kwDOTx0nJs6YKqAd at benchmarks/realtree/record.py:223-234. _selected_comparison may return None, but _verdict_evidence passes it to resource_guardrail and crashes with AttributeError. Add a focused failing test and return a clear validation error instead.
