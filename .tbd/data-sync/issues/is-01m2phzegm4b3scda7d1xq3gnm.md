---
type: is
id: is-01m2phzegm4b3scda7d1xq3gnm
title: End-to-end verification of the final 0.1.0 release candidate
kind: task
status: in_progress
priority: 0
version: 8
delegate: claude-code@spud10
labels:
  - release
  - testing
  - runbook-verified
dependencies:
  - type: blocks
    target: is-01m2phzevyf68fdz9zcs3yzncw
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
  - type: blocks
    target: is-01m2phzkwcvz3fwk2nh150bdmj
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
hold: null
hold_until: null
created_at: 2026-09-17T02:08:52.755Z
updated_at: 2026-09-26T00:00:09.720Z
started_at: 2026-09-26T00:00:09.719Z
---
After the stabilization fixes merge: main CI green; dispatch the release rehearsal on main; download
every artifact and verify SHA256SUMS; run the end-to-end harness against the CI-built macOS arm64 wheel
(filesystem oracle, every view and format, YAML equals JSON, .gitignore selection, analysis sidecar
warm-versus-cold equality, all five cache policies, partial results and exit codes, usage errors,
ignore budget, watch create/modify/delete and Ctrl-C, Python/CLI parity, uv tool paths with an explicit
GIL interpreter, real trees ~/.rustup, ~/wrk, ~/Library/Application Support).

Baseline on the 5f2d36d rehearsal wheel (2026-09-16): 52 of 60 checks passed; failures were 4 harness
errors and the defects now tracked as fdu-gija, the Ctrl-C bead, the YAML contract bead, and uv's
free-threaded interpreter choice. Harness: scratch e2e_fdu.py (consider committing under scripts/release).

## Notes

2026-09-24 final candidate main b06a0201 (tree afe891a8; #119/#120/#122/#124 merged): uninterrupted make check + cross-lint passed on the identical tree (12de7373); CI 19/19; full path-independence matrix run 35979726461 passed on ubuntu, macOS, Windows. Release-candidate wheel (macOS arm64) installed as the user's global fdu 0.1.0-dev+g12de73735 and smoke-tested. The five-platform release rehearsal dispatch on claude/release-publish was refused by the auto-mode classifier; the maintainer dispatches it. Post-publication items remain.
