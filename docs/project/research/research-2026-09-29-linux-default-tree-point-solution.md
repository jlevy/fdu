# Research: A Point Solution for the Linux Default Tree and `.gitignore` Gap

**Date:** 2026-09-29

**Author:** fdu project

**Status:** Design study for 0.2.2. It proposes H171 and H172, and H173 as a conditional
follow-up.
Registered in [the performance loop](../guides/performance-loop.md#hypotheses)
and planned in [the 0.2.2 plan](../specs/active/plan-2026-09-29-linux-parity-0.2.2.md).
Beads: epic `fdu-8a8r`, `fdu-sdul` (H171), `fdu-dp98` (H172), `fdu-emqf` (H164).

## Question

After H162 and H163, fdu still trails pdu on Linux in two places:

- **The default command on a real repository.** On `linux-v6.12` the default tree takes
  211 ms and the default summary 167 ms, against pdu’s 70 ms and diskus’s 74.5 ms.
- **The indexed tree on the generated million-entry tree.** It took 1.25 s in 0.2.0
  against pdu’s 1.02 s. H159 cut that job by about a tenth.

The question was what the smallest change is that closes both gaps without changing any
answer.

## Method and Regime

- **Host:** a 4-vCPU Firecracker ext4 guest, shared with other agents’ builds.
- **Build:** a profiling build of `e889694c` (the 0.2.1 release layer).
- **Instruction counts:** measured per thread with callgrind, so they do not depend on
  host load.
- **Wall-clock figures:** all are screens under load, with the load average given in
  brackets. None is a loop measurement.
- **Subjects:**
  - `linux-v6.12`: 92,474 entries, 5,769 directories, 358 `.gitignore` files with 1,593
    rules;
  - a 200k-entry proxy of the generated `linux-balanced-1m` recipe.

## Findings

### Where the Consumer Spends Its Instructions

| Job, `linux-v6.12` | Consumer | Each walker |
| --- | ---: | ---: |
| default tree | 2,067M | 30M |
| tree, `--no-gitignore` | 257M | 19–32M |
| summary, controls on | 1,918M | 55–64M |
| default tree, H171 prototype | 761M | 24–35M |
| summary, H171 prototype | 660M | 43–49M |
| default tree, H171 plus H172 proxy | 588M | 24–27M |

On the default tree, the consumer’s instructions break down as follows:

- **Classification is 81%.** `ControlChain::is_ignored` accounts for 1,684M inclusive
  (`control.rs`). `glob_matches` accounts for 1,423M (`control/gitignore.rs`; 1,258M
  self), `ControlTable::chain_for` for 104M, and the per-child path split in
  `with_components` for about 76M.
- **The index build alone is the 257M** of the `--no-gitignore` run:
  - `derive_ext` and `intern_ext`: 66M;
  - the child sort: 34M;
  - `contribution` plus `merge`: 52M;
  - `malloc`/`free`: 48M.

### The Rules Are Mostly Lookups

The 1,593 rules split into:
- 1,301 basename patterns, of which 1,118 are literal names and 88 are `*.suffix`;
- 291 anchored patterns;
- one `**` pattern.

Each entry is governed by 110.9 rules from 1.70 sources on average.
The linear scan therefore costs about 111 patterns × about 140 instructions each,
roughly 19.6k instructions per entry, on the one thread that cannot scale.
The root `.gitignore` alone holds 37 `*.suffix`, 32 anchored, 26 literal (9 of them
negated) and 12 other glob patterns.

### The Tree Build Is Work pdu Skips

Balanced proxy, 200k entries:

| Job | Consumer instructions | Allocations | Minor faults | Peak RSS |
| --- | ---: | ---: | ---: | ---: |
| default tree | 523M | 1.40M | 15.0k | 66 MB |
| summary, `--no-gitignore` | 12M | 876k | 1.8k | 10 MB |
| tree, H172 proxy | 195M | — | — | 19.5 MB |

The tree consumer’s self costs, largest first:
- `malloc`/`free`, about 100M;
- `InternedRollUp::merge`, 83M;
- `derive_ext`, 70M;
- the `HashMap<PathBuf>` directory map, about 56M (H167’s target);
- `contribution`, 55M;
- sorting, about 50M;
- the `ExtTally` insert, 43M;
- `intern_ext`, 32M.

That is 2.6k consumer instructions and 7 allocations per entry, and 8 times the page
faults of the summary, for an index that no one-shot reader reuses.
After H159 no cross-thread frees remain; what is left is the build itself.

### A Compiled Matcher Is Not the Lever

`rg --files` walks in parallel and applies `.gitignore` through a compiled globset.
On `linux-v6.12` it took:
- 157–196 ms wall and 235–330 ms of user CPU with ignore rules on;
- 69–75 ms wall and 75 ms of user CPU with them off.

With `-j1` the times were 233–236 ms on and 73–76 ms off (screens, load 2–3). Ignore
handling costs ripgrep about 160 ms of CPU here, twice fdu’s linear matcher, and keeps
it at 2.2–2.7 times its own no-ignore wall even in parallel.
Some of that is per-directory matcher construction and per-candidate allocation rather
than matching, but the conclusion holds: compiling the rules does not by itself make
classification cheap.
What makes it cheap is doing less work per entry.

### Screens

These are wall-clock screens under load, so they give direction only:

- **`linux-v6.12` [load 2–3]:**

| Command | Wall |
| --- | ---: |
| fdu default tree | 207–218 ms |
| fdu `--no-gitignore` | 91–180 ms |
| fdu summary | 194–216 ms |
| pdu | 66–99 ms |
| diskus | 87–109 ms |

- **H171 prototype against its control [load 7, tests compiling]:**

| Job | Control | H171 | Change |
| --- | ---: | ---: | ---: |
| tree | 281–316 ms | 194–222 ms | −27 to −31% |
| summary | 231–378 ms | 194–225 ms |  |
| `--no-gitignore` (placebo) | 157–201 ms | 157–201 ms | none |

- **H172 proxy on top of H171 [load 4]** (the balanced summary took 0.25–0.29 s):

| Subject and job | Before | After | pdu |
| --- | ---: | ---: | ---: |
| `linux-v6.12`, tree | 146–187 ms | 119–158 ms | 101–119 ms |
| `linux-v6.12`, `--no-gitignore` | 117–131 ms | 103–119 ms |  |
| balanced, tree | 0.29–0.34 s | 0.29–0.34 s (unresolved) | 0.23–0.25 s |

## Proposal

### H171: Bucketed `.gitignore` Matching

At `Gitignore::parse`, each pattern goes into one of three buckets:
- **literal names:** a basename with no metacharacter, kept in
  `HashMap<name, Vec<index>>`;
- **suffixes:** a basename `*.suffix` with no other metacharacter, keyed by the text
  after its last `.` and confirmed with `ends_with`;
- **residual:** everything else.

`matches_components` then:
1. probes both maps with the entry’s name and extension;
2. scans the residual;
3. answers with the highest matching index.

Last-match-wins, negation and `directory_only` therefore hold exactly.
`ControlChain::is_ignored_within(dir_components, name, is_dir)` takes a directory split
once per listing.
Its callers are `push_directory` in `index.rs` and the cached parent in
`SummaryControls::classify` in `execution.rs`. The snapshot format does not change,
because matchers are rebuilt from the source bytes.
Grouping anchored (`Fixed`) patterns by segment count, in the same change, removes about
80M more. The prototype diff is in `fdu-sdul`’s notes.

- **Answers:** unchanged.
  The prototype gave 0 non-timing differences across 1,629,566 JSON leaves, and all 93
  control tests pass, including the `git check-ignore` verdict table.
- **Predicted effect:** consumer time from about 200 ms to 60–75 ms, below the 81 ms
  walk floor. The default tree would go from 211 to about 85–95 ms and the summary from
  167 to about 75–85 ms.
  That would put fdu 15–25% behind pdu, level with diskus, and level on
  `--no-gitignore`.

### H172: An Exact Transient Tree Tier

`RetainedState::Tree` applies when a request proves that nothing reads the index:
- a one-shot route;
- no index policy and no analysis;
- the views are `[Tree]`, unfiltered, with population Include;
- a minimum share above zero, with K = ⌈100/pct⌉ ≤ 65,536.

What the tier does:
- `push_directory` allocates entries for directories only.
- Each file is classified, then added to its parent’s `all` and `unignored` roll-ups.
  It is also offered to a global min-heap bounded at the K largest by the request’s size
  metric.
- A file that isn’t kept goes into its parent’s `folded` tally (entries, files, bytes,
  allocated, ignored).
  Its name stays in the listing, so the walker frees it (H159).
- `expand` adds the folded tally to the share omission.

The tier is exact: the share threshold is false for a zero total and requires part × 100
≥ pct × whole, so at most ⌊100/pct⌋ files can ever be rows.

- **Answers:** unchanged.
- **Predicted effect:** on the balanced tree the tree run spends 0.64 CPU-seconds above
  the 0.94 s summary. The tier removes about 63% of consumer instructions and about 70%
  of faults, about 0.4 CPU-seconds.
  That predicts 1.03–1.10 s, level with pdu at 1.02 s and diskus at 1.04 s within the
  interval. It would move ahead only with H167 or H166 on top.
  The 200k proxy could not resolve wall time under load, so this is a prediction.

### H173: The Live Residual Set, Carried Down the Walk

This is a conditional follow-up to H171, registered only if H171’s counters show the
residual still sets the wall time.
- Each directory keeps, for each governing source, the anchored and residual patterns
  whose leading segments still match its path: the NFA state after consuming the
  directory.
- A child file tests only live patterns with one segment left, and a child directory
  inherits the set advanced by one segment.
- An empty set means only the hash probes remain below that directory.

It needs no dependency, and it is the incremental form of evaluating a state machine
across full paths. A single automaton over every entry’s full path is not proposed:
- git’s precedence is not set membership: the deepest source with a match decides, then
  the last matching pattern in it;
- anchored patterns are relative to their own source;
- subset construction over these rules can grow large;
- once each entry costs two probes per source, the consumer is no longer on the critical
  path.

### Rejected Candidates

- **H164 first:** it would move 1.8G instructions onto walkers that do 30M each.
  After H171 the residual is about 0.5G, and H164 becomes the natural follow-up.
  It also needs a public `Op::Upsert` field for the summary route and budget admission
  on the walker side.
- **H166 and H170 for this gap:** they target walker parking, but the tree route is
  bound by the consumer (523M against 50M per walker).
  H170 applies only to the summary.
- **H167 and H168 alone:** 56M and about 100M of a 523M consumer.
  H168 is moot for folded files under H172.
- **Pruning ignored subtrees:** the default population is Include, so the default path
  has nothing to prune.
- **An allocator:** screened at −12% to −17%, but it adds a C dependency and raises peak
  RSS (H74). H172 removes the allocations instead.

## Open Questions

- **H172’s carrier:** a pruned `Index` with a folded tally on directory entries (about
  400–500 lines, plus 10 in `expand`), or tree nodes built outside the index (larger)?
- **K’s ceiling,** and whether `--sort name`, `mtime` and `count` keep the tier, given
  that eligibility depends on size only.
- **Tool comparison:** add `linux-v6.12` with `.gitignore` handling on to the published
  comparison as a product job, and state pdu’s depth there.
- **The `.gitignore` matching survey,** which reads ripgrep, git, gitoxide, Sapling,
  Mercurial, jj and libgit2 from source and benchmarks them against `git check-ignore`,
  was still running when this was written.
  Its result goes into `fdu-sdul` and may revise H171 and H173.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
