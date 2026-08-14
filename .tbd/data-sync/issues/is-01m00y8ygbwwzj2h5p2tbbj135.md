---
type: is
id: is-01m00y8ygbwwzj2h5p2tbbj135
title: Clean up abandoned fdu RAM disks and document their lifecycle
kind: task
status: closed
priority: 1
version: 4
labels:
  - performance
dependencies: []
created_at: 2026-08-14T20:09:05.802Z
updated_at: 2026-08-14T20:30:17.321Z
closed_at: 2026-08-14T20:30:17.320Z
close_reason: "Audited and detached both abandoned RAM-backed images, preserved the dirty H69 prototype in Git, pruned the stale worktree registration, documented the one-image exception and cleanup lifecycle, and verified local make check plus all PR #22 CI checks."
---
Audit FDUCodexTarget and FDUCodexTemp, preserve any unique or dirty work, deregister RAM-backed worktrees, detach the volumes to release RAM and volume-local Trash, and document when temporary volumes are justified plus a cleanup procedure that prevents abandoned mounts.

## Notes

Audit found two day-old ram:// images: a 6 GiB HFS+ Cargo/experiment volume at 5.8 GiB used (4.4 GiB in volume-local Trash) and a 1 GiB APFS temp volume at 731 MiB physical use (1.1 GiB logical Trash because of clones). No persistent writer remained. Preserved the only dirty source work, the rejected H69 two-file opener prototype, as stash commit 7497124c0d8d5155c8a9e08b065926d3bbee205e. Experiment outcomes were already recorded in exp-045 and exp-046; other contents were rebuildable targets, caches, test temp data, or already-trashed artifacts. Detached both images without force, dry-ran and pruned only the stale RAM-volume worktree registration, and verified hdiutil has no attached images. Internal Data availability rose from about 3.2 GiB to 8.7 GiB as RAM and swap pressure fell. Added the default prohibition, exception criteria, preflight, preservation, detach, and interruption-recovery lifecycle to AGENTS.md and the performance-loop guide.
