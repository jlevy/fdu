---
type: is
id: is-01m76mhgvsr7fg5vn9d0vhs21p
title: "Release 0.6.0: tag, publish, and announce"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-12T01:04:20.344Z
updated_at: 2026-11-12T20:27:14.182Z
---
Release commit 8a8febc7cf332dfd4ee8c8536dd6cc00128be9dd (merge of #209, tree 64a7d366). Done by the cloud session on 2026-11-12, reversible steps only:
- Step 2 stability: the #209 Linux host gate pass at 98784661 (same tree) stands by tree identity; make check's only failure was its provenance request refused by the session proxy (rest passed under -k; CI green at that commit). Root supplement on 8a8febc7 (candidate + correctness, refusal as nobody, 16 kinds): all passed. Records in draft PR #210.
- Step 3 preflight: ok for COMMIT on main, Cargo versions, notes, CHANGELOG (2026-11-12), tag absent, crates.io and PyPI unpublished, vulnerability reporting, signing key; demo video, release environment, and registry secrets could not be read through the session's GitHub integration (403) and need the maintainer's gh.
- Step 4: release/v0.6.0 pinned at 8a8febc7 on origin; dispatching the rehearsal was refused to the session's integration (403), so run make release-candidate (it accepts the existing pin).
- Step 5: notes check and body derivation (release_body.py, pinned flowmark) passed offline; no paragraph-internal line breaks; GitHub's markdown render not run (needs gh).
Remaining for the maintainer: make release-preflight, make release-candidate, make release-body, then the go-ahead for step 6 (tag) onward.

## Notes

Release commit is 4450be7e58b1e87a3d2b0fc8ffc0e8646e5b7a09 (merge of #213, tree 2d461255), dated 2026-11-12. Done from the cloud session: step 1 (merged #213); step 2 (stability pass at the same tree, unprivileged: 14/15 with make check failing only at the provenance request the session proxy refuses, make -k check otherwise green, CI provenance green; root supplement 6/6, 16 kinds; records in draft PR #214); release/v0.6.0 re-pinned at 4450be7e; preflight lines without gh all ok (on main, versions 0.6.0, notes and CHANGELOG, tag absent, crates.io and PyPI unpublished); release body derived offline. Blocked for this session: dispatching release.yml (403 Resource not accessible by integration), reading the release environment and its secrets, the GitHub markdown render, release-demo, and environment approval. Not tagged: the standard process tags only after the rehearsal passes. Maintainer runs make release-preflight, make release-candidate, make release-body, then steps 6 to 12.
