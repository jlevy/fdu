---
type: is
id: is-01m1xawdxr2v87f7km4jabmd8x
title: Make the default-command probe use the CLI's control scope
kind: bug
status: in_progress
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
  - type: blocks
    target: is-01m1xbp9qy40ymd7wvyaw0ckp8
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T07:03:53.015Z
updated_at: 2026-09-07T07:19:51.886Z
---
With gitignore enabled, profiling found default-tree retains control observations while the non-watch CLI sets ScanConfig.read_controls=false. The probe therefore measured a different scope. An exact-scope public cache-only open of its snapshot reproduced the mismatch; report-only projection deliberately cannot mask it in the regression test. Align only default-tree with the CLI and retain controls-enabled cold-index/opened coverage. Both measurement arms need the correction. Preserve the preliminary profile as controls-on report evidence, not CLI evidence; neither quiet-host attempt collected trials.

## Notes

Red public snapshot-scope test reproduced the mismatch before the fix. With only default-tree selecting read_controls=false, all 12 gitignore-enabled probe tests pass; the minimal probe remains separately tested. The same test and scope correction pass on the measurement-only structural control 1981747, whose engine remains c6380f7. Full isolated handoff gate is running; final timing has not begun. Earlier 1a39 profiles and provenance are preserved separately as pre-scope evidence.
