---
type: is
id: is-01m0129rtmjnmb8y1a4zpwrckp
title: Audit release readiness and specify packaging and Python API polish
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels: []
dependencies: []
parent_id: is-01m01293x5gaacv3vxjdtrg146
created_at: 2026-08-14T21:19:27.059Z
updated_at: 2026-08-14T21:23:24.389Z
closed_at: 2026-08-14T21:23:24.388Z
close_reason: Completed the senior release-readiness audit on fetched main 043e5a7, verified the required make check and artifact install paths, compared the Python roll-up surface with metabrowser-shaped consumers, recorded nine blocking packaging/API findings and concrete 0.1.0 design decisions in the linked plan, and created dependency-ordered implementation beads. No release artifacts were published.
---
Review the fetched main revision as an installed Rust CLI, library crate, wheel console script, uvx tool, and downstream Python dependency. Compare the Python surface with a representative typed roll-up client, record release blockers and decisions in the plan spec, validate locally and in CI, and hand off implementation beads.
