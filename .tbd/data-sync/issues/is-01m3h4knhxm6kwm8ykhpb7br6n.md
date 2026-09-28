---
type: is
id: is-01m3h4knhxm6kwm8ykhpb7br6n
title: "Linux index tier: remove glibc cross-thread frees from the detached builder (H159)"
kind: task
status: in_progress
priority: 1
version: 12
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - linux
dependencies:
  - type: blocks
    target: is-01m3kkrj5f6n9b38g6d1w6mrew
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-27T09:54:44.925Z
updated_at: 2026-09-28T22:53:27.942Z
---
The 2026-09-27 Linux comparison (docs/project/reports/report-2026-09-27-fdu-linux-tool-comparison.md) found fdu's indexed tree 19% behind pdu and 18% behind diskus on a 1M-entry tree, while summary mode leads both. The unchanged binary under LD_PRELOAD mimalloc/jemalloc/tcmalloc closes the whole gap (1.39 -> 1.11-1.13 s; summary 0.98 -> 0.82 s); glibc.malloc.arena_max=1 makes it 3.3 s. A context-switch profile names the blocking sites: the consumer freeing walker-allocated Vec<DetachedChild> buffers and directory_ids PathBuf keys into walker-owned arenas, plus walker-side PathBuf/file_name allocations waiting on arenas that glibc's tcache has mixed across threads. Candidates, dependency-free first: return drained child buffers to the producing worker (per-worker pools, batched); replace the PathBuf-keyed directory map with walker-assigned directory tokens carried in the queue claim; move the claimed rel_dir into DetachedDirectory instead of copying. Measure each under the accept rule on cold-scan-index and the product CLI job. An allocator dependency stays out unless a structural change cannot reach it (H74, H85 history).

## Notes

2026-09-28 review of #150 (https://github.com/jlevy/fdu/pull/150#issuecomment-5871425959): no blockers; merge when Linux accepts. Doc nits to land with the Linux result: (1) retained listings also keep their largest path capacity (bounded by the OS path limit, ~16 KiB/worker worst case on Linux) — document as a third retention dimension; (2) diagnostic send_ns now includes take-back work.

2026-09-28 (Linux session, epic fdu-92hp + fdu-k1n8): running the deciding Linux cell here. Host: 4-vCPU Firecracker KVM guest, ext4, Linux 6.18.44 (same class as the 2026-09-27 comparison). Probes: control main 0d73ed54 (engine identical to 45c7c577), candidate aa58a6b1 (branch claude/fdu-alternatives-research-qx0xn0 merges main into it; no crates/ change from main). Pre-registered as in exp-167: default-tree primary, cold-scan-index, 12 quiet pairs, linux-v6.12 deciding (reconstructed at adc21867, 92,474 entries), linux-balanced-1m screening, fdu-default-tree product contract in the tool harness, peak RSS non-inferior (upper bound <= +5%). Id: exp-188. Pre-cell screen (hyperfine, sequential, not a verdict) on balanced-1M: indexed tree main 1.61/1.55 s vs H159 1.39/1.34 s over two passes.

2026-09-28 exp-188 (quiet, 12 pairs): linux-v6.12 default-tree -2.19% [-4.50%, +1.08%] REJECT, cold-scan-index +5.27% [-4.28%, +8.81%], RSS non-inferior; linux-balanced-1m screening default-tree -10.63% [-13.00%, -8.23%], cold-scan-index -7.23% [-8.27%, -5.05%], RSS -5%; product fdu-default-tree main +10% [+8%, +15%] (balanced), +2% [-6%, +6%] (linux-v6.12). Reading: on linux-v6.12 the default tree was ~85% .gitignore classification on the consumer (H162/H163), which masked H159. Pre-registering exp-189 before running: same hypothesis, job (default-tree primary, cold-scan-index), subject (linux-v6.12 deciding, balanced screening) and rule (-3%, interval below zero, RSS upper bound <= +5%), on the H162+H163 base: control = main 0d73ed54 + H162 + H163 (scratch eb00edf8), candidate = a15b20f4 (H159 + H162 + H163).

2026-09-28 exp-189 (quiet): on the H162+H163 base H159 is again rejected on linux-v6.12: default-tree +2.26% [-5.33%, +12.96%], cold-scan-index +3.22% [+1.21%, +12.87%]. Recorded as exp-188 and exp-189 (rejected). Balanced-1m screening -10.63%. Merge decision for #150 left to the maintainer.

2026-09-28, pre-registered before any timing (exp-190): the H159 mechanism saves per directory (~1.25 us each on balanced-1m), so a fair real deciding subject must be directory-dense. Subject node-modules-dense: react-scripts 5.0.1 + gatsby 5.13.7 installed with npm ci --ignore-scripts (node 22.22.2, npm 10.9.7), 79,953 entries, 9,438 directories (8.5 per directory), reconstructible from the package.json and package-lock.json to be committed as exp-190 evidence. Same binaries, jobs and rule as exp-189: control eb00edf8 (main + H162 + H163), candidate a15b20f4 (H159 + H162 + H163); default-tree primary, cold-scan-index; 12 quiet pairs; accept at -3% with the interval below zero, peak RSS upper bound <= +5%. Asked for by the maintainer's request for a first-principles decision.
