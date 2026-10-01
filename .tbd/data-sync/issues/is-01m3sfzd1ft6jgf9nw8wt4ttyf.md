---
type: is
id: is-01m3sfzd1ft6jgf9nw8wt4ttyf
title: Make the release stability pass one reproducible command
kind: task
status: closed
priority: 1
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-30T15:47:19.215Z
updated_at: 2026-10-01T03:27:21.025Z
started_at: 2026-10-01T03:27:20.246Z
closed_at: 2026-10-01T03:27:21.024Z
close_reason: "Merged in PR #170 (main 7f2f268b): make release-stability, the pty probe and the two break wrappers are committed and tested, and the review's 12 findings are fixed. Not yet run through the driver: make check, cross-lint and semver-check, tracked as fdu-ibmc."
resolution: null
duplicate_of: null
---
The 0.3.0 stability pass (docs/project/reports/report-2026-09-30-release-0.3.0-stability-pass.md) needed six hand-written scripts that each pass rewrites: the gate sequence with per-gate logs and exit statuses, the Phase 6 pty probe (38 checks), the ext4 top-level walk and self-test replica that explain qa_peer_agreement.py's known exit (fdu-83km), and the two correctness break wrappers (no snapshot stored; partial answer stored). Commit them as tested tooling behind one entry point (for example make release-stability, driven by scripts/release/stability_pass.py) that runs the gates in a clean worktree with its own target directory, builds and installs the candidate wheel, runs the QA harness, peer agreement, pty probe, the three correctness passes and both breaks, fails fast on a missing prerequisite such as GNU time, and writes the dated report and the two summary tables with private paths replaced by labels. Fix fdu-83km so the peer script passes on ext4 instead of needing the separate walk. Document it in release-process.md's Stability Pass and the QA playbook.
