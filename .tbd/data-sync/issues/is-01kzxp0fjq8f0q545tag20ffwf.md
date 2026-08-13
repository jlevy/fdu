---
type: is
id: is-01kzxp0fjq8f0q545tag20ffwf
title: Audit critical CLI goldens after performance integration
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - golden-testing
dependencies: []
parent_id: is-01kzw3te81j66eehy48rx2djv5
created_at: 2026-08-13T13:46:56.471Z
updated_at: 2026-08-13T13:54:54.441Z
closed_at: 2026-08-13T13:54:54.440Z
close_reason: Audited the critical golden surface under tbd guidance, pinned actual bare-fdu help behavior and the realistic natural overview, verified current tryscript 0.2.0 is latest, and passed all 72 goldens.
---
Apply the tbd golden-testing principles to the integrated performance branch: pin the bare no-argument safety path, audit the realistic default overview and visualization, verify latest pinned tryscript, and run the complete golden suite.

## Notes

Applied golden-testing guidelines. Latest npm tryscript is 0.2.0 and the lock is 0.2.0. The concise realistic-project fixture covers 16 files, nested source/docs/tests/bench directories, full natural default output, ten-cell bars, alignment, ranking, counts, depth, and false-ellipsis prevention. Changed the complete help golden to invoke bare fdu itself; unit test proves it equals --help byte-for-byte. All 72 golden cases pass.
