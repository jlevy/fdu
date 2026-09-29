# Plan: 0.2.1, the Comparison Matrix, and Linux Parity in 0.2.2

**Date:** 2026-09-29

**Author:** fdu project

**Status:** Stage 1 in progress, stages 2 and 3 not started.
Epic `fdu-8a8r`. Stage 1 is 0.2.1, released from `c1644575`, which merges
[#155](https://github.com/jlevy/fdu/pull/155) and
[#156](https://github.com/jlevy/fdu/pull/156).

## Overview

Three steps, in order:

1. Ship **0.2.1** as it stands on #155.
2. Publish the README **comparison matrix**. It is documentation only.
3. Ship **0.2.2** with two engine changes that make fdu’s default command on Linux
   faster than pdu and diskus.

0.2.2 is a patch only if it keeps every public API and every answer.
`cargo-semver-checks` against 0.2.1, the golden corpus and the Python parity artifact
decide that. If any of them fails, the change waits for 0.3.0.

## Goals

- Tag 0.2.1 from the gated head of #155, with no further engine change.
- Publish the matrix: fdu, du, ncdu, dust, dua, gdu, pdu, diskus and dumac, plus scc and
  tokei for source-line counts.
  Every cell is backed by source reading or a committed measurement.
- In 0.2.2 on the 4-vCPU Linux host:
  - On `linux-v6.12`, bring the default `fdu .` from 211 ms to within the interval of
    pdu’s 70 ms and diskus’s 74.5 ms.
  - On the generated 1M tree, bring the default indexed tree from 1.25 s to level with
    pdu’s 1.02 s.
  - Change no answers.

## Non-Goals

- New `.gitignore` semantics.
  The matcher answers exactly what it answers today, and `IGNORE_RULES_VERSION` does not
  move.
- A new dependency in `fdu-core`. The always-on list stays `thiserror`, `anstyle`, and
  `pulldown-cmark`.
- macOS or Windows tuning.
  0.2.2 is measured on Linux and screened on macOS for regression only.
- Interactive browsing, deletion, or any matrix row fdu does not already support.

## Background

The design study measured instruction counts per thread with callgrind on `e889694c`
(`fdu-8a8r` notes). Instruction counts do not depend on host load.

- **The default command on a real repo is bound by matching.** On `linux-v6.12`:
  - 81% of the consumer thread’s instructions (1.68G of 2.07G) are `.gitignore`
    classification, a linear scan of about 111 governing patterns per entry.
  - 1,118 of the tree’s 1,593 rules are literal names, 88 are `*.suffix`, 291 are
    anchored, and 1 holds `**`.
  - pdu does no classification at all.

- **The tree view is bound by the index build.** For a one-shot tree that no reader
  reuses, the build costs:
  - 2.6k consumer instructions and 7 allocations per entry;
  - 322 MiB at 1M entries.

  pdu folds everything beyond its display depth instead.

- **Compiling the rules is not, by itself, the win.**
  - ripgrep’s `ignore` crate compiles every source with `globset`, which uses literal,
    extension, and prefix maps in front of a regex set.
  - On this tree it spends 160–250 ms of CPU on ignore handling (screen, load 2–3),
    about twice fdu’s linear matcher.
  - Much of that is per-directory matcher setup and per-candidate path allocation, so
    what matters is how little work each entry does, not whether the matcher is
    compiled.

## Design

### Why Not One Automaton Across Full Paths

One deterministic automaton over every entry’s full path, spanning all 358 sources, runs
into four problems:

1. **Priority is not set membership.**
   - git takes the deepest governing source with any matching pattern, then the last
     matching pattern in that source.
   - Anchored patterns are relative to their own source’s directory.
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
   - H163 took the first step, resolving each listing’s chain once.

So the plan precompiles each source into lookup buckets (H171). It then carries the
still-live residual patterns down the walk (H173), and only if a measured residual still
sets the wall time.

### Approach

**H171: bucketed matching, one compiled matcher per source** (`fdu-sdul`).

- At `Gitignore::parse` (`control/gitignore.rs`), sort each pattern into one of four
  buckets:
  - **literal names**: `HashMap<name, Vec<index>>`, for a basename with no
    metacharacter;
  - **suffixes**: keyed by the text after the pattern’s last `.` and confirmed with
    `ends_with`, for a `*.suffix` basename;
  - **anchored patterns**: grouped by segment count and first literal segment, for the
    `Fixed` shape;
  - **residual**: everything else.
- `matches_components` probes the maps with the entry’s name and extension, checks the
  anchored group for the entry’s depth, and scans the residual.
- It answers with the highest matching index, so last-match-wins, negation and
  `directory_only` behave exactly as today.
- `ControlChain::is_ignored_within(dir_components, name, is_dir)` takes a directory
  already split once per listing.
  The callers are `push_directory` (`index.rs`) and the cached parent in
  `SummaryControls::classify` (`execution.rs`).
- The prototype, without anchored grouping, cut consumer instructions from 2,067M to
  761M (−63%). The output was byte-identical across 1.6M JSON fields.

**H173: carry the live residual down the walk** (conditional, registered only if H171’s
counters call for it).

- Each directory holds, for each governing source, the anchored and residual patterns
  whose leading segments still match its path.
  This is the NFA state after consuming the directory.
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
- Every other file is folded into its directory’s totals and its omission tally.
- The share threshold admits at most ⌊100/min-share⌋ files as rows, so the answer stays
  exact.
- Proxy: consumer instructions fell from 523M to 195M and memory from 66 to 20 MB at
  200k entries. The wall-time effect at 1M is predicted, not confirmed.

**H164: classify on the walker threads** (`fdu-emqf`), after H171 and only if the
residual still sets the wall time.

### Components

- `crates/fdu-core/src/control/gitignore.rs`: buckets, the per-source compiled matcher,
  and the linear reference kept under `cfg(test)`.
- `crates/fdu-core/src/control.rs`: `ControlChain::is_ignored_within`, and H173’s live
  set.
- `crates/fdu-core/src/index.rs`, `execution.rs`: callers.
  H172 adds the `Tree` retained state, the bounded heap, and folded tallies.
- `crates/fdu-core/src/counters.rs`: patterns tested per entry, bucket hits, residual
  scans, and live-set size.

### API Changes

None public. The new functions are crate-private.
If H172 needs a public model change, it moves to 0.3.0 rather than widen 0.2.2.

## Implementation Plan

### Stage 1: 0.2.1

- [x] Stability pass. It ran on `672c2188`, tree-identical to `b10fe7b3`, and is recorded
  in #156. The release commit `c1644575` adds only release tooling.
- [x] #150, #155 and #156 merged.
- [x] The release tag’s signature became optional (#156), so an agent can tag with `gh`.
- [ ] Rehearse, tag, publish, announce, check and clean up 0.2.1 from `c1644575`.

### Stage 2: Comparison Matrix (documentation only, after the tag)

- [ ] `fdu-m3r6`: re-measure the Linux peer table on 0.2.1 on a quiet host.
  Cover `linux-v6.12` and the generated tree, each tool’s default invocation, and pdu’s
  depth stated.
- [ ] `fdu-bj94`: compare fdu with tokei 15.0.0 and scc 4.1.0, differential and timed,
  on `linux-v6.12`.
- [ ] `fdu-dbn9`:
  - the matrix in README (draft and source facts in the epic’s notes);
  - a short SLOC survey brief under `docs/project/research/`;
  - this plan committed under `docs/project/specs/active/`.
- [ ] A PR to `main`. The packaged READMEs pick the matrix up at 0.2.2.

### Stage 3: 0.2.2

- [ ] Run the `.gitignore` matching survey (`fdu-p6vc`) before building H171. It reads
  ripgrep’s globset and ignore, git’s `dir.c` and wildmatch, gitoxide, Sapling’s
  `TreeMatcher`, Mercurial’s Rust matchers, jj and libgit2 from source; measures each in
  instructions per entry on `linux-v6.12`, checked against `git check-ignore`; and
  adopts git’s wildmatch and ignore test cases as a conformance table.
  It may revise H171 and H173. A first attempt was stopped before reporting, so nothing
  from it is recorded.
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
- [ ] Read H171’s counters, then either register H173 or record why it is not needed.
- [ ] H172:
  - settle the carrier: a pruned index with folded tallies, or tree nodes built outside
    the index;
  - pre-register it;
  - implement it with `transient_tree_equals_the_indexed_tree_under_every_bound_case`;
  - run the quiet cell on the generated tree, with `linux-v6.12` as non-inferiority.
- [ ] H164 if the residual still sets the wall time; otherwise record it as not needed.
- [ ] Re-run the peer tables and refresh the matrix’s speed rows.
- [ ] `cargo-semver-checks` against 0.2.1, goldens, and parity.
  If all three pass, run the release layer and checklist for 0.2.2.

## Testing Strategy

- **Differential against the reference.**
  - Keep the current linear matcher under `cfg(test)`.
  - Property-test random rule sets against it with random names.
    The rule sets mix literal, `*.suffix`, anchored, negated, directory-only, escaped,
    `[...]`, non-UTF-8, trailing-`.` and `*.` patterns.
- **Differential against git.**
  - For `linux-v6.12` and the wildcard subject, compare fdu’s ignored set with
    `git ls-files -oi --exclude-standard` and `git check-ignore --stdin` over every
    path.
  - This is the first real-tree check of fdu’s matcher against git itself.
- **Existing tests:**
  - the `git check-ignore` verdict table;
  - the chain-versus-per-entry differential tests across worker counts, orders, and both
    case-lookup modes;
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
- H172’s carrier: a pruned index with a folded tally on each directory, or tree nodes
  built outside the index?
- H172’s K ceiling. Do `--sort name`, `mtime` and `count` keep the tier?
  Eligibility depends on size only.
- Should the published tool comparison add `linux-v6.12` with `.gitignore` handling on
  as a product job?

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
