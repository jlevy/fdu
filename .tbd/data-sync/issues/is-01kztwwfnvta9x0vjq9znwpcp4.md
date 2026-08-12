---
type: is
id: is-01kztwwfnvta9x0vjq9znwpcp4
title: Re-run the worker-depth curve after macOS bulk metadata (H52)
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvt1vamkqp8fffnpwhd93v
created_at: 2026-08-12T11:49:22.234Z
updated_at: 2026-08-12T12:04:44.179Z
closed_at: 2026-08-12T12:04:44.179Z
close_reason: Recorded as exp-025. Sixteen workers regressed 720k indexed wall 19.19%, producer wall 12.65%, CPU 107-117%, and RSS about 33%; the 6/8/12/16 exploratory curve found no retune above the acceptance bar. Current service-time calibration stays at six on the bulk backend, so no code change was required.
---
H26 removed the per-entry fstatat wait that created the pre-bulk sixteen-worker knee. Re-measure fixed 6/8/12/16-worker cold index and producer runs on the immutable 720k cache-pressure tree. Pre-registered expectation: six workers match or beat deeper pools with lower CPU/context switches/RSS, and current service-time adaptation remains inactive. If another depth improves wall by at least 3% without disproportionate resources, confirm it on 12 paired trials and retune; otherwise record the configuration result without code.
