---
type: is
id: is-01kzj8wd52dh1pxg4834fb2q0v
title: "PR #1 review R4: Fingerprint snapshots by semantic scan scope"
kind: bug
status: closed
priority: 1
version: 7
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzj8wdv4jsmw5msz246hqbbc
  - type: blocks
    target: is-01kzj8we2pr4g03f5tdt8n7t08
  - type: blocks
    target: is-01kzj8weryq7v9tf11vr9q5psv
  - type: blocks
    target: is-01kzj8wf8380p0rf53pzemj4w7
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:52.673Z
updated_at: 2026-08-09T03:39:36.694Z
closed_at: 2026-08-09T03:39:36.693Z
close_reason: Fixed with persisted ScanScope identity covering traversal policies and rule/reducer fingerprints, strict warm-open comparison, cold-scan fallback on mismatch, and tests proving depth mismatch invalidates while batch size does not.
---
PR #1 review R4. Files: crates/fdu/src/lib.rs, crates/fdu/src/snapshot.rs, crates/fdu/src/scan.rs. Persist and compare semantic scan scope including max depth, symlink and filesystem-boundary policies, and rule/reducer identities. Operational batch size must not invalidate. Scope mismatch must cold-scan or explicitly prune.
