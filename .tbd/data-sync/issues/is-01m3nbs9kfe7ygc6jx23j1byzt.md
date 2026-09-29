---
type: is
id: is-01m3nbs9kfe7ygc6jx23j1byzt
title: "0.2.2: beat pdu and diskus on Linux for the default tree and .gitignore handling"
kind: epic
status: open
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
child_order_hints:
  - is-01m3nbsak6hftpnr1rx5x90d1w
  - is-01m3nbsbrbr5v9y6r3x8pxgmr5
created_at: 2026-09-29T01:17:07.054Z
updated_at: 2026-09-29T04:47:17.334Z
---
Point solution from the 2026-09-29 design study (brief in this epic's notes). Gap (b), the default command on a real repo, is 81% consumer-thread .gitignore classification: a linear scan of ~111 governing patterns per entry, although 1,118 of the 1,593 linux-v6.12 rules are literal names or *.suffix. Fix: H171, bucketed matching. Gap (a), the tree view on the generated 1M tree, is the one-shot index build that no reader reuses. Fix: H172, an exact transient tree tier. H164 (fdu-emqf) follows on the ~0.5G residual. Predicted quiet ranking on linux-v6.12: default tree 211 -> ~85-95 ms (pdu 70, diskus 74.5); balanced tree 1.25 -> ~1.03-1.10 s (pdu 1.02). Ids H171/H172 come from the unused fdu-k1n8 block (H171-H179).

## Notes

# Verdict

**Point solution for 0.3.0: H171, bucketed `.gitignore` matching** (~150 lines in `gitignore.rs` + `control.rs`), prototyped and measured: consumer instructions on the `linux-v6.12` default tree **2,067M → 761M (−63%)**, summary 1,918M → 660M, answers byte-identical (0 non-timing differences over 1,629,566 JSON leaves; 93 control tests incl. the `git check-ignore` verdict table pass). **Second half of the pair, H172, an exact transient tree tier** (directories + the K largest files), closes gap (a): proxy cuts the balanced consumer 523M → 195M and RSS 66 → 20 MB; wall effect at 1M is a prediction (level with pdu), not confirmed by my 200k screen under load.

Why fdu loses today:
- **(b)** Classification is 81% of the consumer's instructions (1.68G of 2.07G) on the one thread that cannot scale: ~111 governing patterns per entry × ~140 instructions per `glob_matches` = 19.6k instr/entry. 63 of the root file's 106 rules and 1,118 of all 1,593 are literal names or `*.suffix`, answerable by a lookup. This is *removable overhead*, not work pdu skips.
- **(a)** The index build: 2.6k consumer instr/entry (10× a walker's per-thread work), 7 allocs/entry, 66 MB/200k → 8× the page faults, a fifth runnable thread on 4 vCPUs. Work pdu skips by design; after H159 no cross-thread frees remain, and no one-shot reader reads it.

# Evidence

Consumer-thread instructions, callgrind on a profiling build of e889694c (load-independent):

| Job, linux-v6.12 | Consumer | Walker (each) |
|---|---:|---:|
| default tree | 2,067M | 30M |
| tree `--no-gitignore` | 257M | 19–32M |
| summary, controls on | 1,918M | 55–64M |
| default tree, **H171 prototype** | 761M | 24–35M |
| summary, H171 prototype | 660M | 43–49M |
| default tree, H171 + H172 proxy | 588M | 24–27M |

Attribution (default tree, control): `ControlChain::is_ignored` 1,684M incl. (control.rs:765), `glob_matches` 1,423M (gitignore.rs:364; 1,258M self), `chain_for` 104M (control.rs:527), per-child path split in `with_components` ~76M (gitignore.rs:42, called at control.rs:769). Index build alone = the 257M: `derive_ext`/`intern_ext` 66M, child sort 34M, `contribution`+`merge` 52M (index.rs:5252, 1713–1715), malloc/free 48M.

Rule mix (358 files, 1,593 rules): 1,301 basename (1,118 literal, 88 `*.suffix`), 291 anchored, 1 `**`; mean 110.9 governing rules and 1.70 sources per entry. Root file: 37 `*.suffix`, 32 anchored, 26 literal (9 negated), 12 other globs.

| Balanced proxy (200k entries, recipe shape) | Consumer Ir | Allocs | Minor faults | RSS |
|---|---:|---:|---:|---:|
| default tree | 523M | 1.40M | 15.0k | 66 MB |
| summary `--no-gitignore` | 12M | 876k | 1.8k | 10 MB |
| tree, H172 proxy | 195M | — | — | 19.5 MB |

Tree consumer self: malloc/free ~100M, `InternedRollUp::merge` 83M, `derive_ext` 70M, `contribution` 55M, `ExtTally` insert 43M, sort ~50M, `HashMap<PathBuf>` ~56M (H167), `intern_ext` 32M. Proxy remainder: path hashing ~36M, sort 17M.

Screens (rusage, shared host; load average in brackets):
- linux-v6.12 [2–3]: fdu default tree 207–218 ms (user 0.17–0.21), `--no-gitignore` 91–180, summary 194–216, pdu 66–99, diskus 87–109. **Reference ceiling:** `rg --files` (ignore on, 86,615 files) 157–196 ms wall / 235–330 ms user vs `--no-ignore` 69–75 / 75 user; `-j1` 233–236 vs 73–76. ripgrep's compiled globset costs ~160 ms CPU here, 2× fdu's, and stays 2.2–2.7× its own no-ignore wall in parallel: a compiled matcher is not the lever.
- H171 vs control [7, tests compiling]: tree 281–316 → 194–222 (−27–31%); summary 231–378 → 194–225; `--no-gitignore` 157–201 both.
- H172 proxy on H171 [4]: linux tree 146–187 → 119–158, `--no-gitignore` 117–131 → 103–119, pdu 101–119; balanced tree 0.29–0.34 → 0.29–0.34 (unresolved), summary 0.25–0.29, pdu 0.23–0.25.

# Design

**H171 mechanism.** At `Gitignore::parse` (gitignore.rs:104) bucket each pattern: Basename with no metachar → `literal_names: HashMap<name, Vec<index>>`; Basename `*.suffix` with no other metachar → `suffixes` keyed by the text after the pattern's last `.`, each confirmed by `ends_with`; everything else → `residual`. `matches_components` (gitignore.rs:125) probes both maps with the entry's name and extension, scans the residual, and answers `patterns[max matching index].ignored` — last-match-wins, negations and `directory_only` preserved exactly. Add `ControlChain::is_ignored_within(dir_components, name, is_dir)` and split the listing's directory once in `push_directory` (index.rs:1684–1695) and per cached parent in `SummaryControls::classify` (execution.rs:797–801). Snapshot format unchanged (matchers are rebuilt from source bytes at control.rs:500). Optional in the same change: bucket anchored (`Fixed`) patterns by segment count (~80M more). Diff saved at scratchpad `aea5-prototype-h171-h172proxy.diff`.

**H172 mechanism.** `RetainedState::Tree` (execution.rs:25, chosen at :439–447 beside `summary_is_sufficient` :414) when route is OneShot, no index policy, no analysis, views == [Tree], unfiltered, population Include, min-share > 0, K = ⌈100/pct⌉ ≤ 65,536. `push_directory` allocates entries for directories only; each file is classified, folded into its parent's `all`/`unignored` roll-up scalars (no entry, no ext tally), and offered to a global bounded min-heap of the K largest by the request's size metric; unretained files add to the parent's `folded` tally (entries, files, bytes, allocated, ignored); their names stay in the listing so the walker frees them (scan.rs:3260, H159). `expand` adds `folded` to the Share omission at query_report.rs:3044 via `Index::folded_children(id)`, `None` on a full index. Exactness: `ShareThreshold::admits` (query_selection.rs:206–218) is false at total 0 and needs part×100 ≥ pct×whole, so at most ⌊100/pct⌋ files are ever rows whatever the sort key, and K+1 files tied at the boundary would exceed the total. Licensed by the architecture: "One-shot `report()` may use the derived-report plan when the request proves that retained state has no consumer" and execution.rs:21–23.

**Answers change:** nothing, on either.

**Hypothesis rows.**

| Id | Claim | Accept |
|---|---|---|
| H171 | After H162–H163, 81% of the default tree's consumer instructions on linux-v6.12 are a linear scan of ~111 governing patterns per entry, though most rules are literal names or `*.suffix`; indexing each source by name and suffix and taking the highest matching index answers them in two lookups. | `default-tree` (deciding) and `aggregate-summary`, controls on, linux-v6.12: wall −3% with the 95% interval below zero; consumer Ir −50%; placebos both arms `--no-controls` on linux-v6.12 and `default-tree` on balanced-1M include zero; peak RSS non-inferior; route differential identical. |
| H172 | A one-shot tree reads only directory roll-ups and at most ⌈100/min-share⌉ files, yet `fdu .` retains every entry (2.6k instr and 7 allocations per entry, 322 MiB at 1M). A `Tree` tier keeps directories plus the K largest files and folds the rest into roll-ups and omission tallies. | `default-tree` on balanced-1M (deciding): wall −3%, interval below zero, and peak RSS −50%; `default-tree` on linux-v6.12 non-inferior; placebos `aggregate-summary --no-controls` and `cold-scan-index` include zero; goldens and parity unchanged. |

**Tests.** H171: keep the linear matcher under `cfg(test)` and property-test random rule sets (literal, suffix, anchored, negated, dir-only, escaped, `[..]`) × random names against it; the existing verdict table. H172: mirror `compact_summary_equals_the_indexed_summary_under_every_control_case` (execution.rs:2350) as `transient_tree_equals_the_indexed_tree_under_every_bound_case` over workers 1/2/4/8 × both orders × min-share {1%, 0.5%, 10%, 0.01%} × size metric × sort keys × limit/breadth/depth × ties at K × zero-size trees × ignored files at the boundary × refused controls; `tests/parallel_equivalence.rs`; goldens; parity (CI); path-independence subset.

**Predicted ranking.** (b) quiet scale: consumer 2,067M ≈ 200 ms → 761M ≈ 60–75 ms, below the 81 ms walk floor, so default tree 211 → ~85–95 ms, summary 167 → ~75–85; with H172 ~80–90. pdu 70, diskus 74.5: fdu default +15–25% vs pdu, ≈ diskus; `--no-gitignore` level. Closing the rest needs H164 on the 0.5G residual. (a) harness scale: tree 1.25 adds 0.64 CPU-s over the 0.94 summary; H172 removes ~63% of consumer instructions and ~70% of faults ≈ 0.4 CPU-s → 1.03–1.10 s, level with pdu 1.02 and diskus 1.04 within the interval; ahead only with H167 or H166 on top.

**Residual risks.** H171: bucket semantics for `*.`, names ending in `.`, escapes (excluded by metachar test), non-UTF-8 bytes; two maps per distinct source (small; measure RSS). H172: a new model element (`folded`) and tier; every non-tree reader of a pruned index must be unreachable and asserted; heap ties; tiny thresholds; the proxy could not resolve wall at 200k under load. Both: one 4-vCPU virtualized regime.

# Rejected

- **H164** (classify on walkers): moves 1.8G instructions onto walkers doing 30M each; after H171 the residual is 0.5G and H164 is the natural follow-up, not the first move; also needs a public `Op::Upsert` field for the summary route and walker-side budget admission.
- **H166/H170**: target walker parking; the tree route's binding cost is the consumer (523M vs 50M per walker); H170 is summary-only.
- **H167/H168**: 56M and ~100M of a 523M consumer (H168 moot for folded files under H172).
- **Compiled globset**: ripgrep's costs 160 ms CPU on this tree, 2× fdu's linear matcher.
- **Pruning ignored subtrees**: default population is Include; nothing to prune on the default path.
- **Allocator**: −12–17% screened but a C dependency and +RSS (H74); H172 removes the allocations instead.

# Open questions

1. H172 carrier: a pruned `Index` with a `folded` tally on directory entries (~400–500 lines, `expand` +10) or `TreeNode`s built outside the index (`expand` abstracted, larger)?
2. Ship H171 alone in 0.3.0 and H172 in 0.3.x/0.4.0, given its size?
3. K ceiling and whether `--sort name/mtime/count` keep the tier (the proof says eligibility is size-only).
4. Add linux-v6.12 default (controls on) as a product job in the tool comparison, and state pdu's depth.
5. Register H164 as the sequenced follow-up to H171.

Worktree restored to e889694c with no changes; build directory, proxy tree and binaries deleted; scripts and the prototype diff remain in the scratchpad (`aea5-*`).


---
# PLAN (2026-09-29)

# Plan: 0.2.1, the Comparison Matrix, and Linux Parity in 0.2.2

**Date:** 2026-09-29

**Author:** fdu project

**Status:** Draft.
Stage 1 is on [#155](https://github.com/jlevy/fdu/pull/155); stages 2 and 3 have not
started. Epic `fdu-8a8r`.

## Overview

Three steps, in order:

1. Ship **0.2.1** as it stands on #155.
2. Publish the README **comparison matrix**. It is documentation only.
3. Ship **0.2.2** with two engine changes that make fdu's default command on Linux faster
   than pdu and diskus.

0.2.2 is a patch only if it keeps every public API and every answer.
`cargo-semver-checks` against 0.2.1, the golden corpus and the Python parity artifact
decide that. If any of them fails, the change waits for 0.3.0.

## Goals

- Tag 0.2.1 from the gated head of #155, with no further engine change.
- Publish the matrix: fdu, du, ncdu, dust, dua, gdu, pdu, diskus and dumac, plus scc and
  tokei for source-line counts. Every cell is backed by source reading or a committed
  measurement.
- In 0.2.2 on the 4-vCPU Linux host:
  - On `linux-v6.12`, bring the default `fdu .` from 211 ms to within the interval of
    pdu's 70 ms and diskus's 74.5 ms.
  - On the generated 1M tree, bring the default indexed tree from 1.25 s to level with
    pdu's 1.02 s.
  - Change no answers.

## Non-Goals

- New `.gitignore` semantics.
  The matcher answers exactly what it answers today, and `IGNORE_RULES_VERSION` does not
  move.
- A new dependency in `fdu-core`.
  The always-on list stays `thiserror`, `anstyle`, and `pulldown-cmark`.
- macOS or Windows tuning. 0.2.2 is measured on Linux and screened on macOS for
  regression only.
- Interactive browsing, deletion, or any matrix row fdu does not already support.

## Background

The design study measured instruction counts per thread with callgrind on `e889694c`
(`fdu-8a8r` notes). Instruction counts do not depend on host load.

- **The default command on a real repo is bound by matching.** On `linux-v6.12`:
  - 81% of the consumer thread's instructions (1.68G of 2.07G) are `.gitignore`
    classification, a linear scan of about 111 governing patterns per entry.
  - 1,118 of the tree's 1,593 rules are literal names, 88 are `*.suffix`, 291 are
    anchored, and 1 holds `**`.
  - pdu does no classification at all.
- **The tree view is bound by the index build.** For a one-shot tree that no reader
  reuses, the build costs:
  - 2.6k consumer instructions and 7 allocations per entry;
  - 322 MiB at 1M entries.

  pdu folds everything beyond its display depth instead.
- **Compiling the rules is not, by itself, the win.**
  - ripgrep's `ignore` crate compiles every source with `globset`, which uses literal,
    extension, and prefix maps in front of a regex set.
  - On this tree it spends 160–250 ms of CPU on ignore handling (screen, load 2–3), about
    twice fdu's linear matcher.
  - Much of that is per-directory matcher setup and per-candidate path allocation, so
    what matters is how little work each entry does, not whether the matcher is
    compiled.

## Design

### Why Not One Automaton Across Full Paths

One deterministic automaton over every entry's full path, spanning all 358 sources, runs
into four problems:

1. **Priority is not set membership.**
   - git takes the deepest governing source with any matching pattern, then the last
     matching pattern in that source.
   - Anchored patterns are relative to their own source's directory.
   - An excluded directory hides its descendants from re-inclusion, and the walk already
     answers that case through `parent_ignored`.
   - A global automaton would have to encode all of this for each distinct chain of
     sources. That makes it a product of per-source automata, not one machine.
2. **State growth.**
   - Subset construction over 1,593 globs holding `*` and `[...]` can grow large.
   - A lazy DFA bounds that growth, but it needs a dependency and a per-thread cache.
3. **Diminishing return.**
   - Once each entry costs two hash probes per governing source, the predicted consumer
     time falls below the 81 ms walk floor.
   - The consumer then no longer sets the wall time, so a faster matcher cannot shorten
     the run.
4. **The useful part of the idea is incremental evaluation, and it needs no automaton
   library.**
   - The walk is already a traversal of a trie.
   - The directory prefix should be consumed once per directory and carried to the
     children, not re-matched for every entry.
   - H163 took the first step, resolving each listing's chain once.

So the plan precompiles each source into lookup buckets (H171). It then carries the
still-live residual patterns down the walk (H173), and only if a measured residual still
sets the wall time.

### Approach

**H171: bucketed matching, one compiled matcher per source** (`fdu-sdul`).

- At `Gitignore::parse` (`control/gitignore.rs`), sort each pattern into one of four
  buckets:
  - **literal names**: `HashMap<name, Vec<index>>`, for a basename with no
    metacharacter;
  - **suffixes**: keyed by the text after the pattern's last `.` and confirmed with
    `ends_with`, for a `*.suffix` basename;
  - **anchored patterns**: grouped by segment count and first literal segment, for the
    `Fixed` shape;
  - **residual**: everything else.
- `matches_components` probes the maps with the entry's name and extension, checks the
  anchored group for the entry's depth, and scans the residual.
- It answers with the highest matching index, so last-match-wins, negation and
  `directory_only` behave exactly as today.
- `ControlChain::is_ignored_within(dir_components, name, is_dir)` takes a directory
  already split once per listing. The callers are `push_directory` (`index.rs`) and the
  cached parent in `SummaryControls::classify` (`execution.rs`).
- The prototype, without anchored grouping, cut consumer instructions from 2,067M to
  761M (−63%). The output was byte-identical across 1.6M JSON fields.

**H173: carry the live residual down the walk** (conditional, registered only if H171's
counters call for it).

- Each directory holds, for each governing source, the anchored and residual patterns
  whose leading segments still match its path. This is the NFA state after consuming
  the directory.
- A child file tests only live patterns with one segment left.
- A child directory inherits the live set, advanced by one segment.
- When the live set is empty, classification below that directory is the hash probes
  alone.
- This is the state machine across full paths, evaluated incrementally, with no new
  dependency.

**H172: an exact transient tree tier** (`fdu-dp98`).

- When the request proves that nothing will read the index, a one-shot tree retains:
  - every directory;
  - the K largest files, with K = ⌈100/min-share⌉.
- Every other file is folded into its directory's totals and its omission tally.
- The share threshold admits at most ⌊100/min-share⌋ files as rows, so the answer stays
  exact.
- Proxy: consumer instructions fell from 523M to 195M and memory from 66 to 20 MB at 200k
  entries. The wall-time effect at 1M is predicted, not confirmed.

**H164: classify on the walker threads** (`fdu-emqf`), after H171 and only if the residual
still sets the wall time.

### Components

- `crates/fdu-core/src/control/gitignore.rs`: buckets, the per-source compiled matcher,
  and the linear reference kept under `cfg(test)`.
- `crates/fdu-core/src/control.rs`: `ControlChain::is_ignored_within`, and H173's live
  set.
- `crates/fdu-core/src/index.rs`, `execution.rs`: callers. H172 adds the `Tree` retained
  state, the bounded heap, and folded tallies.
- `crates/fdu-core/src/counters.rs`: patterns tested per entry, bucket hits, residual
  scans, and live-set size.

### API Changes

None public.
The new functions are crate-private.
If H172 needs a public model change, it moves to 0.3.0 rather than widen 0.2.2.

## Implementation Plan

### Stage 1: 0.2.1

- [ ] Stability pass on #155's head (`672c2188`):
  - `make check`, `cross-lint`, `release-rehearse`, `semver-check`;
  - the QA playbook with peer agreement, and the correctness runbook.
- [ ] CI green on #155. GitHub has started no run since `e889694c`.
- [ ] The maintainer merges #150, then #155, with merge commits.
- [ ] The maintainer runs the Release Checklist from `make release-preflight` through
  `make release-body`, then tags and publishes.

### Stage 2: Comparison Matrix (documentation only, after the tag)

- [ ] `fdu-m3r6`: re-measure the Linux peer table on 0.2.1 on a quiet host. Cover
  `linux-v6.12` and the generated tree, each tool's default invocation, and pdu's depth
  stated.
- [ ] `fdu-bj94`: compare fdu with tokei 15.0.0 and scc 4.1.0, differential and timed, on
  `linux-v6.12`.
- [ ] `fdu-dbn9`:
  - the matrix in README (draft and source facts in the epic's notes);
  - a short SLOC survey brief under `docs/project/research/`;
  - this plan committed under `docs/project/specs/active/`.
- [ ] A PR to `main`. The packaged READMEs pick the matrix up at 0.2.2.

### Stage 3: 0.2.2

- [ ] Nominate a second real subject heavy with wildcard rules, to test the residual
  path.
- [ ] H171:
  - pre-register against the `fdu-sdul` acceptance row;
  - implement it with anchored grouping;
  - add the property test against the linear matcher and the real-tree differential
    against git;
  - run the quiet cell on `linux-v6.12`, the wildcard subject, and the generated
    placebo;
  - record the results.
- [ ] Read H171's counters, then either register H173 or record why it is not needed.
- [ ] H172:
  - settle the carrier: a pruned index with folded tallies, or tree nodes built outside
    the index;
  - pre-register it;
  - implement it with `transient_tree_equals_the_indexed_tree_under_every_bound_case`;
  - run the quiet cell on the generated tree, with `linux-v6.12` as non-inferiority.
- [ ] H164 if the residual still sets the wall time; otherwise record it as not needed.
- [ ] Re-run the peer tables and refresh the matrix's speed rows.
- [ ] `cargo-semver-checks` against 0.2.1, goldens, and parity. If all three pass, run the
  release layer and checklist for 0.2.2.

## Testing Strategy

- **Differential against the reference.**
  - Keep the current linear matcher under `cfg(test)`.
  - Property-test random rule sets against it with random names. The rule sets mix
    literal, `*.suffix`, anchored, negated, directory-only, escaped, `[...]`,
    non-UTF-8, trailing-`.` and `*.` patterns.
- **Differential against git.**
  - For `linux-v6.12` and the wildcard subject, compare fdu's ignored set with
    `git ls-files -oi --exclude-standard` and `git check-ignore --stdin` over every path.
  - This is the first real-tree check of fdu's matcher against git itself.
- **Existing tests:**
  - the `git check-ignore` verdict table;
  - the chain-versus-per-entry differential tests across worker counts, orders, and
    both case-lookup modes;
  - `tests/parallel_equivalence.rs`;
  - the golden corpus and Python parity, unchanged.
- **H172:** the transient-versus-indexed tree differential across bounds, sort keys,
  metrics, ties at K, zero-size trees, and ignored files at the boundary.
- **Performance** follows the loop:
  - pre-registered, quiet host, 12 interleaved pairs, with placebos;
  - accept at −3% with the 95% interval below zero;
  - instruction counts as the load-independent secondary.

## Rollout Plan

- 0.2.1 ships unchanged.
- The matrix lands on `main` between releases.
- 0.2.2 ships the accepted changes behind no flag, because answers do not change.
- The engine fingerprint changes with the release, as it does every release, so the
  first run after upgrade is cold.

## Open Questions

- Which real repository heavy with wildcard rules should be the second subject?
- H172's carrier: a pruned index with a folded tally on each directory, or tree nodes
  built outside the index?
- H172's K ceiling. Do `--sort name`, `mtime` and `count` keep the tier? Eligibility
  depends on size only.
- Should the published tool comparison add `linux-v6.12` with `.gitignore` handling on as
  a product job?

## References

- [pdu brief](../../research/research-2026-09-28-pdu-and-the-linux-peer-gap.md)
- [Performance loop](../../guides/performance-loop.md), H164–H170
- [Engine architecture](../../architecture/fdu-engine-architecture.md) and
  [design principles](../../architecture/fdu-design-principles.md)
- Beads:
  - epic `fdu-8a8r`, with `fdu-sdul` (H171), `fdu-dp98` (H172) and `fdu-emqf` (H164);
  - `fdu-dbn9`, `fdu-m3r6`, `fdu-bj94`, `fdu-kg22`.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
