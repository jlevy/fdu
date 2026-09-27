---
type: is
id: is-01kzwt2d8dapafh6tmf9f92gek
title: Extend comparative evidence to Linux
kind: task
status: open
priority: 2
version: 8
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - linux
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-08-13T05:38:39.500Z
updated_at: 2026-09-27T10:06:56.315Z
---
Repeat the claim-grade paired comparison on a controlled Linux local-SSD host using the portable backend, then add Linux numbers to the comparison report and white paper. Include exact binary/host/corpus provenance, the full oracle, pre/post fingerprint, a controlled or explicitly classified OS-cache state, and profile-backed follow-up hypotheses such as statx/getdents/io_uring only when measurements justify them. The Linux Comparison Rerun section in explorations/benchmarks/README.md provides the current warm-steady setup, seven supported peers, separate indexed and summary runs, and storage/evidence rules. Refresh current peer rankings before asserting whether the latest Linux build is faster or slower; earlier VM comparisons predate subsequent optimization.

## Notes

Queued after the M1/APFS report. Publish three separate matrices on a controlled Linux local-SSD host: verified warm after explicit full-tree warmups; dut-compatible pagecache-drop-only using echo 1 as a calibration regime; and controlled cold with successful sync plus per-sample echo 3. The kernel contract says echo 1 does not request dentry/inode slab reclamation, so never call that controlled cold. Run ext4 and XFS when available, sweep worker counts, and retain exact-binary, paired adjacency, pre/post fingerprint, semantic work-class, oracle, resource, and bootstrap-CI requirements. Profile before adopting statx/getdents/io_uring or queue changes; do not extrapolate APFS or warm rankings.

2026-09-27: warm-steady cell done on a 4-vCPU Firecracker KVM ext4 host, quiet, balanced 1M tree, seven peers, indexed and transient-summary matrices, 0 invalid samples: docs/project/reports/report-2026-09-27-fdu-linux-tool-comparison.md. Summary mode leads pdu +11% and diskus +16%; indexed tree trails them by 19%/18% (H155, fdu-578e). Still open here: bare-metal local SSD, pagecache-drop-only and controlled-cold regimes, XFS, worker sweep, dut (fdu-k5t5), and a frozen real tree.
