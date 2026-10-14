---
type: is
id: is-01m4w1eqfrazpvdy16ckqc5skm
title: "stability_pass: count Phase 1's sanity flags under Phase 1, not Phase 2"
kind: bug
status: open
priority: 4
version: 1
labels: []
dependencies: []
created_at: 2026-10-14T01:46:57.911Z
updated_at: 2026-10-14T01:46:57.911Z
---
Review A9 on PR #203: scripts/release/stability_pass.py (around line 2098) maps the installed-CLI harness's 4 sanity checks (help, version, docs, skill) into Phase 2's count, so the summary table says 'Phase 2: 22 checks' where the playbook runs 4 of them in Phase 1 and 18 in Phase 2. The 0.5.0 record states the split by hand. Map each harness check to the playbook phase that runs it.
