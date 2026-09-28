---
type: is
id: is-01m3j15py91vjyse2zzvhgxaqp
title: "Ignore-aware transient summary: fold the ignored share without retaining an index"
kind: task
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
delegate: claude-code@spud10.local
labels:
  - performance
dependencies:
  - type: blocks
    target: is-01m3kkrj5f6n9b38g6d1w6mrew
parent_id: is-01kzy554jjg27mz97mryenftym
hold: null
hold_until: null
created_at: 2026-09-27T18:13:56.296Z
updated_at: 2026-09-28T14:30:13.066Z
started_at: 2026-09-28T14:30:07.463Z
---
Default fdu --view summary reads .gitignore, and the summary reducer keeps no control table, so the planner falls closed to a full retained index plus a snapshot write. On the 1M balanced Linux tree (no .gitignore files at all) that costs 1.52 s against 0.92 s for --no-gitignore (screen), and 319 MiB against 10 MiB. The ignored share is a per-entry predicate over the matcher stack the walker already builds; classify in the workers and fold an ignored partition in the streaming reducer. Must match the indexed answer exactly (golden and parity corpora, including negation and nested .gitignore). Option C.1 in docs/project/research/research-2026-09-27-cache-economics-and-default-plans.md.

## Notes

2026-09-28 (C.1 agent, branch claude/perf-transient-ignore-summary, PR #149 on stack 151; head b9ba3b40 at the first handoff).

Implemented as H161 on the streaming path only; the detached builder is untouched.
- cbeb9e57 engine: SummaryFold (execution.rs) keeps `all`, a ControlTable under the scan's limits, the heads of ignored subtrees, and `unignored`; classifies each entry exactly as DetachedIndexBuilder::push_directory (parent ignored => ignored; empty table => not; else matcher_for(path).is_ignored); share = IgnoredTally::between(all, unignored); withheld with the index's note when any control file was refused or unreadable (control::unreadable_control, shared with Index::record_walk_errors). Planner drops `!read_controls` for the summary tier and requires population == Include; --ignored=exclude|only still take the index. report_summary reports the table's ControlCoverage and shares notes/tips helpers with report_in.
- 060bbfe6 scan: a classifying fold delivers every directory's control ahead of its entries: controls moved to the front at listing end, or, when a batch fills first, the directory's .gitignore probed directly (read_directory_control) and that read stands for the listing. Batches stay at batch_size; such a fold takes the concurrent walk even with one worker (the detached index's walk). The first cut (whole-listing hold) cost memory on wide directories (exp-172).

Gates (under timing-lock): new differential test compact_summary_equals_the_indexed_summary_under_every_control_case (whole Report Debug + text/JSON/YAML equal; negation, nesting, self-ignoring, non-file controls, rules under ignored dirs, depth bound, pruned hidden, line-limit refusal, 70 budget refusals, competing budget at 1 worker, unreadable control; workers 1/2/4/auto x batch default/1/3 x both orders) + scan test for control-before-entries; cargo test -p fdu-core and -p fdu; clippy workspace all targets all features -D warnings; fmt; make test-golden 210 passed with no summary golden changed (cli-surface changed only for the corrected skill prose); make lib-only; parity 57 deviations matched, artifact unchanged; path-independence subset OK.

macOS (uncontrolled; quiet attempt invalidated samples): exp-170 metabrowser-clone peak RSS -69.12% [-71.09, -68.71], wall -4.28% [-11.33, -0.21]; exp-171 rustup-toolchains peak RSS -57.86% [-58.98, -53.96], wall -3.87% [-10.86, +0.01], user CPU -32.25%; placebos include zero. Accepted on the pre-registered RSS primary.

PENDING (do not close): Linux deciding cell — perf_probe measure --job aggregate-summary bare (controls on), control a5c0ab46 vs candidate, 12 pairs, quiet; linux-v6.12 deciding + balanced-1M screening; wall -3% with interval < 0 and peak RSS >= 50% down; placebos: both arms --no-controls, and default-tree.
Residual: which files a competing control budget refuses is arrival-order dependent on both routes (pre-existing in the detached builder); equality there is exact only at one worker.

2026-09-28 review fixes (head eeb257c9 on PR #149; the coordinator's c07bf30a merge of main and 3b3e8f5d cfg(unix) field fix are below it).
- Exact name: the mid-listing probe (probe_directory_control) now confirms a lookup hit by listing the directory until the first entry named exactly `.gitignore` (lists_exact_control_name, streaming, retains nothing, paid only on a hit) before reading bytes. Before, on case-insensitive volumes a `.GITIGNORE` answered the probe though no listing or index accepts it, so the default summary could differ from the index depending on where a batch filled. The coordinator's repro tree (`.GITIGNORE` *.log + 2,000 *.log + keep.txt) now prints `7.8 MiB  2,002 files, 0 directories` on both routes (3/3 default runs, 1 index run). Index exact-name semantics unchanged; the narrowed-population walk's read_directory_control still resolves by path (pre-existing, separate).
- Read once: a probed listing prepares its listed `.gitignore` without reading it (prepare_walk_entry_reading(read_control=false)); counters.control_reads counts each file once (58 reads for 58 .gitignore files on the live metabrowser tree, 7,076 directory opens for 7,075 directories + root: no probe hit at the default batch there).
- Docs: ScanConfig::threads one-worker exceptions; probed-listing race semantics; ignored_heads unbounded antichain.
- Tests: a_directory_control_probe_accepts_only_the_exact_name; a_probed_listing_prepares_its_control_entry_without_reading_it (unix); differential case "a case-variant control name" (.GITIGNORE + 1,200 *.log, runs only on a case-insensitive temp volume). Mutation check: disabling the confirmation makes both the probe test and the differential case fail.
- Gates (timing-lock, fresh target): cargo test -p fdu-core (837 lib) and -p fdu; clippy workspace -D warnings; fmt; make admission-sites (8 loops); make cross-lint (x86_64-apple-darwin, x86_64-pc-windows-msvc); make msrv incl. Windows leg; make test-golden 210; make test-parity 57 matched.
- exp-170/171 stay representative: rustup has no .gitignore, so every probe missed at the lookup and neither fix changes its work; on metabrowser no probe hit at the default batch in the counter run, and the fixes only remove a second read or add a partial listing on a hit.
