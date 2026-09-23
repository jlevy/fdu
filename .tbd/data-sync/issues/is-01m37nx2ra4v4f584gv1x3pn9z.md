---
type: is
id: is-01m37nx2ra4v4f584gv1x3pn9z
title: Progress phases, analysis counter, and report and watch entry points
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
labels:
  - cli
  - ux
dependencies:
  - type: blocks
    target: is-01m37nx5g2ez5hcjrmar93ss7s
  - type: blocks
    target: is-01m37nxa8vshzngyvzw9ax67a0
parent_id: is-01m2nsj4zgw7r9j3d1rphnmwf2
created_at: 2026-09-23T17:44:34.822Z
updated_at: 2026-09-23T17:44:49.790Z
---
Add the Loading, Analyzing and Saving phases, the per-file analysis counter with its known candidate total (content_analysis result loop on the caller thread), prepare_report_with_progress(request, delivery, &progress) beside prepare_report_with_scan_diagnostics, and a progress argument on Session::start for the watch's initial scan. Delivery stays a plain value. Tests: phases in order per route, analysis (done, total) exact at completion.
