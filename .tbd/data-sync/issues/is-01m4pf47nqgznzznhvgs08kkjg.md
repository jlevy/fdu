---
type: is
id: is-01m4pf47nqgznzznhvgs08kkjg
title: Stabilize and merge the 0.5.0 stack and prepare release readiness
kind: task
status: in_progress
priority: 1
version: 3
delegate: codex@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-11T21:50:27.510Z
updated_at: 2026-10-11T23:22:41.275Z
started_at: 2026-10-11T21:50:41.640Z
---

## Notes

2026-10-11 stabilization: independently reviewed PRs #191, #192, #194, and #196 merged atomically via stack #193 as d59e6531; its tree equals fully gated f084d9a3. Reviewed README cleanup #195 merged as 3d3f31f3. Full local make check and all five cross-lint targets passed on f084d9a3; 20 CI checks passed on each PR head. Review findings dispositioned; research deferrals retain their own beads. Release prep #197 remains draft at d0862487, tree 67e3ca0f6475d371cec13465569dadc9bdbe3216, with senior/security review complete and 20 CI checks passing. Full release-stability is running from a clean worktree. Merging current main and this candidate produces that same tree. Remaining: final installed-candidate correctness and peer QA, stability record, merge #197, non-publishing GitHub rehearsal/preflight, and pre-tag disposition of fdu-4arl plus the human terminal-window phase. Tag/publication not authorized. The user approved the regenerated color demo; its MP4 and GIF are preserved outside scratch.
