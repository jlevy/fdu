---
type: is
id: is-01m1wxpmpmqycpjvgcav4daxp2
title: Final merge-readiness review of PR 52
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T03:13:31.859Z
updated_at: 2026-09-07T03:31:21.342Z
closed_at: 2026-09-07T03:31:21.337Z
close_reason: Completed final merge-readiness review with two reproduced validation bugs, documentation/acceptance cleanup, passing existing gates, and explicit remaining parity/stack requirements. Findings tracked in fdu-9o4u, fdu-dtb6, and fdu-b49n; campaign remains open.
resolution: null
duplicate_of: null
---
Review 5d7b86f against PR 51, validate correctness and performance evidence, assess maintainability and backend boundaries, run isolated gates, and track actionable findings.

## Notes

Final review of PR52 at 5d7b86f against PR51: request changes. R1 fdu-9o4u fixes the opened-discovery oracle; reproduced by dropping one of two fixture files from the actual opened stream while the attached detached-scan digest still passed. R2 fdu-dtb6 fixes allocation guards that reject lower allocations; actual helper failed a one-allocation-per-entry improvement. R3 fdu-b49n reconciles +3/+5 percent criteria and final documentation. Historical final-binary parity remains open in fdu-lj4h; exp101 is exploratory against an intermediate control. Architecture direction is sound: one reusable Index, private compact child storage and local promotion, shared statically dispatched walker, no new public backend restriction. An additional 10-step differential mutation trace at workers1/4 passed directory/file replacement, nested removals, reinsertions, metadata updates, control reclassification/removal, invalidations, ordering, per-directory partition rollups and exact commits. Existing CI is 19/19 green. Isolated make check passed through Python concurrency; wheel smoke initially selected free-threaded Python and failed the standard ABI wheel. Rerunning python-smoke, python-sdist-smoke, parity-check and release-test with explicit standard Python3.14 passed. Apple/Windows cross-lint passed. Initial in-place gate saw an unrelated ignored nested Claude worktree and was not a PR defect. Formal stack53 exists, but PR50 lacks the latest PR48 pagination fix and needs rebase before integrated merge. No production code or PR metadata changed; main and isolated source checkouts are clean. Review-only test/fault-injection patch and logs preserved under /tmp/fdu-pr52-review-*.
