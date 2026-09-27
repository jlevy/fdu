---
type: is
id: is-01kzthak0rr8xefvmf9svswkmg
title: Enforce docs-format-check in CI and pin flowmark
kind: task
status: closed
priority: 3
version: 2
labels: []
dependencies: []
created_at: 2026-08-12T08:27:21.495Z
updated_at: 2026-09-27T08:12:59.552Z
closed_at: 2026-09-27T08:12:59.551Z
close_reason: "Implemented: frozen pinned Flowmark in docs-format-check, invoked by CI without absent-tool skip; current PR #133 CI 36303716655 passes."
resolution: null
duplicate_of: null
---
make docs-format-check is wired into make check but skips gracefully when flowmark is absent, so it is advisory rather than enforced. To make it real: add flowmark to a CI job and pin its version the way softschema is pinned in benchmarks/pyproject.toml, since it is a first-party tool exempt from the cool-off. Until then a contributor without flowmark can land unformatted docs and the drift is only caught locally.
