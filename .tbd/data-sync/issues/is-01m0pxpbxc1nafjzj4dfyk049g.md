---
type: is
id: is-01m0pxpbxc1nafjzj4dfyk049g
title: "Review fixes to PR #45: subject size floor, accept-vs-ranking split, per-host subject document"
kind: task
status: closed
priority: 1
version: 2
labels:
  - performance
  - campaign-2
dependencies: []
created_at: 2026-08-23T09:02:14.444Z
updated_at: 2026-08-23T09:07:27.013Z
closed_at: 2026-08-23T09:07:27.012Z
close_reason: "Merged in PR #45 (merge 778aa74): subject size floor, accept-vs-ranking split, per-host subject document, tail None-safety; host re-nominated with three deciding subjects."
---
Pre-merge review of PR #45 (campaign-2 instruments). Found: (1) the subject policy could be satisfied on paper -- a 5,838-entry cargo registry labelled source-checkout gave the set 'can decide' status; a 3-trial self-vs-self smoke on it returned ACCEPT -23.72%. Fix: MINIMUM_DECIDING_ENTRIES=50k and dense for a subject to decide; set-level split into accept (one deciding subject suffices, the campaign's literal rule) vs ranking (deciding subjects span 3 characters). (2) Makefile wrote every host's nominated-subjects document to one path; now keyed by host class (nominated-subjects-darwin-arm64.json). (3) _tail_spread formatted None with :.2f. Re-nominated this host: metabrowser clone (60k, source-checkout), /System/Library/PrivateFrameworks (159k, system-prefix, sealed, reconstructible), ~/.rustup (175k, package-cache), cargo registry src (5.8k, screens). Commit 9a3b5b1 on claude/perf-campaign-2-instruments.
