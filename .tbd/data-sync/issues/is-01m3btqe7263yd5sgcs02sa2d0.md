---
type: is
id: is-01m3btqe7263yd5sgcs02sa2d0
title: "Release QA: peer agreement on real trees, ~/Library included"
kind: task
status: closed
priority: 1
version: 4
labels: []
dependencies: []
parent_id: is-01m2s053k7cjccm6c6c4rwck2f
created_at: 2026-09-25T08:25:50.561Z
updated_at: 2026-09-25T13:33:44.185Z
closed_at: 2026-09-25T13:33:44.174Z
close_reason: "Done in PR #126, merged as 87d2b663 on 2026-09-25 after four review rounds: self-test exact for all 13 tool readings; quiet trees exact (39 readings); ~/Library within fdu's bracketing readings or short by measured skipped folders, one dua reading not verifiable. make check and CI green."
resolution: null
duplicate_of: null
---
Release QA phase: compare fdu's totals on real trees (this repo, ~/.rustup, /Applications, ~/Library) with GNU du (--count-links as the byte-exact reference), dust, pdu, dua, diskus, and BSD du; every difference named (hard links, symbolic links, directory sizes, drift). Script scripts/qa_peer_agreement.py and playbook Phase 7 on branch claude/qa-peer-agreement; first run in progress 2026-09-25.

## Notes

2026-09-25: PR #126 (claude/qa-peer-agreement). First run on main 963d4852: four trees, no unexplained difference; quiet trees byte-exact vs GNU du --count-links; ~/Library within bracketing fdu readings; fdu 153 unreadable dirs = GNU du 153 denied. Finding: GNU du and pdu on macOS skip a subtree when a directory read is interrupted (cost GNU du 1.4 GiB in one run). Report: docs/project/reports/report-2026-09-25-peer-agreement.md. Review running.
