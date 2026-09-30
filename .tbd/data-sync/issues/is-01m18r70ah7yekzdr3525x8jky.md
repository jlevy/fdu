---
type: is
id: is-01m18r70ah7yekzdr3525x8jky
title: "~/Library scan is SIGKILLed (137): unbounded growth the control cap does not govern"
kind: bug
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - scale
  - macos
  - stack-followup
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:12:47.952Z
updated_at: 2026-09-30T04:06:23.768Z
---
Field report: 'fdu ~/Library -d 2 -n 30 --sort size --min-size 300M' exited 137 (SIGKILL) on the branch binary. This is a different failure mode from the control-table aborts - the OS killed it rather than fdu refusing cleanly - which points at growth the control budget does not bound.

Partially reproduced, NOT confirmed as OOM: on this machine './target/release/fdu ~/Library' (main build) exceeded 10 minutes and was killed by timeout rather than by the OS. Progressive depth probe on main: --scan-depth 1 = 0.036s, depth 2 = 0.14s, depth 3 = >300s.

Isolated the slow subtree: ~/Library/Containers (1012 sandbox containers). IMPORTANT - this one is not fdu's fault: 'du -sh ~/Library/Containers' and fdu both time out at 60s, so that subtree is hostile to every tool (TCC permission checks per container). Do not chase it as an fdu perf bug.

What remains genuinely open is the memory behaviour: why a SIGKILL rather than a slow scan. Reporter's host was at 95-99% disk during testing, so memory pressure is a confound to control for.

Acceptance: establish whether peak RSS grows unbounded with entry count on ~/Library-shaped trees (deep, wide, many small files); if so, identify what accumulates and bound it; distinguish that from TCC-induced slowness, which is out of scope.

## Notes

2026-09-14 (triage at c0511e9): no code change to cite; the query retains a full index (`execution.rs:176-181@c0511e9`) on both main and the stack. Needs an RSS-slope measurement on a quiet host at three fixture sizes, both binaries, before it can be attributed to the stack or closed.

2026-09-18 installed CLI QA (fdu 0.1.0-dev+gcb9666a2a): bounded ~/Library --view=summary --scan-depth=1 completed 0.08s / 23 MiB RSS exit 0; --scan-depth=2 completed 0.23s / 27 MiB RSS exit 2 (26 TCC warnings). Preferences and Logs at --scan-depth=2 --depth=1 --limit=10 also exit 0. Did not reproduce SIGKILL on these bounded commands. Unbounded full-Library scan still not retested.

2026-09-30 stability pass (claude/stability-fixes, Linux): code read at b1376507 for growth no cap governs. The reporter's command carries --min-size 300M, and execution::TreeRetention::for_request returns None for any selection that is not unfiltered (also for a metric sort or a narrowed population), so the run planned RetainedState::FullIndex rather than the folded tree the default fdu PATH now builds: every entry of ~/Library is retained (about 300 B each plus its name), which on a Library of a few million entries is gigabytes, the only bound is the entry count, and neither the control budget nor the 62 MiB folded-tree figure in the CHANGELOG applies to it. That is a concrete candidate for the SIGKILL on a host at 95-99% disk (swap-starved). Not reproduced on Linux here: a generated tree large enough to exhaust this container's memory was not attempted on the shared host. A fix would fold a filtered tree the way the unfiltered one is folded (keep every directory, count files into roll-ups, keep only the files the selection and share threshold can show), which is a design change to the transient plan rather than a bug fix; TCC-induced slowness under ~/Library/Containers is separate, as the notes say.
