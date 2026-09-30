---
type: is
id: is-01m3q1w6yqkp9ec5nk7gdrwarh
title: Revise the README and user docs against the current code after the Linux round
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
hold: null
hold_until: null
created_at: 2026-09-29T17:02:25.751Z
updated_at: 2026-09-30T00:01:10.404Z
started_at: 2026-09-29T17:03:18.327Z
closed_at: 2026-09-30T00:01:10.403Z
close_reason: "README and user docs revised against the current code in PR #162 (Speed rewrite, SKILL.md, usage.md, engine architecture, crate READMEs), reviewed and fixed; ready to merge after #161."
resolution: null
duplicate_of: null
---
Stacked layer on #161 with the comparison matrix (fdu-dbn9), branch claude/readme-comparison-matrix. README's Speed bullet and section are stale for Linux (they say the tree view and .gitignore handling are still slower; the final head is level with pdu's default and diskus on real trees, and the default tree is 39% faster than 0.2.1 on linux-v6.12). Revise README.md, docs/README.md, docs/usage.md, docs/machine-output.md, crates/fdu/README.md, crates/fdu-core/README.md, crates/fdu-py/README.md and the bundled SKILL.md for accuracy against the current binary and engine (examples, flags, outputs, performance claims), and check fdu-engine-architecture.md and platform-tuning.md describe the transient tree tier and the Linux native reader. Every claim cites a current measurement; no answer or CLI change is implied by the round.

## Notes

# Landed on claude/readme-comparison-matrix (2026-09-29), commit d2408cc3 plus README 93bac1cf

- Checked against the final-head binary (ebc06c78): every example command in README, usage.md and SKILL.md runs; the README code-analysis example reproduces at 7a499493 (21 vs 22 unclassified files, the extra one a worktree's .git file); Quick Start output format matches (numbers are an older snapshot of this repo, left as illustrative).
- Fixed: SKILL.md (and cli-surface golden) said ordinary metadata requests retain the reusable index, stale since H172's folded tree; SKILL.md and usage.md said an opened index reads and writes the cache, but the opened route refuses every cache policy but off (now "a retained index from the Rust or Python open"); engine architecture said the aggregate-only summary keeps no control table and runs only with observation off, stale since H161; README Speed bullet and section rewritten from current evidence; taglines "Fastest" -> "Fast" in the three READMEs; fdu-core README "default build features".
- Checked, no change: engine architecture already describes RetainedState::Tree (H172); platform-tuning.md's Linux row now describes the glibc native reader (H169) after #161's consistency commit; docs/machine-output.md and crates/fdu-py/README.md matched the code.
- Left alone: Cargo.toml and pyproject descriptions, and the CLI about string derived from Cargo.toml, still say "Fastest native du replacement"; research-2026-09-28 pdu brief's overview still describes the pre-round gap (dated research).

Stack layer (2026-09-29): branch claude/readme-comparison-matrix, draft PR jlevy/fdu#162, based on #161. Close when #162 merges.
