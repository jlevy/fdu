---
type: is
id: is-01m76mhgvsr7fg5vn9d0vhs21p
title: "Release 0.6.0: tag, publish, and announce"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-12T01:04:20.344Z
updated_at: 2026-11-12T18:15:58.374Z
---
Release commit 8a8febc7cf332dfd4ee8c8536dd6cc00128be9dd (merge of #209, tree 64a7d366). Done by the cloud session on 2026-11-12, reversible steps only:
- Step 2 stability: the #209 Linux host gate pass at 98784661 (same tree) stands by tree identity; make check's only failure was its provenance request refused by the session proxy (rest passed under -k; CI green at that commit). Root supplement on 8a8febc7 (candidate + correctness, refusal as nobody, 16 kinds): all passed. Records in draft PR #210.
- Step 3 preflight: ok for COMMIT on main, Cargo versions, notes, CHANGELOG (2026-11-12), tag absent, crates.io and PyPI unpublished, vulnerability reporting, signing key; demo video, release environment, and registry secrets could not be read through the session's GitHub integration (403) and need the maintainer's gh.
- Step 4: release/v0.6.0 pinned at 8a8febc7 on origin; dispatching the rehearsal was refused to the session's integration (403), so run make release-candidate (it accepts the existing pin).
- Step 5: notes check and body derivation (release_body.py, pinned flowmark) passed offline; no paragraph-internal line breaks; GitHub's markdown render not run (needs gh).
Remaining for the maintainer: make release-preflight, make release-candidate, make release-body, then the go-ahead for step 6 (tag) onward.

## Notes

Re-cut 2026-11-12 to include #211: release PR #213 at bf4981b9 (tree 2d461255). Stability pass on that tree, unprivileged: 14/15, check failing only at the provenance request the session proxy refuses; make -k check otherwise green; root supplement 6/6, 16 kinds; CI 20/20. Review A addressed. Release body derived offline (no GitHub render). After merge: confirm the merge tree is 2d461255, delete and re-pin release/v0.6.0 at the merge commit (now at the old 8a8febc7), record the pass in a records PR, then the maintainer runs release-preflight, release-candidate, release-body, and decides the tag.
