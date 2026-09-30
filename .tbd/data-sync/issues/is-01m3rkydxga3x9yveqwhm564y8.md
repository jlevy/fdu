---
type: is
id: is-01m3rkydxga3x9yveqwhm564y8
title: Summary at sub-listing batch sizes probes a search-denied directory's control and answers unlike the full index
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3qgck5yzpd603akhhkpw7t9
created_at: 2026-09-30T07:37:27.216Z
updated_at: 2026-09-30T07:50:23.790Z
closed_at: 2026-09-30T07:50:23.790Z
close_reason: "Not reachable once fdu-o5gx is fixed: the probe in StreamingEmission::settle_listing fires only when a batch fills inside a listing, and a search-denied directory records no entry (every child's stat fails before record_entry), so its listing never fills a batch and is never probed on either route. Verified: compact_summary_equals_the_indexed_summary_under_every_control_case passes the search-denied case at batch sizes 1024, 1 and 3 as an unprivileged user; the case is compared at every batch size."
resolution: null
duplicate_of: null
---
StreamingEmission::settle_listing probes a directory's .gitignore directly when a batch fills inside its listing (batch sizes below a listing, which only a library caller can set through ScanConfig::batch_size). For a directory that is readable but not searchable (mode 0400) the probe's stat of denied/.gitignore fails with EACCES, so the compact summary records an unreadable control and an error the whole-listing arm, which listed the directory and saw no control, does not, and the full index never probes. Found while adding the search-denied control case to compact_summary_equals_the_indexed_summary_under_every_control_case (fdu-o5gx); that case is compared at whole-listing batch sizes only (ControlCase::whole_listings) until this is settled: either the probe treats a listing that showed no control as definitive, or the exception is classified.
