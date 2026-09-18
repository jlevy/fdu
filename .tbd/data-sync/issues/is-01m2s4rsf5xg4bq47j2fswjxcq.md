---
type: is
id: is-01m2s4rsf5xg4bq47j2fswjxcq
title: Characterize this virtualized Linux host with the full performance protocol
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-09-18T02:15:46.405Z
updated_at: 2026-09-18T02:39:12.405Z
closed_at: 2026-09-18T02:39:12.405Z
close_reason: |
  Completed on this virtualized Linux cloud-agent (4 Xeon logical cores, 15 GiB, ext4, Linux 6.12.94+). Product binary origin/main 98379c76 (fdu 0.1.0-dev+g98379c76b). Regime: exploratory; tool/probe matrices uncontrolled; floor quiet. os_cache warm-steady. Not claim-grade for fdu-nffc (no controlled-cold; guest drop_caches cannot reach the hypervisor).

  make test-performance: 15+71+307 after moving the uv-tool fdu out of login PATH (bash -lic otherwise shadows the fixture via ~/.profile).

  Headline 1,000,001-entry balanced corpus (semantic digest 4bbd97c0d3d4e2ad, same recipe as 2026-09-16): CLI indexed-tree median 0.892 s / 332.6 MiB. diskus 0.600 s (-32.5%), pdu 0.566 s (-37.2%), dua 1.016 s (+12.4%), gnu-du 1.469 s (+65.7%). dust oracle-invalid on all 15 samples (allocated 3498745856 vs 2986741760). fdu tree vs index-summary vs transient-summary: 0.883 / 0.890 / 0.898 s, all ~330 MiB — transient contract still retains the index (fdu-hkyh / fdu-if7o).

  Floor (quiet, 4 workers): usr-share index 2.23x and aggregate 2.53x parfloor-stat; balanced-100k 1.76x / 1.91x. Both miss campaign-2 x-floor thresholds.

  Artifacts under /tmp/fdu-realtree and /tmp/fdu-tool-comparison. Do not republish the ledger from this host.
---

## Notes

Completed on this virtualized Linux cloud-agent (4 Xeon logical cores, 15 GiB, ext4, Linux 6.12.94+). Product binary origin/main 98379c76 (fdu 0.1.0-dev+g98379c76b). Regime: exploratory; tool/probe matrices uncontrolled; floor quiet. os_cache warm-steady. Not claim-grade for fdu-nffc (no controlled-cold; guest drop_caches cannot reach the hypervisor).

make test-performance: 15+71+307 after moving the uv-tool fdu out of login PATH (bash -lic otherwise shadows the fixture via ~/.profile).

Headline 1,000,001-entry balanced corpus (semantic digest 4bbd97c0d3d4e2ad, same recipe as 2026-09-16): CLI indexed-tree median 0.892 s / 332.6 MiB. diskus 0.600 s (-32.5%), pdu 0.566 s (-37.2%), dua 1.016 s (+12.4%), gnu-du 1.469 s (+65.7%). dust oracle-invalid on all 15 samples (allocated 3498745856 vs 2986741760). fdu tree vs index-summary vs transient-summary: 0.883 / 0.890 / 0.898 s, all ~330 MiB — transient contract still retains the index (fdu-hkyh / fdu-if7o).

Floor (quiet, 4 workers): usr-share index 2.23x and aggregate 2.53x parfloor-stat; balanced-100k 1.76x / 1.91x. Both miss campaign-2 x-floor thresholds.

Artifacts under /tmp/fdu-realtree and /tmp/fdu-tool-comparison. Do not republish the ledger from this host.
