---
type: is
id: is-01m1xawdxr2v87f7km4jabmd8x
title: Make the default-command probe use the CLI's control scope
kind: bug
status: closed
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
  - type: blocks
    target: is-01m1xbp9qy40ymd7wvyaw0ckp8
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T07:03:53.015Z
updated_at: 2026-09-07T07:32:08.680Z
closed_at: 2026-09-07T07:32:08.679Z
close_reason: Red-green regression tests, full isolated gate, cross-platform lint, and all 19 CI checks passed on the pushed formal stack.
resolution: null
duplicate_of: null
---
With gitignore enabled, profiling found default-tree retains control observations while the non-watch CLI sets ScanConfig.read_controls=false. The probe therefore measured a different scope. An exact-scope public cache-only open of its snapshot reproduced the mismatch; report-only projection deliberately cannot mask it in the regression test. Align only default-tree with the CLI and retain controls-enabled cold-index/opened coverage. Both measurement arms need the correction. Preserve the preliminary profile as controls-on report evidence, not CLI evidence; neither quiet-host attempt collected trials.

## Notes

Completed at 64c6e616f2d8d81eb99712bdd26a0e8a57bd2ed5. The red public snapshot-scope test reproduced the mismatch; all 12 gitignore-enabled and 11 minimal probe tests now pass. Full isolated make check, cross-lint, and all 19 GitHub CI checks passed (run34095392687). Both measurement arms have the scope correction; the structural measurement-only control is 1981747 with unchanged c6380f7 engine. Earlier 1a39 profiles/manifests are preserved and labelled pre-scope, not final CLI evidence. No final timing sample has been collected.
