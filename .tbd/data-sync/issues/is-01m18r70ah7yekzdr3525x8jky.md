---
type: is
id: is-01m18r70ah7yekzdr3525x8jky
title: "~/Library scan is SIGKILLed (137): unbounded growth the control cap does not govern"
kind: bug
status: open
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - scale
  - macos
  - stack-followup
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:12:47.952Z
updated_at: 2026-09-30T10:56:24.945Z
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

2026-09-30 stability pass, second entry (Linux, release binary at d92b9ba5): the growth is reproduced and measured. Peak RSS (ru_maxrss, minimum of three runs) of the reporter's shape `-d 2 -n 30 --sort size --min-size 300M` against the default `fdu PATH`: linux-v6.12 (92,473 entries) 40.5 MiB against 10.8 MiB; node-modules-dense (79,956) 53.6 MiB against 12.7 MiB; a generated tree of 404,000 empty files 90.2 MiB against 15.2 MiB. The folded default is flat in the entry count; the filtered shape grows linearly at 190 to 520 B per entry (name length dependent), the same with `--cache off` and under auto. `--sort size` alone stays folded (10.9, 12.7, 15.3 MiB), so the size filter is what routes the request to RetainedState::FullIndex: execution::TreeRetention::for_request refuses any selection that is not is_unfiltered(). At a few million entries that is 0.6 to 1.5 GiB of index on top of the walk, and on a host at 95-99% disk with no room to swap the OOM killer is the plausible 137. No cap governs it and none should: a full index must hold every entry. The fix is a fold that applies the selection: keep every directory, since roll-ups are unfiltered, and rank for the share bound only the files the selection admits. TreeRetention keeps the top largest_files by size, any row the report can show has a share at or above the threshold, so the top-K admitted files by size contain every showable file row for every selection that filters a file by its own path and attributes. That is a change to the default route's fold and to for_request's admission, proven by extending the transient-versus-indexed differentials in execution.rs with filtered cases (min and max size, include and exclude, a modified window, and a name sort under each). Not done in this pass: it moves which route the default command takes for filtered requests on the eve of the release, and the differential extension is the larger half of the work. The TCC-induced slowness under ~/Library/Containers remains separate.

2026-09-30 stability pass, third entry (claude/filtered-fold at 1275c485, merged into claude/stability-fixes): the proposed fold is not exact, so nothing changed in the planner. The prior note's premise that "roll-ups are unfiltered" is false for a one-shot filtered tree (query_subtrees::measure -> walk -> tree_node/expand): a directory whose measured subtree passes the filter matches and then covers every descendant not excluded, a directory that does not match counts only what the filter admits beneath it, and the share denominator is the filtered root total. CLI check, --size apparent --min-share 0%: the unfiltered root is 1.2 KiB; with --min-size 300 the root is 950 B, A/ (600 B, matched) shows A/small (50 B) and A/deep/tiny (30 B), and B/ (255 B) is hidden. A fold keeping unfiltered roll-ups and ranking only admitted files gets the directory sizes, the denominator and the covered rows wrong; ranking every file is also wrong, since files the selection never counts can outnumber K and displace a shown row (the crowded_tree fixture at 1%, K=100: hit/covered is 900 B, below the minimum but shown, with 111 larger unselected files).

Landed instead: a_filtered_tree_answers_as_the_full_index_on_every_route (min size, include/exclude names and directories, include+exclude, modified since/before/window, kinds, ignored exclude/only, min size with include; default and name order, the reporter's -d 2 -n 30 --sort size, at 1% and 10%, both metrics, on a random and a crowded tree; it checks each filter changes the answer and compares the folded index wherever the planner folds) and a_filtered_tree_is_measured_against_what_it_selects (the counterexample with exact totals). Temporarily admitting filters in for_request makes the differential fail, so it guards any future filtered fold.

An exact --min-size fold is a design change: a reader deriving filtered sums from roll-ups with the top-level matching rule (restating walk's ignored tallies, newest times and unknown classification), and a retention keeping top-K per top-level directory of files >= s*M, bounded by K x (top-level directories holding one) rather than MAX_FILES; or fold, check the smallest kept file is below s*T, and re-walk with the full index when it is not (exact, slower on failure). kind file, exclude-only and ignored-exclude trees map onto roll-up partitions and could fold with a new reader; none is the reporter's filter. 0.3.0 answers a filtered tree from the full index, as 0.2.1 did: no regression, the memory growth the second entry measured remains.
