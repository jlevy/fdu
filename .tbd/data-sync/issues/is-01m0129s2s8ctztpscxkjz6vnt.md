---
type: is
id: is-01m0129s2s8ctztpscxkjz6vnt
title: Stabilize the typed fdu Python package and roll-up API
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0129stsv3wn03hy6m3j4r4t
  - type: blocks
    target: is-01m0129sjsp6xht1ae8aebvx0r
parent_id: is-01m01293x5gaacv3vxjdtrg146
created_at: 2026-08-14T21:19:27.320Z
updated_at: 2026-08-14T22:42:43.110Z
---
Replace the unpublished fdu_py public import with an fdu package over a private native extension. Add typed options, enums, immutable report and roll-up records, structured partial errors, py.typed and extension stubs; separate completeness from freshness; expose provenance and missing scope/query parity; and prove runtime-to-stub exports under strict downstream type checking.

## Notes

Implementation branch codex/python-packaging-release-engineering created directly from origin/main at 043e5a7, with the reviewed plan commits cherry-picked. Scope is the complete mixed-PyO3 Python package and API contract. Registry project creation, trusted-publisher configuration, tokens, tags, uploads, and GitHub Release publication remain excluded under fdu-9cf0.
