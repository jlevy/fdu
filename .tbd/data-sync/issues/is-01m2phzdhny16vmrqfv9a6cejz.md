---
type: is
id: is-01m2phzdhny16vmrqfv9a6cejz
title: Correct the release runbook, packaging spec, and plan specs for 0.1.0
kind: task
status: open
priority: 0
version: 2
labels:
  - release
  - docs
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T02:08:51.764Z
updated_at: 2026-09-17T02:08:52.755Z
---
From the 2026-09-16 audits. Release runbook: SSH-signed tag setup and `git tag -v && git push`; name
recheck before the push; final PyPI check with `--no-build --python 3.12` (a free-threaded default
interpreter otherwise builds the sdist and passes without testing a wheel); partial-upload/API-lag
case in recovery; trusted publishers only after a protected environment exists; post-publish checks
(docs.rs, crates.io and PyPI pages); private vulnerability reporting prerequisite; reproduced crate
digests. Packaging spec: two crates, hand publication, waived Phase-1 blockers. Plan specs: status
lines and stale statements across phase-1, rust-quality, e2e-perf, composable CLI and CLI-UX (move to
done), post-phase-1 roadmap, campaign-2, opened-root engine, streaming parity, checkpoints,
experiment loop, evidence scope, progressive results, content-metrics and cache-layers (done
amendments), output contract. Shipped-doc accuracy: README provenance claim, exit status wording,
summary-floor qualification, depth-one rustdoc, JSON integer precision note.

Being addressed by the documentation PR (branch claude/release-docs-accuracy).
