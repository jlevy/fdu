---
type: is
id: is-01m395yyerh2pd48exc8dx0x7s
title: "PR #123 review R1: gate each cargo publish on its compare step having succeeded"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m395wcbdbx8ndh16agfhmrhy
created_at: 2026-09-24T07:44:27.603Z
updated_at: 2026-09-24T07:50:42.760Z
closed_at: 2026-09-24T07:50:42.759Z
close_reason: "Fixed in 15e61d0e on claude/release-publish: publish steps gated on their comparison step's outcome; wiring tests added."
resolution: null
duplicate_of: null
---
release.yml publish job: 'Publish fdu-core' is gated on steps.audit.outputs.fdu_core == 'missing' while the compare step is gated on steps.audit.outputs.crates == 'true'; an empty 'crates' output skips the comparison but not the upload. Add step ids and '&& steps.<id>.outcome == "success"' to both publish steps; add a test tying every steps.audit/pypi.outputs.* reference in the publish job to the keys publish_gate.audit() returns. PR #123 review R1 (Medium).
