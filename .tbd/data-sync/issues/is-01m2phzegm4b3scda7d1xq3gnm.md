---
type: is
id: is-01m2phzegm4b3scda7d1xq3gnm
title: End-to-end verification of the final 0.1.0 release candidate
kind: task
status: closed
priority: 0
version: 10
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
updated_at: 2026-09-26T00:50:50.878Z
started_at: 2026-09-26T00:00:09.719Z
closed_at: 2026-09-26T00:50:50.877Z
close_reason: Final candidate, CI wheel, five-platform artifacts, peer totals, and handoff gates verified
resolution: null
duplicate_of: null
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

2026-09-25 final candidate 7cf7f1b4b (signed v0.1.0). Main was clean, make check, cross-lint, docs-format, and make release-rehearse passed on this commit. The five-platform release rehearsal 36203963537 passed; all eight retained files (two crates, sdist, five wheels) passed SHA256SUMS verification. The CI-built macOS arm64 wheel was installed with an explicit CPython 3.12 interpreter and reported fdu 0.1.0. Installed CLI QA: 51 checks, 0 failures, 1 expected warning for documents without analysis; terminal PTY tests 3/3 and live medium-tree terminal run completed cleanly. A temporary fixture confirmed all five cache policies give the same summary, .gitignore selection is honored, and Python/CLI summary output agrees. Peer agreement self-test: 13/13 exact; real-tree comparison on the repository, rustup, Applications, and Library: all 52 readings explained, with no unexplained difference. The clean tag clone passed resolve_plan --validate-checkout. Detailed local transcripts are retained in external release scratch; do not commit their private paths.
