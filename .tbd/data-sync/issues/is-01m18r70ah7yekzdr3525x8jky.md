---
type: is
id: is-01m18r70ah7yekzdr3525x8jky
title: "~/Library scan is SIGKILLed (137): unbounded growth the control cap does not govern"
kind: bug
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - scale
  - macos
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:12:47.952Z
updated_at: 2026-08-30T07:12:47.952Z
---
Field report: 'fdu ~/Library -d 2 -n 30 --sort size --min-size 300M' exited 137 (SIGKILL) on the branch binary. This is a different failure mode from the control-table aborts - the OS killed it rather than fdu refusing cleanly - which points at growth the control budget does not bound.

Partially reproduced, NOT confirmed as OOM: on this machine './target/release/fdu ~/Library' (main build) exceeded 10 minutes and was killed by timeout rather than by the OS. Progressive depth probe on main: --scan-depth 1 = 0.036s, depth 2 = 0.14s, depth 3 = >300s.

Isolated the slow subtree: ~/Library/Containers (1012 sandbox containers). IMPORTANT - this one is not fdu's fault: 'du -sh ~/Library/Containers' and fdu both time out at 60s, so that subtree is hostile to every tool (TCC permission checks per container). Do not chase it as an fdu perf bug.

What remains genuinely open is the memory behaviour: why a SIGKILL rather than a slow scan. Reporter's host was at 95-99% disk during testing, so memory pressure is a confound to control for.

Acceptance: establish whether peak RSS grows unbounded with entry count on ~/Library-shaped trees (deep, wide, many small files); if so, identify what accumulates and bound it; distinguish that from TCC-induced slowness, which is out of scope.
