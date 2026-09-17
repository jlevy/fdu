---
type: is
id: is-01kzypf1yd2v4g8q8tk2v1xmxs
title: Implement or explicitly defer content analysis in watch mode
kind: bug
status: open
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - release
dependencies: []
parent_id: is-01m2pmrcrmjrm62bm1x3mxwgvm
created_at: 2026-08-13T23:14:08.460Z
updated_at: 2026-09-17T02:57:28.107Z
---
The CLI now explicitly rejects enabled content analysis with --watch, the Python watch feed remains metadata-only, and user-facing docs call content analysis one-shot. Implement incremental content reanalysis on metadata deltas before claiming full mode composability.

## Notes

2026-09-15 ~00:30 PDT status:
- Open PRs: GitHub stack #59 (#56 -> #57 -> #60), plus #58 and #55, which stand alone on main. Every one is 19/19 green and CLEAN. #58 and #55 are independent of the stack and merge cleanly with it and with each other, so they are deliberately not stacked.
… [46 lines omitted]

2026-09-17: Release blocker. Confirmed: fdu.open(..., analysis=lines).watch() serves the metrics the index
opened with, marked fresh. Fix on claude/release-e2e-fixes: watch_session::Session::new returns
Error::UnsupportedScanConfig for an index with a content tier (Python raises InvalidArgumentError), as
the CLI already refuses --analyze with --watch; Rust integration test and wheel smoke test added.
