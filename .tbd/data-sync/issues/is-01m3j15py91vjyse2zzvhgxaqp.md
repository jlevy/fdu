---
type: is
id: is-01m3j15py91vjyse2zzvhgxaqp
title: "Ignore-aware transient summary: fold the ignored share without retaining an index"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
dependencies:
  - type: blocks
    target: is-01m3kkrj5f6n9b38g6d1w6mrew
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-09-27T18:13:56.296Z
updated_at: 2026-09-28T13:20:05.765Z
---
Default fdu --view summary reads .gitignore, and the summary reducer keeps no control table, so the planner falls closed to a full retained index plus a snapshot write. On the 1M balanced Linux tree (no .gitignore files at all) that costs 1.52 s against 0.92 s for --no-gitignore (screen), and 319 MiB against 10 MiB. The ignored share is a per-entry predicate over the matcher stack the walker already builds; classify in the workers and fold an ignored partition in the streaming reducer. Must match the indexed answer exactly (golden and parity corpora, including negation and nested .gitignore). Option C.1 in docs/project/research/research-2026-09-27-cache-economics-and-default-plans.md.

## Notes

2026-09-28 (C.1 agent, branch claude/perf-transient-ignore-summary, head b9ba3b40; not linked into stack 141 yet).

Implemented as H161 on the streaming path only; the detached builder is untouched.
- cbeb9e57 engine: SummaryFold (execution.rs) keeps `all`, a ControlTable under the scan's limits, the heads of ignored subtrees, and `unignored`; classifies each entry exactly as DetachedIndexBuilder::push_directory (parent ignored => ignored; empty table => not; else matcher_for(path).is_ignored); share = IgnoredTally::between(all, unignored); withheld with the index's note when any control file was refused or unreadable (control::unreadable_control, shared with Index::record_walk_errors). Planner drops `!read_controls` for the summary tier and requires population == Include; --ignored=exclude|only still take the index. report_summary reports the table's ControlCoverage and shares notes/tips helpers with report_in.
- 060bbfe6 scan: a classifying fold delivers every directory's control ahead of its entries: controls moved to the front at listing end, or, when a batch fills first, the directory's .gitignore probed directly (read_directory_control) and that read stands for the listing. Batches stay at batch_size; such a fold takes the concurrent walk even with one worker (the detached index's walk). The first cut (whole-listing hold) cost memory on wide directories (exp-172).

Gates (under timing-lock): new differential test compact_summary_equals_the_indexed_summary_under_every_control_case (whole Report Debug + text/JSON/YAML equal; negation, nesting, self-ignoring, non-file controls, rules under ignored dirs, depth bound, pruned hidden, line-limit refusal, 70 budget refusals, competing budget at 1 worker, unreadable control; workers 1/2/4/auto x batch default/1/3 x both orders) + scan test for control-before-entries; cargo test -p fdu-core and -p fdu; clippy workspace all targets all features -D warnings; fmt; make test-golden 210 passed with no summary golden changed (cli-surface changed only for the corrected skill prose); make lib-only; parity 57 deviations matched, artifact unchanged; path-independence subset OK.

macOS (uncontrolled; quiet attempt invalidated samples): exp-170 metabrowser-clone peak RSS -69.12% [-71.09, -68.71], wall -4.28% [-11.33, -0.21]; exp-171 rustup-toolchains peak RSS -57.86% [-58.98, -53.96], wall -3.87% [-10.86, +0.01], user CPU -32.25%; placebos include zero. Accepted on the pre-registered RSS primary.

PENDING (do not close): Linux deciding cell — perf_probe measure --job aggregate-summary bare (controls on), control a5c0ab46 vs candidate, 12 pairs, quiet; linux-v6.12 deciding + balanced-1M screening; wall -3% with interval < 0 and peak RSS >= 50% down; placebos: both arms --no-controls, and default-tree.
Residual: which files a competing control budget refuses is arrival-order dependent on both routes (pre-existing in the detached builder); equality there is exact only at one worker.
