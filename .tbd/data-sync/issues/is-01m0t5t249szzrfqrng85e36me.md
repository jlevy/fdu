---
type: is
id: is-01m0t5t249szzrfqrng85e36me
title: Hidden-path admission as scope, with an exact-name allowlist
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T15:21:47.400Z
updated_at: 2026-08-24T15:21:47.400Z
---
Hidden-path admission as scope, split out of the old fdu-mvt3 during the 2026-08-24
restructure so the tag model does not carry it. This is the reconciliation's settled
answer: hidden files PRUNE at scope with an exact-name allowlist. They are not a tag
and not a plane, because the only named consumer wants them absent, and a tag would
still walk the .git, cache, and virtualenv trees it exists to exclude.

Distinct from the dotfile TAG the model ships, and the distinction is the axis test:
the tag filters with both numbers visible; this prunes, and a pruned entry does not
exist in the index at all.

WHAT LANDS: prune hidden components during the walk except a configured exact-name
allowlist; keep governing control files readable without retaining them; fingerprint
the rule and its allowlist into snapshot identity. ScanScope is positional in the
snapshot header, so the new field means a FORMAT_VERSION bump -- deliberate this time,
unlike the leaf-count case where recompute-at-load avoided one.

No dependency on the tag model. P2: metabrowser Phase 2 wants it, nothing in the
critical plane path does.
