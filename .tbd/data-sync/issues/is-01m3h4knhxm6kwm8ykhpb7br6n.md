---
type: is
id: is-01m3h4knhxm6kwm8ykhpb7br6n
title: "Linux index tier: remove glibc cross-thread frees from the detached builder (H159)"
kind: task
status: in_progress
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - linux
dependencies:
  - type: blocks
    target: is-01m3kkrj5f6n9b38g6d1w6mrew
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-09-27T09:54:44.925Z
updated_at: 2026-09-28T13:59:35.361Z
---
The 2026-09-27 Linux comparison (docs/project/reports/report-2026-09-27-fdu-linux-tool-comparison.md) found fdu's indexed tree 19% behind pdu and 18% behind diskus on a 1M-entry tree, while summary mode leads both. The unchanged binary under LD_PRELOAD mimalloc/jemalloc/tcmalloc closes the whole gap (1.39 -> 1.11-1.13 s; summary 0.98 -> 0.82 s); glibc.malloc.arena_max=1 makes it 3.3 s. A context-switch profile names the blocking sites: the consumer freeing walker-allocated Vec<DetachedChild> buffers and directory_ids PathBuf keys into walker-owned arenas, plus walker-side PathBuf/file_name allocations waiting on arenas that glibc's tcache has mixed across threads. Candidates, dependency-free first: return drained child buffers to the producing worker (per-worker pools, batched); replace the PathBuf-keyed directory map with walker-assigned directory tokens carried in the queue claim; move the claimed rel_dir into DetachedDirectory instead of copying. Measure each under the accept rule on cold-scan-index and the product CLI job. An allocator dependency stays out unless a structural change cannot reach it (H74, H85 history).

## Notes

2026-09-28 review of #150 (https://github.com/jlevy/fdu/pull/150#issuecomment-5871425959): no blockers; merge when Linux accepts. Doc nits to land with the Linux result: (1) retained listings also keep their largest path capacity (bounded by the OS path limit, ~16 KiB/worker worst case on Linux) — document as a third retention dimension; (2) diagnostic send_ns now includes take-back work.
