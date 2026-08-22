---
type: is
id: is-01m0ncz0wtawmvece4fpe84zwp
title: Make the hypothesis registry an artifact set with checkable pre-registration
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md
labels: []
dependencies: []
parent_id: is-01m0ncyyjd1r8yh51evp1v4vcn
created_at: 2026-08-22T18:50:37.849Z
updated_at: 2026-08-22T18:50:37.849Z
---
The registry is the one place fdu does not apply its own discipline: a hand-maintained table, free-text hypothesis ids on artifacts, no check that a referenced id exists, status updated by hand. Make each hypothesis one soft-schema artifact carrying the claim, the predicted metric and direction, the regime, and the registration date. Generate status from the experiments that reference it. Fail the build on an unknown id. This is what turns the pre-registered-metric rule from honoured into enforced.
