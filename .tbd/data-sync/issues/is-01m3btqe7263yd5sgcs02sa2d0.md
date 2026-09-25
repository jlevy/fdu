---
type: is
id: is-01m3btqe7263yd5sgcs02sa2d0
title: "Release QA: peer agreement on real trees, ~/Library included"
kind: task
status: in_progress
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m2s053k7cjccm6c6c4rwck2f
created_at: 2026-09-25T08:25:50.561Z
updated_at: 2026-09-25T08:25:52.771Z
---
Release QA phase: compare fdu's totals on real trees (this repo, ~/.rustup, /Applications, ~/Library) with GNU du (--count-links as the byte-exact reference), dust, pdu, dua, diskus, and BSD du; every difference named (hard links, symbolic links, directory sizes, drift). Script scripts/qa_peer_agreement.py and playbook Phase 7 on branch claude/qa-peer-agreement; first run in progress 2026-09-25.
