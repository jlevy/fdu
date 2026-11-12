---
type: is
id: is-01m76mhgvsr7fg5vn9d0vhs21p
title: "Release 0.6.0: tag, publish, and announce"
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
hold: null
hold_until: null
created_at: 2026-11-12T01:04:20.344Z
updated_at: 2026-11-12T22:34:38.778Z
started_at: 2026-11-12T22:34:38.107Z
closed_at: 2026-11-12T22:34:38.778Z
close_reason: "0.6.0 tagged, published to crates.io and PyPI, and announced on GitHub; record in PR #214"
resolution: null
duplicate_of: null
---
Release commit 8a8febc7cf332dfd4ee8c8536dd6cc00128be9dd (merge of #209, tree 64a7d366). Done by the cloud session on 2026-11-12, reversible steps only:
- Step 2 stability: the #209 Linux host gate pass at 98784661 (same tree) stands by tree identity; make check's only failure was its provenance request refused by the session proxy (rest passed under -k; CI green at that commit). Root supplement on 8a8febc7 (candidate + correctness, refusal as nobody, 16 kinds): all passed. Records in draft PR #210.
- Step 3 preflight: ok for COMMIT on main, Cargo versions, notes, CHANGELOG (2026-11-12), tag absent, crates.io and PyPI unpublished, vulnerability reporting, signing key; demo video, release environment, and registry secrets could not be read through the session's GitHub integration (403) and need the maintainer's gh.
- Step 4: release/v0.6.0 pinned at 8a8febc7 on origin; dispatching the rehearsal was refused to the session's integration (403), so run make release-candidate (it accepts the existing pin).
- Step 5: notes check and body derivation (release_body.py, pinned flowmark) passed offline; no paragraph-internal line breaks; GitHub's markdown render not run (needs gh).
Remaining for the maintainer: make release-preflight, make release-candidate, make release-body, then the go-ahead for step 6 (tag) onward.

## Notes

Released 2026-11-12 from the cloud session, every step with gh on the direct GitHub channel (tbd shortcut setup-github-cli, Proxied Remote Sessions). Release commit 4450be7e58b1e87a3d2b0fc8ffc0e8646e5b7a09 (merge of #213, tree 2d461255). Stability pass at bf4981b9 (same tree): every step passed, make check on its second run reaching GitHub directly; root supplement 6/6. Tag v0.6.0 = annotated tag object 0fbba725, unsigned, release-verify-tag all ok. Preflight 13/13 ok. Rehearsal run 43314068874 green (publishing-path jobs skipped), eight files verified. release-body ok. release-demo attached fdu-demo.mp4 (reused from 0.5.0; first attempt lost a read-after-create race, fdu-87iu). Publishing run 43318821657: every job green on attempt 1; release environment approved via API on the maintainer's go-ahead. GitHub release fdu 0.6.0 published 22:29:01Z, immutable, latest, 12 assets. release-published: all registries identical (first attempt hit an artifact-storage 503). release-announced --cargo: all ok. After-publishing checks (crates.io READMEs and links, PyPI page, --install-skill) passed. release-cleanup deleted release/v0.6.0 (git needed gh's credential on the direct channel). Phase 6 manual window/Windows QA and the macOS checks did not run; decided by the maintainer's go-ahead. Record: docs/project/reports/report-2026-11-12-release-0.6.0-stability-pass.md (Release Record), PR #214.
