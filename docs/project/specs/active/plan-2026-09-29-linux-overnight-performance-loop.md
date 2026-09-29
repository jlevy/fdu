# Plan: The Linux Overnight Performance Loop

**Date:** 2026-09-29

**Author:** fdu project, with Claude Code

**Status:** Reviewed; queue ready to run.
Bead `fdu-fkyf` under epic `fdu-8a8r`. This plan reviews and extends
[the 0.2.2 Linux parity plan](plan-2026-09-29-linux-parity-0.2.2.md); where they
disagree, this plan records why and the 0.2.2 plan’s rows are updated to match.

## Overview

The goal is for fdu’s default command on Linux to beat pdu and diskus on real trees,
with `.gitignore` handling on and no answer changed.
This plan does three things:

1. It reviews the 0.2.2 plan against the whole performance record (H1–H173,
   exp-000–191), fdu’s Linux hot path, the source of eleven peer walkers, and seven
   `.gitignore` matchers.
2. It maps every performance attribute to the hypotheses that test it, with what the
   record already rules out.
3. It sets an ordered queue of pre-registered iterations, and the protocol, delegation
   policy and stop rules for running it unattended for about eight hours.

The review’s central finding is arithmetic.
With `.gitignore` on, fdu’s wall time here is set by one consumer thread, and the 0.2.2
plan’s two changes, by its own predictions, leave that thread slower than pdu’s whole
run.
The Fable review of this plan added the second regime: with `.gitignore` off the run
is already CPU-bound (3.3 cores busy), so once the consumer stops being critical, wall
time follows total CPU divided by about 3.4. Beating pdu therefore needs classification
made nearly free (H171 as revised here, with H175), the consumer’s index build shrunk
(H172 with H176), and then less CPU in the walk itself (H169’s in-place names, and a
per-listing name arena, H177). Tonight’s realistic target is the first two; the walk is
the next night’s.

## Goals

- The destination: fdu’s default `fdu PATH` faster than pdu’s default invocation
  (`pdu --silent-errors PATH`, the `pdu-default` tool contract) in a quiet paired tool
  cell on `linux-v6.12` (a real source tree with 358 `.gitignore` files) and on
  `node-modules-dense` (a real, directory-dense tree), and on `linux-balanced-1m`
  (generated, screening only).
- Tonight’s target: on `linux-v6.12`, the default command at or under 1.25 times pdu’s
  default (from 2.4 times at Q0, exp-175); on `node-modules-dense`, close the 11% gap
  measured at Q0 (exp-176).
- Change no answers: goldens, Python parity, the `git check-ignore` verdict table and a
  git differential over every path all stay identical.
- Record every cell, accepted or not, so the loop can resume from the record alone.

## Non-Goals

- A public API change.
  Anything that needs one (H164’s summary route) waits for 0.3.0.
- A new dependency in `fdu-core`, an allocator in the command line, or shipping PGO.
  These are maintainer decisions; the loop may screen them but never adopts them.
- New `.gitignore` semantics.
  `IGNORE_RULES_VERSION` does not move; the two git divergences found here (`fdu-ifci`)
  are fixed separately.
- macOS or Windows tuning.
  Platform-gated Linux code is linted with `make cross-lint`.
- Any release step.

## Review of the 0.2.2 Plan

Sources: the registry and ledger, the runbook, the campaign-2 plan, the floor report,
the pdu brief and the design study; fdu’s hot path read from source at `e5a71c8a`; peers
read from source under `attic/` (pdu 0.24.0, diskus, dut, bfs, gdu, dua, dust, jwalk,
the `ignore` crate, fastwalk, ncdu, GNU du); and git `dir.c`/`wildmatch.c`, ripgrep’s
`globset` and `ignore`, gitoxide, libgit2 and jj for matching.

**R1. The plan cannot reach its own goal.** Wall time is roughly fixed cost plus the
larger of the parallel walk and the serial consumer.
On `linux-v6.12` the consumer executes about 2.07G instructions with `.gitignore` on and
257M with it off (design study, callgrind); the walk alone, measured as the
`--no-gitignore` summary whose consumer is nearly idle, takes 68 ms on this host (screen
below), against pdu’s 64 ms and dut’s 54 ms.
H171’s prototype leaves 761M consumer instructions and predicts 85–95 ms; H172 targets
the generated tree. Neither moves the walk.
The goal needs the consumer below about 150M instructions and a faster walk.

**R2. H171 as prototyped leaves most of the remaining matching.** A rule model of the
kernel tree’s 1,593 rules (matcher review) finds the prototype still tests about 45
residual patterns per entry: the root file’s 32 anchored rules and about 11 wildcard
basename rules.
Two changes cut that to about 11: git’s general *ends-with* form (`*tail`
for any tail, not only `*.ext`), and anchored grouping by segment count made required.
Literal-prefix, suffix and minimum-length pre-checks leave about four full globs per
entry. The estimate is 0.4–0.7k instructions per entry against about 18k today.
H171 is semantically exact: last-match-wins equals highest-matching-index, as ripgrep’s
`Gitignore::matched_stripped` also uses.

**R3. H173 is not worth running on the kernel tree.** Carrying the live residual down
the walk saves 0.46 pattern tests per entry on top of revised H171, because the tree has
one `**` rule and 96.9% of entries see no live anchored rule.
It is demoted to a stateless refinement of `chain_for`, justified only by a `**`-heavy
subject.

**R4. H164 cannot ship in 0.2.2 as registered.** The summary route needs a new field on
the public `Op::Upsert` (`engine_contract.rs`), which is not `#[non_exhaustive]`. The
tree route’s `DetachedDirectory` is crate-private, so walker-side classification there
is 0.2.2-eligible. H164 is split: the tree route stays in the queue, and the summary
route moves to 0.3.0.

**R5. Several rows name the generated tree as deciding.** H166, H167, H169 and H172 do,
and the loop’s own rule forbids an accept decided on a generated subject.
Their deciding subjects are re-pointed to `node-modules-dense` and `linux-v6.12`, with
`linux-balanced-1m` as screening.

**R6. The index build is the next wall-setter, and the plan has one lever for it.** With
`.gitignore` off, the tree route’s consumer still spends 2.6–2.8k instructions and 7
allocations per entry: a name sort per listing, an extension `String` interned through
`BTreeMap<String>`, two one-entry `BTreeMap` roll-ups built, merged and freed per file,
a SipHash `HashMap<PathBuf>` per directory, and two full-arena passes after the walk.
Every one of these is a pure function of one listing except the merge into the parent.
H174 (new) moves that listing-local work onto the walker that read the listing, keeping
the consumer as the single writer.

**R7. The plan has no walker track, and fdu’s walk is slower than pdu’s here.** On this
host fdu spends 84 ms of user CPU and 174 ms of kernel CPU on the kernel tree with
`.gitignore` off, against dut’s 12 and 146. The syscall mix is identical to pdu and dut
except glibc `opendir`’s extra `fstat` per directory; the record already rules out raw
`getdents64`, narrow masks and io_uring as syscall-count levers (H71). What is left is
user space per entry (std’s `DirEntry` allocates a `CString` per name, then fdu copies
it twice), opening each directory by absolute path, and five runnable threads on four
vCPUs.

**R8. The standing numbers do not transfer to this host.** The 211 ms / 70 ms / 1.25 s
targets mix probe cells with hyperfine screens, two pdu depths and pre-0.2.1 builds, on
a different Firecracker kernel build.
Session-to-session drift on these hosts has reached 20–40%. Every claim tonight is
paired on this host, and the night starts with a re-baseline.

**R9. The harness can print ACCEPT with invalid samples.** `ledger.verdict` does not
check them; only exit code 3 does.
A cell that exits non-zero is not recordable.

**R10. The instruments cannot see the bottleneck directly.** No counter records consumer
busy against idle time, patterns tested per entry, or the freshness pass.
With counters on, each bump copies a 424-byte struct, including on every allocation, so
counter-run wall times are not comparable with counters-off runs.

**R11. The matcher survey (`fdu-p6vc`) is half done.** Its source-reading half ran in
this review. The benchmark half (instructions per entry per engine against
`git check-ignore`) remains; H171’s correctness does not depend on it, because H171 is
proven by a property test against the linear matcher and a git differential.

**R12. Two correctness findings, filed separately.** `statx` from Rust std omits
`AT_NO_AUTOMOUNT`, and the kernel’s `statx` path triggers automounts where `fstatat`
does not (`fs/stat.c`; `fdu-puk7`). fdu’s `.gitignore` parser does not skip a UTF-8 BOM
or truncate at an embedded NUL as git does (`fdu-ifci`).

## Where the Time Goes on This Host

Host: 4-vCPU Intel Xeon at 2.1 GHz, Firecracker guest, Linux 6.18.44-fc-v49, ext4 on
virtio, warm cache.
Screen of 11 runs per command at load 2.5, the 0.2.1 engine plus docs
(`e5a71c8a`); proportions only.

| Command, `linux-v6.12` | Wall (median) | User CPU | Kernel CPU |
| --- | ---: | ---: | ---: |
| `fdu PATH` (default tree) | 189 ms | 200 ms | 182 ms |
| `fdu PATH --view summary` | 149 ms | 212 ms | 171 ms |
| `fdu PATH --no-gitignore` | 78 ms | 84 ms | 174 ms |
| `fdu PATH --view summary --no-gitignore` | 68 ms | 79 ms | 170 ms |
| pdu 0.24.0 (default depth 10) | 64 ms | 53 ms | 176 ms |
| pdu `--max-depth 1` | 58 ms | 44 ms | 176 ms |
| diskus 0.9.0 | 68 ms | 69 ms | 186 ms |
| dut (source at `attic/dut`) | 55 ms | 12 ms | 147 ms |

`strace -c` on the same tree: every tool makes about 92.5k `statx`, 11.5k `getdents64`
and 5.8k `openat`/`close`. fdu and pdu add 5,773 `fstat` (glibc `opendir`); dut does
not. fdu adds about 4.5k `futex` calls.
Fixed start-up cost is small: 2.4 ms for fdu on an empty directory against 2.1 ms for
pdu.

An A/A cell (the same probe on both arms, uncontrolled, load 0.7) read `default-tree`
+8.26% [−0.02%, +14.10%]: one hundredth of a percent from a false signal.
Every verdict tonight therefore comes from the quiet regime.

## Attribute Map

Each performance attribute, what the record settles, and what tonight tests.
“Dead” means rejected with evidence that still applies; each cites its experiments in
the [registry](../../guides/performance-loop.md#hypotheses).

| Attribute | Settled by the record | Open, and tonight’s item |
| --- | --- | --- |
| Directory enumeration syscalls | Raw `getdents64`, bigger buffers, io_uring, terminating-call elision are dead on Linux (H71, H27; io_uring +327% warm) | Parent-relative `openat` was refuted on macOS only; H169 with in-place names (Q6) |
| Per-entry metadata | Narrow masks and `AT_STATX_DONT_SYNC` do nothing; `d_type` skip kept for the summary (H72) | Directory metadata from the opened fd (H169, Q6); automount flag (`fdu-puk7`, correctness) |
| Walker count and topology | Deeper APFS counts and adaptive controllers dead (H52–H99); `--threads 8` regressed a classification-bound default (H84, exp-149) | Walker count after classification is cheap, including `cores − 1` (H165, Q3) |
| Queue, parking and wakeups | Cutting consumer wakes alone does not move wall (H158) | Wake-one on extend and spin-then-park (H166, Q7) |
| `.gitignore` classification | Allocation-free matching and per-directory chains kept (H162, H163) | Bucketed matching, revised (H171, Q2); walker-side on the tree route (H164-tree, Q9); H173 demoted |
| Consumer index build | Per-file allocation trims alone fail on wall (H157 twice, H89, H13) | Exact transient tree tier (H172, Q4); walker-side listing digest (H174, Q5) |
| Retained memory | Transient summary cuts RSS 95–97% (H59, H161) | H172’s tree tier (Q4) |
| Allocation and allocator | Allocator adoption is the maintainer’s call (H74, H85); listing and batch recycle kept (H147, H159) | Removed at the source by H172 and H174 rather than by an allocator |
| Summary route | Worker-local reduction refuted on macOS (H62–H65) | Per-walker fold with the Linux cross-thread-free mechanism (H170, Q8), after reconciling with H62 |
| Teardown | Detached release of a large index kept (H156) | Measure the residual drop cost first; no item unless it is ≥3% |
| Build profile | PGO screened −8% on the pre-H162 engine (H148, stale) | Re-screen on the current engine (Q11); never shipped by the loop |
| Instrumentation | Counters cost under ~3.3% (exp-052, exp-053) | Consumer busy and idle time, patterns tested, freshness pass (Q1) |

## Amendments After the Plan Review

A Fable subagent reviewed this plan against the code before any candidate was measured.
Its findings are kept in
[the research brief](../../research/research-2026-09-29-linux-peers-matchers-and-hot-path.md);
the amendments it drove, adopted at 08:30 UTC, are listed here and take precedence over
the queue text below where they differ.

1. **Two regimes.** Wall time is roughly the larger of the consumer’s serial work and
   total CPU divided by about 3.4, plus a 5–10 ms tail.
   On `linux-v6.12` the controls-on tree runs about 2 cores busy (consumer-bound) and
   the blind tree 3.3 (CPU-bound).
   Predictions are stated in CPU milliseconds from the harness’s user and system time;
   instruction counts are the load-independent secondary.

2. **Q1 is deferred.** The walker attribution (`starved_ns`, `lock_wait_ns`, `send_ns`)
   already exists behind `FDU_SCAN_DIAGNOSTICS=1`; the pattern counters ride with Q2.

3. **Q2 gains H175.** Each listing’s control chain is derived from its parent’s instead
   of `ControlTable::chain_for` (about 104M consumer instructions on `linux-v6.12`), as
   a separate commit measured stacked on H171. Q2 carries two predictions: the revised
   buckets bring the controls-on tree to at most 1.15 times its `--no-controls` arm, and
   the prototype would reach only 1.45 times.
   The matcher stores indices, so `content_cost` stays an honest bound.

4. **Q3 is a screen.** Arms `--threads 3`, `4`, `6` and `8` on the Q2 head, after
   reading `starved_ns`. With the consumer no longer critical, `cores − 1` loses.
   A `PORTABLE` change needs the accept rule on both real subjects and a formulation
   that does not also change hosts with more cores, since a measurement is evidence
   about its own regime.

5. **Q4 is settled in design.** The carrier is a pruned `Index` with a
   `folded: FoldedChildren` element per directory, so there is one reader.
   The folded tally carries exactly what `record_omission` sums: entries, files, bytes,
   allocated, and ignored as an option.
   A folded index never escapes the one-shot route.
   H176 is part of the tier: it maintains only the reducers the requested views read, so
   no per-extension maps are built for a tree.
   The post-walk passes are fused.
   `linux-v6.12 default-tree` is a co-primary beside `node-modules-dense`. The review
   checked exactness against `ShareThreshold::admits` and the root total: sort keys,
   depth, breadth and ties at K all keep the tier.

6. **Q5 is conditional.** It runs only if, after Q4, the consumer is busy for more than
   80% of the walk. Its accept rule adds “total instructions non-increasing”, because
   moving work off the consumer can satisfy a consumer-only secondary while adding work.

7. **Q6 is phase 1 only.** It keeps absolute-path opens and adds raw `getdents64` into a
   reused 64 KiB buffer, names used in place, and
   `statx(dirfd, d_name, AT_SYMLINK_NOFOLLOW | AT_NO_AUTOMOUNT)`. `libc` is already a
   unix dependency of `fdu-core`.
   - Parent-relative opens are dropped: their fd budget is the breadth-first frontier,
     thousands of directories on the generated tree.
   - Directory attributes from the opened fd move to phase 2 (H179).
   - A `getdents64` failure declines the directory before any child is published, and
     the portable path re-reads it.
   - It needs 20 pairs, since the predicted effect is 10–15%.

8. **Q7 is gated** on `starved_ns`: under 2% of walker time closes it without a build.

9. **Q10 is folded into Q4.** Its path-clone piece is H51, refuted on macOS, and would
   have to say why Linux differs.

10. **Answer identity before every cell.** The probe’s oracle checks only root tallies.
    Before every cell, the product command line is byte-diffed in JSON and JSONL over
    all three subjects, with timestamps masked.
    H172 adds `--sort name`, `--min-share 0.1%`, `--depth 3`, `--breadth 5`,
    `--view summary` and `--view types`.

11. **Noise rules** (from exp-175’s false A/A accept).
    A candidate predicted below 10% is measured at 20 pairs, or recorded as a screen.
    A placebo that excludes zero by at most 3% is noted and does not block; one beyond
    3% blocks the verdict.

12. **Locks and gates.**
    - Builds take a shared `flock`, and cells take it exclusively.
    - At most two implementation worktrees at once.
      The orchestrator builds every release probe in the main checkout.
    - Per item, the gate is `cargo test -p fdu-core`, `make fmt-check` and
      `make clippy`. `make check` runs on the integrated head at the end of the night,
      and again if `unsafe` or platform-gated code landed.

13. **New hypotheses:**
    - H175, the chain from the parent;
    - H176, reducer elision by view;
    - H177, a per-listing name arena;
    - H178, the consumer walks when its channel is empty;
    - H179, directory attributes from the opened fd.

    H177–H179 are registered for the next night.

14. **Side-by-side profiling** (the maintainer’s request).
    Q0 adds callgrind, `strace` and CPU splits of fdu, pdu, dut and diskus on both real
    subjects. The loop can then name the work fdu does that the fastest peer does not.

## The Queue

Items run in this order unless a result says otherwise.
Each is pre-registered in the registry before its first timed sample, with a deciding
job, a deciding real subject, placebos and a prediction.
The accept rule is the loop’s: paired median at most −3% with the 95% interval below
zero on the pre-registered primary, no invalid sample, non-inferiority (upper bound
under +3%) on the named guard jobs, and answers identical.
Ids come from the 0.2.2 reservation: exp-175–186 and exp-192–199, then exp-200 onward;
new hypotheses take H174–H179.

### Q0. Re-baseline this host

A baseline, not a verdict (exp-175 and following).

- A quiet A/A cell on `linux-v6.12`, `default-tree` and `aggregate-summary`: both arms
  the same binary. It must include zero, and its interval width is tonight’s noise
  estimate.
- Quiet four-arm baselines of `e5a71c8a`: `default-tree` and `aggregate-summary`,
  controls on and `--no-controls`, on `linux-v6.12`, `node-modules-dense` and
  `linux-balanced-1m`.
- `make perf-floor` on `linux-v6.12` and `node-modules-dense`: no floor exists for the
  current engine on any Firecracker host.
- A quiet tool cell with the `fdu-default-tree` contract against pdu and diskus on
  `linux-v6.12` and `node-modules-dense` (part of `fdu-m3r6`). dut has no harness
  adapter and stays a screen.
- Callgrind, per thread, on the profiling probe: `default-tree` on both real subjects,
  controls on and off.
  This is the consumer budget every later item is judged against.

### Q1. Instrument the consumer

A methodology change, measured as a placebo.

- Counters: consumer busy and blocked time in the detached consumer loop, patterns
  tested per entry and bucket hits (used by Q2), and the freshness pass inside
  `detached_finish_us`.
- Placebo: counters off, the instrumented build against `e5a71c8a` on `default-tree`
  must include zero.

### Q2. H171, revised: bucketed matching with git’s pre-checks

Bead `fdu-sdul`. The prototype diff is in its notes; the revision follows the matcher
review.

- **Mechanism:** at `Gitignore::parse`, each pattern goes into one bucket:
  - literal basenames, in a map from name to the highest matching index, one for any
    entry and one for directories, holding indices rather than copied keys;
  - *ends-with* basenames (`*tail` for any literal tail), keyed by the tail’s last bytes
    and confirmed by `ends_with`;
  - anchored `Fixed` patterns grouped by segment count, then by first literal segment;
  - the residual, each with git-style pre-checks: literal prefix, literal suffix,
    minimum length.
- **Per entry:** hash the name once (an in-crate FNV, not SipHash), probe the maps,
  check the anchored group for the entry’s depth, then scan the residual in descending
  index order and stop below the best hit so far.
  `ControlChain::is_ignored_within(dir_components, name, is_dir)` splits each listing’s
  directory once. `**/seg` is rewritten as a basename rule at parse time.
- **Memory charge:** `content_cost` must still bound what the compiled matcher holds;
  storing indices keeps it within the existing charge, otherwise the charge and its
  boundary tests change on purpose.
- **Deciding:** `default-tree` and `aggregate-summary`, controls on, on `linux-v6.12`.
- **Placebos:** both arms `--no-controls` on `linux-v6.12`; `default-tree` on
  `linux-balanced-1m`, which has no `.gitignore`.
- **Secondary, load-independent:** consumer instructions down at least 50% (callgrind).
- **Prediction:** the controls-on `default-tree` falls to within 10% of its own
  `--no-controls` arm.
- **Tests:** keep the linear matcher under `cfg(test)`; a property test of random rule
  sets (literal, ends-with, anchored, negated, directory-only, escaped, `[...]`,
  non-UTF-8, trailing `.` and `*.`) against it; git’s `t3070-wildmatch` rows as a
  pattern-level table; a differential of fdu’s ignored set against
  `git check-ignore --no-index` over every path of `linux-v6.12`; the existing verdict
  table, route differential tests, goldens and parity.

### Q3. H165, revised: the walker count once classification is cheap

No engine change: the variants pass `--threads 3`, `4` and `6`.

- The consumer is the critical path and shares four vCPUs with four walkers.
  A `cores − 1` arm gives it a core.
- **Deciding:** `default-tree` on `linux-v6.12` and `node-modules-dense`, on top of Q2.
- **Guard:** no subject, including `linux-balanced-1m`, regresses past +3%.
- A change to `PORTABLE` needs both real subjects to clear the accept rule.
  This is the revisit H84’s row sanctions.

### Q4. H172: an exact transient tree tier

Bead `fdu-dp98`. A Fable design review settles the carrier first: a pruned `Index` with
a folded tally per directory, or tree nodes built outside the index.

- **Mechanism:** when a one-shot tree request proves nothing reads the index, retain
  every directory and the K largest files, with K = ⌈100/min-share⌉, and fold every
  other file into its directory’s totals and omission tally.
  The share threshold admits at most ⌊100/min-share⌋ files as rows, so the answer is
  exact.
- **Deciding** (re-pointed from the generated tree, R5): `default-tree` on
  `node-modules-dense`, and non-inferior on `linux-v6.12`.
- **Screening:** `default-tree` on `linux-balanced-1m`, with peak RSS.
- **Placebos:** `aggregate-summary --no-controls` and `cold-scan-index`.
- **Tests:** `transient_tree_equals_the_indexed_tree_under_every_bound_case` across
  worker counts, orders, shares, sort keys, limits, ties at K, zero-size trees and
  ignored files at the boundary; parallel equivalence; goldens and parity.

### Q5. H174 (new): a walker-side listing digest

- **Mechanism:** the work the consumer does per entry that depends only on one listing
  moves to the walker that read it, before the listing is sent.
  This covers the name sort and dedup, each file’s extension bucket, and the listing’s
  per-extension and total tallies.
  The consumer then interns a few extensions per listing, not one per file, and merges
  one pre-folded contribution into the parent.
  The consumer stays the single writer, since every value the walker computes is a pure
  function of its own listing.
- **Not** dut’s walker-side roll-up into parents (T4), which would break the single
  writer; not H157, which trimmed allocations in place.
- **Deciding:** `default-tree` on `node-modules-dense` and `linux-v6.12`, on top of Q4;
  consumer instructions down at least 30%.
- **Placebo:** `aggregate-summary`, whose route this does not touch.

### Q6. H169, revised: a Linux-native directory reader

Bead `fdu-leja`.

- **Mechanism:** a `cfg(target_os = "linux")` reader beside `macos_bulk.rs`, with an
  audited `unsafe` boundary like it.
  It opens each directory relative to its parent’s fd and reads `getdents64` into a
  reused per-walker buffer.
  Names are used in place, with no per-entry `CString`. It stats with
  `statx(dirfd, name, AT_SYMLINK_NOFOLLOW | AT_NO_AUTOMOUNT)`, and takes each
  directory’s own metadata from `fstat` on the fd it opened, rather than a `statx` from
  the parent. The portable path stays for `DT_UNKNOWN` and every error case.
- **Deciding:** `aggregate-summary --no-controls` on `node-modules-dense` and
  `linux-v6.12`, the jobs the walk bounds; `default-tree` non-inferior.
- **Secondary:** syscalls per `strace -c`, and user CPU.
- **Tests:** parallel equivalence against the portable reader; the permission and
  unreadable-directory fixtures; `make cross-lint`.

### Q7. H166: wake one walker per new directory

Bead `fdu-i6nk`.

- **Mechanism:** replace the unconditional `notify_all` on every `extend` with
  `notify_one` per directory added, and only when a walker is parked; add a short spin
  before parking.
- **Deciding:** `aggregate-summary --no-controls` on `node-modules-dense`; voluntary
  context switches down by an order of magnitude.
- H158 showed that fewer wakes alone need not move wall; the prediction here is about
  walker idle time, and a null result closes the item.

### Q8. H170: a per-walker summary fold

Bead `fdu-lz25`, only after a Fable review reconciles it with the H62–H65 refutations,
which were macOS measurements of a different mechanism.

### Q9. H164, tree route: classify on the walker that read the listing

Bead `fdu-emqf`, split per R4. It runs only if Q2–Q5 leave the consumer above the walk.

### Q10. A micro bundle

Measured once as a bundle, never item by item:
- the freshness pass fused into `finish()`;
- a pre-sized `.gitignore` read;
- the summary route’s per-entry path clone;
- the report’s per-child `PathBuf` and `String`.

It is dropped if the bundle cannot clear 3%.

### Q11. PGO re-screen

A screen of the current engine, trained on the real subjects.
It is recorded, and never changes the release profile (`fdu-pdne`).

### Q12. Standing and handoff

- Re-run the Q0 tool cells on the night’s head.
- Refresh the evidence report and the runbook’s Current Pickup.
- Run a final Fable review of the night’s record.
- Hand off.

## Loop Protocol

One iteration:

1. **Pre-register.** Write or amend the registry row and the bead: mechanism, deciding
   job and subject, placebos, prediction, accept rule.
   Commit it before any timed sample.
2. **Design review.** For any item that touches classification, the index
   representation, the walker–consumer protocol or `unsafe` code, a Fable subagent
   reviews the design against the design principles before code is written.
3. **Implement.** In a separate worktree with its own `CARGO_TARGET_DIR`, with the tests
   named in the item. `cargo test -p fdu-core`, `make fmt-check` and `make clippy` pass.
4. **Diff review.** A Fable subagent reviews the diff adversarially for answer changes,
   `.gitignore` semantics, concurrency and `unsafe` soundness.
   Findings are fixed or rebutted in writing before measuring.
5. **Build.** Release probes for control and candidate, copied to
   `/home/user/perf/bin/<sha>`; the control is the last accepted head.
6. **Measure.** Take the measurement lock, wait for the host to go quiet, run the cell.
   A non-zero exit is not recordable: rerun it once, quiet; a second failure is recorded
   as blocked.
7. **Record.** `perf-record` with the evidence copied under
   `docs/project/experiments/evidence/exp-NNN/`, then `perf-ledger` and `perf-report`.
8. **Decide and commit.** An accept lands in two commits, the change then the record
   naming it. A reject reverts the code and commits the record.
   Update the registry row, the bead, and Current Pickup.
9. **Gate and push.** `make check` (never during a cell), `tbd sync`, push.

**The measurement lock.** The file `/home/user/perf/MEASURING` exists for the whole of a
cell.
Every build or test command any agent runs goes through `/home/user/perf/guard.sh`,
which waits while the lock exists.
The orchestrator takes the lock, then waits until one-second CPU busy has stayed under
15% for ten seconds before starting the harness, whose own gate is 25%. Implementation
can then run in parallel with other implementation, but never with measurement.

## Delegation Policy

- **The orchestrator** (the main session) owns what makes the record trustworthy: the
  order of the queue, pre-registration, every measurement cell, every verdict,
  recording, commits, pushes, merges and beads.
  It never delegates a verdict.
- **Opus 5.5 subagents** do mechanical and well-specified work:
  - implementing a hypothesis from a written specification, in its own worktree;
  - writing the property, differential and conformance tests an item names;
  - fixing `make check`, clippy and format failures;
  - building probes and reconstructing subjects;
  - running callgrind or `strace` attribution and tabulating it;
  - drafting experiment bodies from run JSON;
  - updating registry rows, the runbook and bead notes, and running `make docs-format`.
- **Fable subagents** do deep technical, algorithmic and creative work, on the big items
  only (the maintainer’s decision, 2026-09-29):
  - a review of this plan before the loop starts;
  - a design review and an adversarial diff review of H171 (Q2), H172 (Q4), H174 (Q5)
    and the `unsafe` Linux reader (Q6);
  - interpreting a surprising or null result on one of those items;
  - one “what have we missed” sweep in the middle of the night, and the morning review
    of the night’s record.
- **Opus 5.5 reviewers** review the diffs of the smaller items (Q1, Q3, Q7, Q10), with
  the same adversarial brief.
- **Read-only Explore agents** find code when only the location is needed.

Every delegated build or test goes through the guard script, and every subagent prompt
states the lock rule, the worktree, and that it must not push, record, or touch beads.

## Unattended Operation

**Readiness**, checked before the loop starts:
- uv 0.12.1 (a PyPI install in a venv; astral.sh is blocked), Python 3.12 for the
  harness, the pinned Rust 1.97.1 and 1.85.0 toolchains with the cross-lint targets, and
  cargo-deny 0.20.2;
- pdu 0.24.0 and diskus 0.9.0 from `cargo install --locked`, and dut built from source;
- the three subjects rebuilt with their recorded counts: `linux-v6.12` (92,474 entries,
  358 `.gitignore`), `node-modules-dense` (9,439 directories) and `linux-balanced-1m`
  (digest `4bbd97c0`);
- an end-to-end harness cell, a push to the branch, and `tbd sync`, each run once;
- `make check` run once and passing in this container.

**Stop rules:**
- An item gets about two and a half hours from pre-registration to verdict.
  Past that it is parked with notes on its bead.
- A cell that fails the quiet gate twice is recorded as blocked, and the loop moves on.
- Two consecutive null results on the walker track (Q6, Q7) end that track for the
  night.
- A threshold is never lowered, and a metric is never switched after the fact.

**Resumption:** a self check-in is scheduled every hour.
When it fires, the loop reads this plan, the task list and
`tbd list --status in_progress`, and continues from the first unfinished step.

**Needs the maintainer** (the loop records and stops at these):
- any public API change;
- a new dependency or allocator;
- adopting PGO;
- changing `IGNORE_RULES_VERSION`;
- any release step.

**Decided by the maintainer before the run** (2026-09-29):
- The Linux-native reader (Q6) may add an audited `unsafe` boundary behind
  `cfg(target_os = "linux")`, on the `macos_bulk.rs` pattern, if it passes review and
  measurement.
- The Linux worker default in `PORTABLE` may change when the walker-count sweep (Q3)
  clears the accept rule on both real subjects with no subject regressing past +3%.
- All work lands on one branch in one draft pull request, each accept as a change commit
  followed by its record commit.
- The matcher survey `fdu-p6vc` is taken over by this loop.

## Status

Updated after every iteration.
Times are UTC on 2026-09-29; the run started at 08:10.

| Item | Bead | State | Result |
| --- | --- | --- | --- |
| Review and plan | `fdu-fkyf` | Done | This document; five source reviews and a Fable plan review |
| Environment | `fdu-fkyf` | Done | Tools pinned, three subjects rebuilt with their recorded counts, one harness cell, push, `tbd sync` and `make check` (19 min, pass) all run |
| Q0 re-baseline | `fdu-o4z5` | Cells done; side-by-side profiling running | exp-175: `linux-v6.12` default tree 181.9 ms (blind 89.9), summary 149.5 (blind 77.8), fdu 0.19 s against pdu default 0.079 s; exp-176: `node-modules-dense` default tree 84.0 ms, fdu 0.086 s against pdu default 0.077 s (−11%); exp-177: balanced 1.36 s tree, 1.23 s summary; a false A/A accept on the `linux-v6.12` summary set the noise rules |
| Q1 consumer counters | `fdu-hjo1` | Deferred | Walker attribution exists behind `FDU_SCAN_DIAGNOSTICS`; pattern counters ride with Q2 |
| Q2 H171 revised, with H175 | `fdu-sdul`, `fdu-hb0u` | Accepted, merged `2379233a` | exp-178: `linux-v6.12` default tree −29.62%, summary −25.45%; exp-179: H175 another −3.31% on the tree; placebos at zero; glob evaluations 110 → 0.0019 per entry; answers identical to the base and to git |
| Matcher survey | `fdu-p6vc` | Source half done | Findings in the bead’s notes; benchmark half queued after Q2 |
| Q3 H165 walker count | `fdu-c11z` | Screened, rejected | exp-182: three walkers +10.25% and +21.95%; six and eight not better on both real subjects; `PORTABLE` unchanged |
| Standing after Q4 | — | Measured 11:00 | Quiet tool cells on the H172 head: `linux-v6.12` fdu 0.084 s against pdu default 0.078 s (was 0.19 against 0.079 at Q0) and diskus 0.079 s; `node-modules-dense` fdu 0.076 s against pdu default 0.077 s and diskus 0.075 s (evidence under exp-180). Tonight’s 1.25× target is met |
| Q4 H172 tree tier, with H176 | `fdu-dp98`, `fdu-dnfs` | Accepted, merged `0228ea42` | exp-180: `linux-v6.12` default tree −13.48% (91.8 → 80.2 ms); exp-181: `node-modules-dense` −10.30%; balanced screen −3.20% wall, peak RSS −79%; placebos at zero |
| Q5 H174 listing digest | `fdu-sfse` | Queued | — |
| Q6 H169 native reader | `fdu-leja` | Accepted (phase 1), merged `217861c1` | exp-185/186: `--no-controls` summary −6.25% and −7.90%, default summary −9.7% and −9.4%; default tree −4.29% on `node-modules-dense`, −2.06% (not clearing) on `linux-v6.12`; four audited `unsafe` expressions; public diagnostics fields deferred (`fdu-q7hf`) |
| Q7 H166 wake one | `fdu-i6nk` | Closed by its gate, no build | Walkers starved 2.2% (`linux-v6.12`) and 0.6% (`node-modules-dense`) of their time |
| H180 summary walker trims | `fdu-cfbf` | Accepted, merged `c2a75fe4` | exp-183: `node-modules-dense` default summary −8.68%; exp-184: `linux-v6.12` −5.63% (blind −12.76%); tree placebos at zero |
| H181 conditional wakes, H182 hash sort | `fdu-uk0u`, `fdu-sp4m` | Rejected (exp-192), not merged | Bundle cell: +0.20% on `linux-v6.12`, −1.40% on `node-modules-dense`; the wake syscalls and consumer instructions fell, wall did not |
| Standing after H169 | — | Measured 12:40, 20 pairs | `linux-v6.12` fdu 0.077 s against pdu default 0.072 s (−8%) and diskus 0.073 s; `node-modules-dense` fdu 0.080 s against pdu default 0.081 s (+1%) and diskus 0.082 s (+1%): parity on the dense tree (evidence under exp-185) |
| H183 matcher pre-checks | `fdu-7ydi` | Implementing (branch `perf/h183-matcher-prechecks`) | A callgrind of the head found `memcmp` (114M) and `Checks::admit` (97M) are most of the 333M instructions `.gitignore` still costs the consumer on `linux-v6.12` |
| Q8 H170 summary fold | `fdu-lz25` | Queued | — |
| Q9 H164 tree route | `fdu-emqf` | Conditional | — |
| Harness invalid-sample verdict | `fdu-c2c6` | Fixed, `8a58d432` | INCONCLUSIVE for any invalid sample; the recorder refuses such an accept |
| Automount flag | `fdu-puk7` | Filed | Folded into Q6 on Linux |
| BOM and NUL divergence | `fdu-ifci` | Filed | Not tonight: it changes answers |

## Open Questions

- The second real subject heavy with wildcard rules.
  No checkout in `attic/` is large enough; candidates are a large monorepo with a built
  output tree beside it.
- H172’s carrier and K ceiling (the Q4 design review).
- Whether H174 should also carry each listing’s classification on the tree route.
  That would fold Q9 into Q5.

## References

- [0.2.2 plan](plan-2026-09-29-linux-parity-0.2.2.md) and
  [design study](../../research/research-2026-09-29-linux-default-tree-point-solution.md)
- [pdu brief](../../research/research-2026-09-28-pdu-and-the-linux-peer-gap.md)
- [Performance loop](../../guides/performance-loop.md) and
  [runbook](../../guides/performance-loop-runbook.md)
- [Design principles](../../architecture/fdu-design-principles.md) and
  [engine architecture](../../architecture/fdu-engine-architecture.md)
- Beads: `fdu-fkyf`, epic `fdu-8a8r`, `fdu-sdul`, `fdu-dp98`, `fdu-emqf`, `fdu-c11z`,
  `fdu-i6nk`, `fdu-leja`, `fdu-lz25`, `fdu-p6vc`, `fdu-puk7`, `fdu-ifci`.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
