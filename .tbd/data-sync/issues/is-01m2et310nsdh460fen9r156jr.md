---
type: is
id: is-01m2et310nsdh460fen9r156jr
title: "PR #51 verification FIX51-1: Only-policy open after a report errors, but docs say it scans cold"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2eafpfpe8k5c9z9dhrqvy2y
created_at: 2026-09-14T01:56:43.156Z
updated_at: 2026-09-14T01:56:43.156Z
---
Verification of #51 fixes (a69b95e docs). Under CachePolicy::Only, an index-returning open (fdu --watch --cache only, fdu.open(cache=ONLY)) after a one-shot report's controls-off snapshot fails with 'no usable snapshot for this root and scan scope' (lib.rs:386-420 at 51154f9; snapshot_scope_serves only projects for ReportOnly). The new docs on open/fdu.open (lib.rs:299-301, _api.py:340-342) say it 'scans cold' without qualifying the policy. Behavior predates (6d45be9) and is the honest Only contract. Fix: qualify the docs by policy; make the Only error name the stored and wanted scopes; optionally add the mirror test (the other direction is covered by one_shot_reports_share_one_cache_whichever_surface_wrote_it).
