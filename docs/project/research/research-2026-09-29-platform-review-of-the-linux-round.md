# Research: Platform Review of the Linux Overnight Round, and the macOS Benchmarks to Run Next

**Date:** 2026-09-29

**Author:** fdu project, with Claude Code

**Status:** Complete for the source reading; the macOS cells it plans (§7) have not run.
It reviews the six changes the
[2026-09-29 Linux overnight round](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md)
accepted in [#161](https://github.com/jlevy/fdu/pull/161), at `0c8131fd` against the
pre-round engine `e5a71c8a`: which platforms each runs on, what it should do on macOS,
and what must be confirmed there before 0.2.2 claims anything about macOS. It makes no
new timing claim: every figure is cited from a recorded cell, and anything marked
**[inference]** is reasoned from mechanism or from earlier records, not read in the code
or measured. Every `file:line` citation is at `0c8131fd`. Epic: `fdu-9q80`.

## 0. Findings in brief

- Of the six accepted changes, **one is Linux-only** (H169 phase 1, and only on glibc).
  **Five are portable** and run on macOS, Windows and musl: H171, H175, H172 (with H176
  and F6e), H180 and H183. **None is macOS-specific.** None of the four rejected
  hypotheses shipped any code.

- On macOS, four accepted changes are on the default `fdu PATH` hot path: H171, H175,
  H172 and H183. `fdu --view summary` runs three: H171, H180 and H183. H169 is not
  compiled on macOS.

- **None of the ten was measured on macOS.** Every cell ran on one 4-vCPU Firecracker
  guest: Linux 6.18.44-fc-v49, ext4 on virtio, warm cache, x86_64 glibc.
  So the five portable changes change macOS’s default behaviour on Linux evidence alone.
  The biggest of these is H172: the plan `fdu PATH` takes on every platform is now a
  folded index instead of the full one.

- **No tuning constant in `platform_tuning.rs` changed.** H165 was rejected, and
  `platform_tuning.rs` is not in the round’s diff.
  So no `Tuned::measured` value is newly inherited.
  What is inherited is a default *plan* (H172) and a default *matcher* (H171, H183). By
  the spirit of `platform-tuning.md` ("a default claiming both states needs a
  measurement in both"), macOS needs a confirmation before the release claims anything
  there.

- **Expected effect on macOS, from the macOS record:**
  - The macOS default tree is kernel-bound.
    User CPU is about 3–12% of total CPU:
    - 207 of 2,064 ms on metabrowser (exp-083–088);
    - 209 of 1,787 ms on rustup (exp-066);
    - 1.3 of 37.7 s on the generated million-entry tree (exp-165).
  - Five to six cores stay busy with six walkers on eight performance cores, so the
    consumer does not compete with the walkers for cores.
    On the 4-vCPU Linux host it did: four walkers and one consumer shared four vCPUs.
  - Earlier macOS cells cut user CPU 36–51% and moved wall by less than 2%: H62 and H63
    (exp-041, exp-042), and H51 (exp-016).
  - Prediction: **large peak-RSS cuts** (H172), **large user-CPU cuts** (H171, H183),
    and **small wall changes, 0% to −12%**. The largest wall change should be on a tree
    with many `.gitignore` rules, such as the kernel tree cloned onto APFS.
  - No change has a credible mechanism for a macOS wall regression beyond noise.

- **Correctness.** Matching was byte-exact and case-sensitive on every platform before
  the round (`control.rs:18-19`, `control/gitignore.rs:6-8`), and it still is.
  One answer did change on Linux glibc, and on purpose: on a tree holding an unmounted
  autofs trigger directory, 0.2.1 mounted the trigger by statting it and reported the
  mounted root; the round reports the trigger, as `lstat` and GNU `du` do.
  The review of #161 (R161-2) found that H169 phase 1 had changed it on the parallel
  walk alone, so the same request answered differently by route; `fdu-d2fn` then made
  every route list through the reader and pass `AT_NO_AUTOMOUNT` on every stat of a
  listed child, so the answer no longer depends on route, worker count, or which reader
  served a directory. The walk root is resolved on every route.
  No other answer changed by construction.
  - H171 and H183 compare the same bytes the linear matcher compared, so
    case-insensitive APFS and NFD names get exactly the answers they got before.
  - A pre-existing difference is newly worth documenting: git on macOS defaults
    `core.ignorecase=true` and `core.precomposeunicode=true`, and fdu follows neither.
  - The `.GITIGNORE` spelling rule is unchanged by H180 (`scan.rs:3964`,
    `control.rs:925-945`).
  - Windows sees WTF-8 bytes through `Path::components`, as before.

- **A shipping gap that is not on macOS.** The manylinux2014 **aarch64** wheel ships
  H169’s `unsafe` reader, which has never run in CI:
  - CI tests only on x86_64 `ubuntu-latest` (`ci.yml:61-68`);
  - the aarch64 wheel is not smoke-tested (`release.yml:240-244`, `native: false`);
  - `CROSS_TARGETS` does not include `aarch64-unknown-linux-gnu`.

- **Does `getattrlistbulk` already cover H169?** Yes for every syscall-level piece, and
  more:
  - no `fstat` per directory;
  - no stat call per entry;
  - a reused 64 KiB buffer.

  It does not cover “names used in place”: `macos_bulk` allocates an `OsString` per
  entry and a `Vec` per directory, and the detached route then copies each name again.
  H179 has no macOS target, because the parent’s bulk record already carries each
  directory’s attributes.
  H169 phase 3, parent-relative opens, was refuted on macOS twice (exp-024, exp-038).

## 1. How the default command runs, per platform

**`fdu PATH` (default tree, `.gitignore` on).** The path is the same on every platform
up to the directory reader:

1. `plan()` picks `RetainedState::Tree` for a one-shot, unfiltered tree whose share
   bounds the rows it can show (`execution.rs:507-516`, `TreeRetention::for_request` at
   `execution.rs:78-99`).
2. The Tree arm (`execution.rs:660-686`) calls `scan_into_folded_index`
   (`scan.rs:4992`), then `scan_detached_directories` (`scan.rs:4879`) and
   `scan_concurrent_detached` (`scan.rs:2676`).
3. Each walker runs `walk_detached_worker` (`scan.rs:3384`), which calls
   `walk_worker_with` (`scan.rs:3450`).
4. **The directory reader is the only per-platform step:**
   - macOS: the `getattrlistbulk` reader, `cfg(target_os = "macos")` (`scan.rs:44-46`,
     `3467-3468`, `3482-3517`);
   - Linux glibc: the `getdents64`/`statx` reader,
     `cfg(all(target_os = "linux", target_env = "gnu"))` (`scan.rs:49-51`, `3469-3470`,
     `3526-3575`);
   - everything else, and any directory a native reader declines: portable `read_dir`
     from `scan.rs:3577`. On Windows it takes metadata through `windows_metadata`
     (`scan.rs:53-55`).
5. Each entry goes through `DetachedEmission::record_entry`, then
   `record_detached_entry` (`scan.rs:3691`). It copies the name once:
   `name.to_os_string()` at `scan.rs:3730`.
6. On the single consumer, `DetachedIndexBuilder::push_directory` (`index.rs:1768`):
   - sorts the listing by name (`index.rs:1813`);
   - derives the control chain (H175, `index.rs:1838-1844`);
   - classifies each child (H171 and H183, `index.rs:1860`);
   - folds files (H172, `index.rs:1864-1871`, `offer` at `index.rs:1695`,
     `keep_largest_files` at `index.rs:1920`).
7. The report is built from the folded index, which is then released
   (`execution.rs:684`). The release runs inline below 64k entries and always on Windows
   (`lib.rs:294`, `lib.rs:318`).

**`fdu --view summary PATH`.**

1. `RetainedState::Summary` runs `walk_worker_transient_fold` (`scan.rs:3431`), whose
   emission is `StreamingEmission`.
2. Each entry goes through `record_walk_entry` (`scan.rs:3823`), then
   `prepare_walk_entry_reading` (`scan.rs:3772`). This is where H180 applies.
3. On the consumer, `SummaryControls` resolves the chain with `chain_for`, not H175’s
   derivation. It classifies with `SplitDirectory` and `is_ignored_within`
   (`execution.rs:896-910`), which is where H171 and H183 apply.

**Retained, opened, watch and refresh routes.**

- `ControlMatcher::is_ignored` calls `Gitignore::matches` (`control.rs:771`), which
  calls `decide`, so H171 and H183 apply.
- Opened discovery calls `prepare_walk_entry` (`opened.rs:1322`), so H180 applies.

**Worker pools.**

- The tuning table is chosen with `cfg(target_os = "macos")`
  (`platform_tuning.rs:192-199`): `MACOS` is measured (`platform_tuning.rs:123-131`),
  and `PORTABLE`, used by Linux, Windows and every other target, is inherited
  (`platform_tuning.rs:149-157`).
- On an M1 Pro (10 cores) the automatic pool starts six walkers and may expand to 16
  after calibration (`scan.rs:2590-2608`).
- On the Linux host, four walkers started, with a maximum of eight.

## 2. Change by change: the accepted six

### H171: bucketed `.gitignore` matching

- **Code:** `control/gitignore.rs`:
  - the rule index is documented at `81-114`;
  - `decide` is at `184-270`;
  - `RuleIndex::build` is at `616-680`;
  - `SplitDirectory` and `is_ignored_within` are in `control.rs:820-883`.
- **Platforms:** all of them.
  There is no `cfg` in the non-test code.
- **Call paths:** every route that classifies:
  - the tree route (`index.rs:1860`);
  - the summary route (`execution.rs:909`);
  - the retained, opened, watch and refresh routes (`control.rs:771`).
- **On macOS’s default hot path:** yes.
- **Expected effect on macOS:**
  - The instruction cut is the same on any ISA: 110 rules tested per entry falls to
    0.0019 on `linux-v6.12`. It is a function of the rules and the names only.
  - Whether it shortens wall depends on whether the consumer is on the critical path.
    On macOS the walk costs several times its ext4 cost per entry (`platform-tuning.md`:
    APFS is about 3.5×). Six walkers run on eight P-cores beside one consumer, so the
    consumer’s saving becomes wall only through the end-of-walk backlog **[inference]**.
  - Prediction for the kernel tree on APFS: default-tree wall −2% to −12%, user CPU −35%
    to −65%. For metabrowser, with 59 `.gitignore` files and fewer rules per entry: wall
    0% to −6%.
  - It has no effect on a tree with no `.gitignore`: `classifying` is false
    (`index.rs:1845`), so `Name::new` never runs.
- **Could it regress on macOS?** Only on a tree whose chain holds a few rules.
  - There, `Name::new` hashes each name twice (FNV of the whole name and of its
    extension) and builds a byte-class set (`gitignore.rs:355-381`). That could cost a
    few tens of instructions more per entry than a linear test of two or three rules.
  - With user CPU at about 10% of the macOS total, the bound is well under 1% of wall
    **[inference]**.
  - There is no tuned threshold.
    `in_order` is used only past `u32::MAX` rules (`gitignore.rs:616-620`).
- **Correctness by platform:**
  - Matching is byte-exact and case-sensitive regardless of `core.ignorecase`
    (`gitignore.rs:6-8`, `control.rs:18-19`). The literal-name and extension buckets
    hash the exact bytes, so `*.o` does not match `x.O` on case-insensitive APFS or
    NTFS, as before.
  - **Unicode normalization:** APFS returns names as stored; the bulk reader test
    creates an NFD name (`macos_bulk.rs:362`). A rule written in NFC therefore does not
    match an NFD file. That is unchanged: the property test holds the buckets to the
    linear matcher on the same bytes.
    But git on macOS defaults to `core.precomposeunicode=true`, so fdu and git can
    disagree there. This is undocumented; the only mentions of normalization are in
    `watch.rs:137` and `watch.rs:1035`. Any macOS differential against
    `git check-ignore` must pin
    `-c core.ignorecase=false -c core.precomposeunicode=false`.
  - **Windows:** components come from `Path::components()` with `Prefix` and `RootDir`
    dropped (`gitignore.rs:44-56`). Names are WTF-8 through `as_encoded_bytes()`. This
    is as before.

### H175: control chains derived from the parent’s

- **Code:** `ControlTable::chain_below` (`control.rs:556-569`); the builder’s
  `directory_ids: HashMap<PathBuf, (EntryId, Arc<ControlChain>)>` (`index.rs:1627`); the
  derivation (`index.rs:1838-1844`), guarded by a `debug_assert` against `chain_for`.
- **Platforms:** all. It is not `cfg`-gated.
- **Call path:** only the detached builder, which serves the tree route (folded or full)
  and `scan_into_index`. The summary route still calls `chain_for` per parent change
  (`execution.rs:902`).
- **On macOS’s default hot path:** yes.
- **Expected effect on macOS:** a few percent of consumer instructions on
  control-bearing trees.
  It was about 104M instructions on `linux-v6.12`, a quarter of the consumer after H171.
  On macOS it should be invisible in wall.
- **Could it regress?** No mechanism beyond one `Arc` clone per directory on one thread.
- **Windows and correctness:** the `by_directory` keys and the builder’s keys both come
  from the same walker’s `join`, so they are spelled the same.
  The `debug_assert` runs on the Windows and macOS CI test jobs (`ci.yml:61-68`).

### H172 with H176 and F6e: the exact transient tree tier

- **Code:**
  - `TreeRetention` (`execution.rs:48-99`), whose `MAX_FILES = 65,536` at
    `execution.rs:66` is a memory bound, not a timing constant;
  - the plan (`execution.rs:507-516`) and the Tree arm (`execution.rs:660-686`);
  - `TreeFold` and `offer` (`index.rs:1643-1716`);
  - the fold path (`index.rs:1864-1871`) and `keep_largest_files` (`index.rs:1920`);
  - `detached_builder` and `scan_into_folded_index` (`scan.rs:4866`, `scan.rs:4992`);
  - `admitted_parts_bound` in `query/query_selection.rs`.
- **Platforms:** all. `the_default_request_of_every_surface_plans_the_folded_tree`
  asserts that the default requests of the command line and of Python both take the
  tier. The perf probe’s `default-tree` mode builds the same tree request
  (`examples/perf_probe.rs:988-1003`).
- **On macOS’s default hot path:** yes.
  This is now the plan every platform’s `fdu PATH` takes.
- **Expected effect on macOS:**
  - **Peak RSS** is a property of the data structure, not the OS. On Linux the counters
    showed the kernel tree’s index falling from 92,473 to 5,930 entries and from 100.8
    MB to 20.7 MB allocated.
    On the generated million-entry tree, peak RSS fell from 306 to 64 MB. Predicted on
    macOS:
    - `macos-balanced-1m`: 281–294 MiB to about 55–80 MiB;
    - rustup-toolchains: the default tree was 63–77 MiB in `fdu-syyl` and should fall to
      about 15–25 MiB.
  - **Wall:** on Linux the dense-tree win (−10.3%) came from freeing a contended vCPU.
    macOS has spare P-cores, so the prediction there is 0% to −5% on real trees and 0%
    to −3% on the generated tree.
    The Linux generated-tree screen was only −3.2%, because “the walk’s CPU sets the
    time” (exp-180).
- **Could it regress on macOS?** One new piece of work lands on the walkers, which are
  macOS’s critical path:
  - Folded file names now stay in the listing, and the walker that allocated them frees
    them in `collect_returned` (`scan.rs:3275`). Before, the names moved into the index
    and were freed on the background release thread.
  - The bound is one free per file, about 20–30 ns on libmalloc’s nano allocator, spread
    across six walkers. That is about 0.4 ms on the kernel tree and about 4 ms of 7.2 s
    on the million-entry tree **[inference]**.
  - The frees are timed as handoff (`scan.rs:3363`), so the adaptive worker calibration
    does not see them.
  - H159, the same mechanism of freeing on the walker, was neutral on macOS (exp-167).
- **Peak RSS is also the proof that macOS took the tier.** The plan is not visible in
  any output (`execution.rs` test comment), so peak RSS should drop by at least half
  whenever the tier runs.
- **Correctness by platform:**
  - Kept files are put back in the order `name.cmp` gives (`index.rs:1940-1947`), the
    comparator the full path uses (`index.rs:1813`). That is byte order on Unix and
    WTF-8 order on Windows, the same on both routes.
  - Ties at K cannot change an answer (`KeptFile` docs).
  - APFS allocated sizes, which can be 0 for compressed files and are 4 KiB-quantized,
    only create ties.
  - On Windows the index is always released inline (`lib.rs:318`, `cfg!(windows)`). H172
    therefore shortens a synchronous drop on Windows’s critical path.
    It is plausibly a Windows gain, but Windows is unmeasured **[inference]**.
  - The real-tree byte diff of text, JSON and JSONL (171 comparisons) ran on Linux only.
    The macOS and Windows CI jobs run `cargo test` and the golden corpus
    (`ci.yml:61-110`) on small fixtures.

### H180: the summary route’s walker trims

- **Code:**
  - `join_listed_name` (`scan.rs:3815`);
  - `read_named_control_op` (`scan.rs:3964`);
  - moving the path instead of cloning it (`scan.rs:3885-3890`).
- **Platforms:** all.
- **Call paths:** `prepare_walk_entry_reading` and `record_walk_entry`, which serve
  `StreamingEmission` (the transient summary and the retained streaming walk) and opened
  discovery (`opened.rs:1322`). **It does not touch the detached tree route**:
  `record_detached_entry` already tested the name bytes and joined only control names
  and directories (`scan.rs:3691-3736`).
- **On macOS’s default hot path:** no.
  It is on `fdu --view summary`.
- **Expected effect on macOS:**
  - Walker user CPU falls by the `Path::file_name` parse and the `realloc` saved per
    entry, about 1k instructions per entry on Linux.
  - Wall should not move.
    H51 (exp-016, −0.44%) and H62/H63 (exp-041, exp-042) cut user CPU on macOS walkers
    without moving wall.
    Predicted −2% to +2%.
- **Could it regress?**
  - H51’s macOS signature was peak RSS and minor faults up about 4%, from moving the
    path instead of cloning it.
  - H180 moves an exact-capacity buffer (`join_listed_name` allocates
    `rel_dir + 1 + name`), so the batch holds the same bytes a clone would.
    H51’s mechanism should not recur **[inference]**. Watch minor faults and RSS on
    `aggregate-summary --no-controls`.
- **Correctness by platform:**
  - `read_named_control_op` asks `control_spelling(name)` of the listed name, and a
    `debug_assert` holds that name to be the joined path’s last component.
    That is exactly what `path_control_spelling` did (`control.rs:925-945`).
  - The case-insensitive `.GITIGNORE` variant still resolves by a lookup of the
    canonical path.
  - `a_listed_name_joins_as_path_join_does` runs on all three CI operating systems, so
    Windows’ `PathBuf::push` separator rules are tested.

### H169 phase 1: the Linux-native directory reader

- **Code:** `scan/linux_dents.rs`, gated
  `cfg(all(target_os = "linux", target_env = "gnu"))` at `scan.rs:49-51`, `3469-3470`
  and `3526-3575`.
  - Opens use `O_DIRECTORY | O_NOFOLLOW` (`linux_dents.rs:218-223`).
  - Stats pass `AT_SYMLINK_NOFOLLOW | AT_NO_AUTOMOUNT | AT_STATX_SYNC_AS_STAT`
    (`linux_dents.rs:47-48`).
  - Listings borrow their names from the reader’s buffer (`linux_dents.rs:112-138`).
- **Platforms:**
  - **Linux glibc only.** That covers the x86_64 and aarch64 manylinux wheels.
  - Not musl: `libc` has no `struct statx` there (`scan.rs:48`).
  - Not macOS, not Windows.
- **Correctness by platform:**
  - On glibc the reader’s `AT_NO_AUTOMOUNT` is an answer change against 0.2.1 on autofs
    trees, and after `fdu-d2fn` it holds on every route: the serial walk, revalidation,
    reconciliation and opened discovery list through the reader too, the directories it
    declines and the paths a route verifies by itself are stated by path with the same
    flags (`linux_dents::stat_path`, `scan::observe_path`), and the walk root alone is
    resolved through an opened descriptor (`scan::root_device`), under glibc and musl
    alike. `strace` of every probe route shows every tree-entry `statx` carrying the flag
    (exp-196).
  - musl never mounted by a stat: std’s `DirEntry::metadata` and `symlink_metadata` are
    `fstatat` and `lstat` there (`library/std/src/sys/pal/unix/fs.rs` at 1.85.0 lines
    95–109 and 915, `library/std/src/sys/fs/unix.rs` at 1.97.1 lines 109–123 and 1080),
    which the kernel treats as passing the flag.
  - macOS: `fdu-bida`, below, is still open.
- **On macOS’s default hot path:** no; it is not compiled.
  - The macOS engine at the H169 merge (`217861c1`) is functionally the engine at
    `c2a75fe4`. The merge’s non-Linux changes are `cfg(test)` widening
    (`scan.rs:1482-1491`), doc comments, and the `counters.rs` doc text.
    That makes it a **natural placebo step for a stacked macOS cell** (§6).
- **Risk on other platforms:**
  - musl builds take the portable path, and `make cross-lint` now checks
    `x86_64-unknown-linux-musl`.
  - **aarch64 glibc ships it untested.** It is compiled in the manylinux2014 aarch64
    wheel (`release.yml:240-244`), but that job does not run the smoke tests, CI’s test
    matrix is x86_64 only, and cross-lint omits the target.
  - The record layout and the syscall numbers exist on aarch64, and the 256-byte `statx`
    assertion is arch-neutral, so the risk is medium-low.
    Still, it is an `unsafe` boundary that has never executed on that architecture.

### H183: cheap matcher pre-checks

- **Code:**
  - `byte_class` (`gitignore.rs:384-386`);
  - `Name::classes` (`gitignore.rs:355-381`);
  - `Checks::admit` (`gitignore.rs:589`);
  - `admits` and `holds_literals` (`gitignore.rs:301-316`).
- **Platforms:** all. There is no `cfg` and no `target_arch`.
- **On macOS’s default hot path:** yes, on trees whose residual rules are wildcards.
- **Expected effect on macOS:** consumer instructions fell 436M → 238M on `linux-v6.12`,
  and matching’s `memcmp` 101M → 0. Wall on macOS moves by the share the consumer holds
  of the critical path, as for H171.
- **Could it regress?**
  - The survivors compare with inline byte loops instead of libsystem’s NEON `memcmp`.
    The rules are short and most are rejected before any comparison, so there is no
    material mechanism **[inference]**.
  - The instruction counts are x86_64 callgrind figures and do not carry over to arm64
    as numbers. callgrind does not run on Apple Silicon.
- **Correctness:**
  - Every check is a necessary condition; an independent review ported the checks and
    found no false rejection in 576,000 rule/name pairs.
  - The fold covers all 256 byte values, so non-ASCII UTF-8 and WTF-8 bytes are classed
    like any other byte.

## 3. The rejected four

| Hypothesis | Code shipped | Where it would have applied | macOS view |
| --- | --- | --- | --- |
| H165: walker count 3, 6 or 8 (exp-182) | None. It was a `--threads` screen | `PORTABLE.scan_threads_cap` (`platform_tuning.rs:150`): Linux, Windows and every non-macOS target, **never macOS**, which reads the measured `MACOS` table | Already settled on APFS by H52, H57 and H65 (exp-025, exp-036, exp-043). Nothing to do |
| H181: conditional queue wakes (exp-192) | Not merged | Portable (`DirectoryQueue` on std `Condvar`) | Probably a no-op on macOS as well: std’s `Condvar` there is backed by `pthread_cond`, whose broadcast with no waiter should not enter the kernel **[inference]**. It did not move Linux wall either |
| H182: hash-ordered listings (exp-192) | Not merged | Portable (the consumer’s sort) | APFS lists entries in hash order too **[inference]**, so the consumer sorts the same amount of work. There is no macOS-specific reason to expect a different outcome |
| H166: wake one walker (closed by its gate) | Never built | Portable | Its gate, walkers starved under 2% of the time, was read only on Linux. `starved_ns` is portable (`FDU_SCAN_DIAGNOSTICS=1`), so cell M7 below re-reads it on macOS. Re-open for macOS only if walkers starve for 2% or more |

## 4. The table

All measurements: one 4-vCPU Firecracker guest (Intel Xeon at 2.1 GHz, Linux
6.18.44-fc-v49, ext4 on virtio, warm, quiet, x86_64 glibc).
Cells are 20 pairs unless noted.

| Change | Platforms it applies to | macOS default `fdu PATH` | Measured where (all Linux) | Expected macOS effect | Risk on macOS |
| --- | --- | --- | --- | --- | --- |
| H171 bucketed matching | All (Linux glibc and musl, macOS, Windows); every route | Yes | exp-178: tree −29.6%, summary −25.5% on `linux-v6.12` | User CPU −35% to −65% on rule-heavy trees; wall −2% to −12% on the kernel tree on APFS, about 0% to −6% on metabrowser, 0 with no `.gitignore` | Low. Answers identical by construction; under 1% worst case with two or three rules |
| H175 derived chains | All; detached tree route only | Yes | exp-179: tree −3.3% (stacked on H171) | Invisible in wall; a few percent of consumer CPU | Very low |
| H172, H176, F6e: tree tier | All; the default one-shot tree | Yes, and it changes the plan | exp-180: −13.5% (`linux-v6.12`); exp-181: −10.3% (dense); 1M screen −3.2%, RSS −79% | **Peak RSS −50% to −85%**; wall 0% to −5% on real trees, 0% to −3% on 1M | Low for wall (walker-side frees, about 0.1%); correctness is medium-low only because the real-tree byte diff ran on Linux alone |
| H180 walker trims | All; summary, retained streaming and opened routes | No (summary only) | exp-183: −8.7% (dense); exp-184: −5.6% (`linux-v6.12`) | Summary wall −2% to +2%; walker user CPU down | Very low; H51’s RSS mechanism is absent **[inference]** |
| H169 native reader | Linux glibc only (x86_64 measured; aarch64 shipped untested); not musl, macOS or Windows | No; not compiled | exp-185, exp-186 (summary `--no-controls` −6.3% and −7.9%) | None | None on macOS. aarch64 Linux untested (medium-low) |
| H183 matcher pre-checks | All; every route | Yes | exp-193: tree −7.6% (`linux-v6.12`) | Consumer CPU down; wall 0% to −5% on the kernel tree on APFS | Low |
| H165 walker count | Would have been `PORTABLE` (Linux, Windows); rejected | n/a | exp-182 (12 pairs) | n/a (macOS table separate) | None |
| H181 + H182 | Portable; not merged | n/a | exp-192 | n/a | None |
| H166 wake-one | Portable; never built | n/a | Gate read on Linux | Re-read the gate on macOS (M7) | None |

End to end on Linux (exp-194, exp-195): `linux-v6.12` default tree −39.0%, dense −9.75%.
In both cells system CPU was unchanged (+0.25% and −1.6%) and user CPU fell 69% and 36%.
That split is the reason to expect a much smaller wall effect on macOS, where the user
share of CPU is a quarter or less of Linux’s.

## 5. Counts

| Set | Linux-only | Portable | macOS-specific |
| --- | ---: | ---: | ---: |
| Accepted (6; H176 and F6e counted inside H172) | 1 (H169 phase 1, glibc only) | 5 (H171, H175, H172, H180, H183) | 0 |
| Rejected (4; none shipped) | 0 | 3 as code (H181, H182, H166) and 1 as a `PORTABLE` tuning table (H165: Linux, Windows, not macOS) | 0 |

- On macOS’s default `fdu PATH`: 4 (H171, H175, H172, H183).
- On macOS’s `--view summary`: 3 (H171, H180, H183).
- Measured on macOS: 0 of 10.

## 6. Does anything shipped rest on a Linux-only measurement but change macOS by default?

Yes.
H171, H175, H172 with H176 and F6e, and H183 change what the default `fdu PATH` does
on macOS, and on Windows.
H180 changes `fdu --view summary` and opened discovery.
None has a macOS measurement.

The round changed no tuning value (`platform_tuning.rs` is not in the diff), so no
`Tuned` value is newly inherited.
The rule in `platform-tuning.md` still applies: a constant tuned in one regime is
inherited, not proven, in the others.
It applies to the default plan H172 selects and to the matcher’s cost model.
Before 0.2.2 is tagged with any claim about macOS, confirm on macOS:

1. **Identical answers on APFS.** Diff the product command line’s text, JSON and JSONL
   output between `e5a71c8a` and `0c8131fd`: the 54 base comparisons and the 171
   tier-reaching variants (sort, share, depth, breadth, size, view), on the macOS
   subjects of §7. This is the one correctness check that ran on Linux real trees only.
   APFS differs in hash-ordered listings, 4 KiB-quantized and compressed allocated
   sizes, the `.GITIGNORE` spelling lookup, and firmlink and mount fallbacks.
2. **Default tree and default summary no worse.** On each deciding APFS subject, the
   paired final-versus-Q0 change needs an upper 95% bound of at most +3% (the loop’s
   non-inferiority margin, `performance-loop.md` scheduling section).
3. **Peak RSS no worse, and the tier taken.** The RSS upper bound must be at most +3%. A
   drop of at least half on the tree is the observable proof that macOS took the tier.
4. **Same worker-policy decisions.** Record the adaptive worker-policy history (6
   against 16 workers) on both arms.
   The threshold is marginal on real APFS trees (`fdu-9tul`), and a decision flip would
   confound any wall difference.
5. **Same syscalls.** Compare `FDU_COUNTERS=1` `dir_opens`, `dir_enumeration_calls` and
   `stats` between the arms on one subject.
   If they are equal, the round cannot change cold-cache behaviour through I/O, so no
   cold cell is needed.

Outside macOS, before the release:

- **Linux aarch64:** add `aarch64-unknown-linux-gnu` to `CROSS_TARGETS` and run the
  reader’s tests on an arm64 Linux runner.
- **Windows:** there has never been a speed measurement.
  The release notes should say the Windows default path changed on correctness evidence
  only: `cargo test` and the goldens on `windows-latest`.

## 7. Next macOS benchmark plan

### 7.1 Regime

- **Host:** Apple Silicon, bare metal: the historical M1 Pro, 10 cores (8P + 2E), 32
  GiB, Darwin 25.x. Internal SSD, default case-insensitive APFS.
  - Record `host_performance_cores` and `host_efficiency_cores`.
  - Six walkers, a consumer and the main thread fit on eight P-cores.
    A base 4P + 4E Mac would put some of them on E-cores, which could make the consumer
    critical and the round’s consumer savings visible in wall **[inference]**. That
    regime is an optional replication (M10).
  - Intel Macs ship too (`release.yml:245-250`) and remain unmeasured.
- **Warm-steady only.** `/usr/sbin/purge` only approximates cold on macOS
  (`performance-loop.md:119-121`). Check 5 of §6 is what justifies skipping cold:
  identical syscall counts mean a cold cell cannot separate the arms by I/O.
- **Quiet regime:** the one-second Mach host CPU busy gate at 25% or less.
  No macOS cell since 0.1.0 has held it (evidence report, lines 45-46).
  - Before starting:
    - exclude the subject roots from Spotlight (Settings → Spotlight → Privacy);
    - pause Time Machine and iCloud sync;
    - close other agents and builds;
    - run on AC power, with thermal state normal and memory pressure normal.
  - If the gate refuses three starts, record the cell `uncontrolled`. Say so in the
    release evidence: a non-inferiority result then counts as exploratory.
- **Storage** (per `plan-2026-09-27-macos-performance-rerun.md`):
  - builds, `CARGO_TARGET_DIR` and uv caches on the external SSD;
  - subjects, `TMPDIR`, and the harness’s snapshot and state directory on internal APFS;
  - results outside every subject.
- **No RAM disk** for any of these cells.
  They are real-tree evidence and must run on an ordinary APFS volume
  (`performance-loop.md:370-451`).
  - Run the audit anyway before starting (`hdiutil info`, `git worktree list`, `df`), so
    that a leftover image is found and cleaned up rather than measured beside.
  - If a named synthetic follow-up ever needs a RAM disk, use one image, record its
    purpose, capacity, mount point and cleanup owner in the bead, and detach it without
    `-force` before handoff.
- **Toolchain and binaries:**
  - `rustc 1.97.1`, release profile, `aarch64-apple-darwin`.
  - Release probes (`perf_probe`) and CLIs built from `e5a71c8a` (Q0, the 0.2.1 engine)
    and `0c8131fd` (final; the engine is identical to `a5dbac0f`, because nothing under
    `crates/` changed after it).
  - For the stacked cell (M6), also build `2379233a`, `0228ea42`, `c2a75fe4`, `217861c1`
    and `a5dbac0f`.
  - Copy every binary outside the tree.
- **Instruments:**
  - `make perf-profile` on macOS uses `/usr/bin/sample` (`realtree/profile.py:13-15`,
    `68-75`).
  - `FDU_COUNTERS=1` gives per-layer counts, including the new `ignore_bucket_probes`,
    `ignore_bucket_hits` and `ignore_patterns_tested`.
  - `FDU_SCAN_DIAGNOSTICS=1` or the probe’s `--diagnostics` gives walker `starved_ns`,
    `lock_wait_ns` and `send_ns`, and the policy trace.
  - `xcrun xctrace` Time Profiler and System Trace give thread states, which answer the
    serial-tail and handoff questions.
    CPU Counters gives instructions, in place of callgrind.

### 7.2 Subjects

A subject decides only if it is dense and has at least 50,000 entries.
Three subject classes are needed for a set (transfer) claim.

| Label (new Darwin label) | Class | Provenance and what differs from Linux | Role |
| --- | --- | --- | --- |
| S1 `linux-v6.12-apfs` | source-checkout | `git clone --depth 1 --branch v6.12` onto case-insensitive APFS. **The kernel tree has paths that differ only in case** (the netfilter `xt_*` and `ipt_*` headers and sources); git reports them as collided and one of each pair is lost. Record the collision count and the resulting shape; do not reuse the Linux shape. Also record `diskutil info` “Case-sensitive: No” in the provenance | Deciding. The rule-heavy subject for H171, H175 and H183 (358 `.gitignore` files; verify) |
| S2 `node-modules-dense-darwin` | package-cache | exp-190’s `package.json` and lockfile, then `npm ci --ignore-scripts` (node 22.22.2, npm 10.9.7). The lockfile has 31 OS-restricted packages (`fsevents`, `@lmdb/lmdb-darwin-arm64`, `@parcel/watcher-darwin-arm64`, …), so the darwin shape differs from the 79,957-entry Linux one | Deciding. No `.gitignore`, so it isolates H172 |
| S3 `metabrowser-clone` | source-checkout | The existing Darwin nomination: 137,085 entries, 59 `.gitignore` files, digest `0eed491e…`; re-fingerprint first | Deciding. Continuity with 44 earlier macOS cells |
| S5 `system-private-frameworks` | system-prefix | Existing Darwin nomination: 158,705 entries, 55,256 directories, 6,907 symlinks, no `.gitignore`. Reconstructible by OS build | Deciding, for non-inferiority. Directory-dense, and gives the third class |
| S4 `macos-balanced-1m` | generated | Existing: 1,000,001 entries, 125,001 directories, no symlinks. Reuse it; do not generate another | Screen only. The RSS primary, and the one subject valid for `dust` |

### 7.3 Pre-registered cells

Record these in a plan’s Status table before any timed sample, as `78980e4a` did for
exp-194.

Proposed ids start at exp-202: exp-196 and H184 went to the autofs fix (`fdu-d2fn`) and
exp-197–201 and H185–H190 to the pdu track
([the uniformly-faster brief](research-2026-09-29-uniformly-faster-than-pdu.md)), so the
cells below take exp-202–207 and the borrowed-name bulk listing, proposed as H184 when
this was written, takes the next free hypothesis id (H191 onward) when it is registered.
The runbook’s Current Pickup states the next free ids.

**Arms** in every end-to-end cell, as interleaved variants in this order:

- `q0`: the `e5a71c8a` probe;
- `q0-copy`: a byte-identical copy of `q0` at another path, the placebo arm;
- `final`: the `0c8131fd` probe.

The harness compares each variant with the first and each with its predecessor, in one
schedule (`realtree/measure.py:1991-2010`). If it refuses two variants with the same
hash, run the A/A as a separate 20-pair cell on S1 first.
Each `--no-controls` leg is its own run, with `--no-controls` passed to every variant.

**Pairs and warmups:** 20 pairs after 3 warmups for the deciding subjects, because every
effect is predicted below 10% (the Linux noise rule, from exp-175’s false A/A accept).
12 pairs for screens.

**Decision rules, fixed before sampling:**

- **Release gate (non-inferiority):** `final` against `q0` must have an upper bound of
  +3% or less on `default-tree` and `aggregate-summary`, on each of S1, S2, S3 and S5.
  Peak RSS must also have an upper bound of +3% or less.
  No sample may be invalid.
- **Transfer claim ("faster on macOS"):** a median of −3% or better with the interval
  below zero, on at least one deciding subject.
  A set claim needs deciding subjects in three classes: S1 or S3, S2, and S5.
- **Placebo:** `q0-copy` against `q0` must include zero on every job.
  If it excludes zero by more than 3%, the cell’s verdict is blocked.
- **Results outside the predicted range** are reported as found, as exp-194 did.

| Cell | Subject | Jobs (deciding in **bold**) | Arms | Pairs | Predicted: wall; user CPU; peak RSS (`final` against `q0`) |
| --- | --- | --- | --- | --- | --- |
| **M1** (exp-202) | S1 `linux-v6.12-apfs` | **`default-tree`**, **`aggregate-summary`**, `default-tree --no-controls`, `aggregate-summary --no-controls` | q0, q0-copy, final | 20 | Tree: −2% to −12%; −35% to −65%; −50% to −80%. Summary: −2% to −12%; −30% to −60%; ±10%. Tree `--no-controls` (H172 alone): 0% to −5%; −15% to −35%; −50% to −80%. Summary `--no-controls` (H180 alone): −2% to +2%; −3% to −12%; ±5% |
| **M2** (exp-203) | S2 `node-modules-dense-darwin` | **`default-tree`**, **`aggregate-summary`**, both `--no-controls` | q0, q0-copy, final | 20 | Tree: 0% to −5%; −10% to −30%; −50% to −80%. Summary (no `.gitignore`, H180 only): −2% to +2%; −3% to −12%; ±5% |
| **M3** (exp-204) | S3 `metabrowser-clone` | **`default-tree`**, **`aggregate-summary`**, both `--no-controls` | q0, q0-copy, final | 20 | Tree: 0% to −6%; −20% to −45%; −55% to −80%. Summary: 0% to −5%; −15% to −40%; ±10% |
| **M4** (exp-205) | S5 `system-private-frameworks` | **`default-tree`**, **`aggregate-summary`** | q0, q0-copy, final | 20 | Tree: −1% to +1% (about 20 µs of kernel time per entry sets the walk); −5% to −20%; −30% to −70% (all 55k directories are kept). Summary: −1% to +1% |
| M5 (exp-206) | S4 `macos-balanced-1m` | `default-tree` (primary metric **peak RSS**), `aggregate-summary` | q0, q0-copy, final | 12 | Tree: wall 0% to −3%; user −5% to −25%; **RSS 281–294 → 55–80 MiB (−72% to −82%)**. Summary: 0% to −3%. This is also H66’s registered macOS rule ("decisive RSS reduction without meaningful latency regression"), which H172 now carries |
| M6 (exp-207), attribution screen | S1 | `default-tree`, `aggregate-summary` | Stacked: q0 `e5a71c8a` → `2379233a` (H171 + H175) → `0228ea42` (+H172) → `c2a75fe4` (+H180) → `217861c1` (+H169: **the macOS placebo step**) → `a5dbac0f` (+H183) | 20 | The step predictions follow each row above. The H169 step must include zero. Run it only if M1 shows an effect worth attributing, or a regression, which it would then bisect |
| M7 (screen, no verdict) | S1, S3 | `default-tree --diagnostics` on both arms | q0, final | 12 | The same worker-policy decision distribution on both arms. `starved_ns` gives the H166 gate on macOS. `send_ns` and `lock_wait_ns` are inputs to M9 |
| M8 (peer standings; supplementary to M1–M5) | S1, S2, S3 (20 pairs); S4 (12) | `make perf-compare-tools` with the anchor `fdu-default-tree` at `0c8131fd`; a second run anchored at `e5a71c8a` gives before and after | Peers below | 20 / 12 | Standings, not verdicts. Predicted: fdu’s default ahead of `pdu-default`, `diskus`, `dua` and `gdu` by 20–50%, and within ±10% of `dumac`. Every peer except dumac stats each entry on macOS; on the million-entry tree pdu was +49%, diskus +42% and dumac +9% (2026-09-26 table) |
| M9 (profile, no verdict) | S1, S2 | `make perf-profile` (`sample`), `xctrace` System Trace, `FDU_COUNTERS=1` | final only | — | The macOS half of `fdu-j4p7`: cores busy (CPU ÷ wall) against dumac and `pdu-default` from M8; voluntary switches; the serial tail after the walk ends; handoff waits. It names a hypothesis only if the target is at least 3% of wall |
| M10 (optional) | S1 on a 4P + 4E Mac | as M1 | as M1 | 20 | A larger consumer effect than on the M1 Pro if the consumer lands on E-cores **[inference]** |

**Peers the harness supports on macOS** (`realtree/compare_tools.py:67-278`). Reuse the
hash-identical executables of the 2026-09-26 table:

- `pdu-default` and `pdu` (`--max-depth 2`), pdu 0.24.0;
- `dumac`, pinned `1ffbe3c`. It is the macOS-relevant `getattrlistbulk` peer;
- `diskus` 0.9.0;
- `dua` 2.41.1;
- `gdu` v5.36.1;
- `dust` 1.2.4, **only on S4**: the harness refuses dust on any subject with symlinks
  (`compare_tools.py:832`), and S1, S2 and S5 all have them.

Skip `ncdu`, BSD `du` and GNU `du` in this round: on a million entries they take 50–68 s
each, and the serial floors are not the question.

### 7.4 Order

1. **M0 pre-flight.** No timing.
   1. Run the RAM-disk and worktree audit.
   2. Check storage identity, AC power, thermal state and memory pressure.
   3. Exclude the subjects from Spotlight.
   4. Build all binaries once.
      Run `cargo test -p fdu-core` at `0c8131fd` on the Mac.
   5. Rebuild S1 and S2. Run `make perf-subjects` and `make perf-baseline` for each
      label.
   6. **The answer-identity differential** of §6 item 1. Stop on any difference.
   7. **The counters check** of §6 item 5.
2. **M1**, the rule-heavy real tree and the release gate’s hardest case.
3. **M2**, which isolates H172 with no `.gitignore`.
4. **M3**, for continuity with the macOS record.
5. **M4**, the third subject class, which completes the release gate.
6. **M5**, the RSS screen.
7. **M7**, the policy and diagnostics screen.
8. **M8**, peer standings.
   This is the costliest step and is for publication, so it runs after the gate.
9. **M9**, the profile, which proposes the next hypotheses.
10. **M6**, only if M1 needs attribution or bisecting.
11. **M10**, optional.

**Stop rules:**

- A differential mismatch in M0 stops everything.
- A failed non-inferiority bound in M1–M4 stops the queue.
  Run M6 on that subject to find the step before anything else.
- A blocked placebo stops that cell.
  Re-run it once when the gate holds, and otherwise record it `uncontrolled`.

**Workload arithmetic, not a promise.**

- M1–M3: about 12 minutes each.
  A 3-variant, 4-job, 23-invocation cell makes 276 invocations at about 0.3 s each, plus
  about 2 s of boundary CPU observations.
- M4: about 25 minutes.
  M5: about 15 minutes.
  M6: about 22 minutes.
- M8: about 13 minutes per real tree, and about 35 minutes on S4.
- In all, about 3 to 3.5 hours of cells, plus builds, subject rebuilds and fingerprints.

### 7.5 macOS analogues of the Linux leads

- **Utilization and handoffs (`fdu-j4p7`).** On Linux, fdu spent the least CPU of the
  three tools but kept fewer cores busy: 3.65 against 3.80, with 608 voluntary context
  switches against 128.
  - The macOS inputs: M8 gives cores busy and switches for fdu, dumac and `pdu-default`.
    M7 gives walker `starved_ns`, `send_ns` and `lock_wait_ns`. M9 gives thread states
    at the tail.
  - Earlier profiles (exp-045, exp-046) put about 95% of worker samples in `open` plus
    `getattrlistbulk`. On macOS, handoffs will matter only if walkers starve.
  - `starved_ns` at 2% or more re-opens H166 for macOS.
  - The consumer’s tail after the walk matters only if M9 shows it at 3% or more of
    wall.

- **H169 on macOS.** `getattrlistbulk` already covers every syscall-level piece:
  - the directory is opened with `File::open`, with no `fstat` (`macos_bulk.rs:92`);
  - there is no stat call per entry: each bulk record carries the attributes
    (`macos_bulk.rs:33-44`);
  - a 64 KiB buffer is reused (`macos_bulk.rs:18`, `52-59`).

  It does not cover in-place names:
  - one `OsString` per entry (`macos_bulk.rs:242`) and one `Vec<Entry>` per directory
    (`macos_bulk.rs:102`);
  - then a second copy per entry in `record_detached_entry` (`scan.rs:3730`).

  macOS therefore makes two name allocations per retained entry, where Linux now makes
  one. A borrowed-name bulk listing (proposed as **H184**; that id has since gone to the
  autofs fix, so it takes the next free one) is the macOS form of H169 phase 1’s
  user-space half.
  - H54 (exp-028: reusing the staging `Vec`, +0.2%, RSS worse) and H63 (exp-042: user
    CPU −51%, wall +1.9%) predict a user-CPU cut of a few percent and no wall change.
  - So it is a screen only, gated on M9 naming at least 3%.

- **H179 has no macOS target.** The parent’s bulk record already carries each child
  directory’s `ATTR_DIR_ALLOCSIZE` and `ATTR_DIR_DATALENGTH` (`macos_bulk.rs:42-43`), so
  there is no second per-directory stat to remove.

- **H169 phase 3** (opens relative to the parent) is already refuted on macOS: exp-024
  −0.1% and exp-038 −0.7%.

- **The only macOS idea that removes per-directory kernel work** is still `searchfs`
  (H77, `fdu-9716`), and the macOS floor instrument (`fdu-9hdc`) is what would say how
  much headroom is left.

- **`AT_NO_AUTOMOUNT` on macOS.** The bulk reader already declines to the portable path
  for any directory holding a mount or trigger (`macos_bulk.rs:211-217`). Whether the
  portable `lstat` on an autofs trigger mounts it on macOS is unverified: that is the
  macOS half of `fdu-puk7`, and a correctness question, not a performance one.

## 8. Where to record the results

- **Experiment records.** `docs/project/experiments/exp-202…exp-207-macos-….md`, written
  with `make perf-record`, with artifacts in `evidence/exp-2NN/`.
  - Put the case sensitivity, the collisions and the darwin package set in
    `tree_provenance`.
  - Gzip the M8 standings as supplementary runs, as exp-194 did.
  - Then run `make perf-ledger` and `make perf-report`. The regime-coverage table and
    the by-platform page follow from them.
- **A new plan:**
  `docs/project/specs/active/plan-2026-09-30-macos-confirmation-of-the-linux-round.md`.
  It holds §7’s cells, predictions and order in a Status table, filled before sampling.
  Do not reopen `plan-2026-09-27-macos-performance-rerun.md`, which is marked as run.
- **Evidence report** (`report-2026-08-20-fdu-performance-evidence.md`):
  - `### macOS` (line 42): replace “neither has any change from the 2026-09-29 Linux
    round” (lines 77-78) with the M1–M5 results;
  - `### What Ships Where` (121): give #161’s macOS standing;
  - `## Standing Results by Tier` (147): add macOS rows for the default tree, the
    default summary, and default-tree peak RSS;
  - `## The Loops in Order` (180): add a row for the macOS confirmation;
  - after `## The Linux Overnight Round (2026-09-29)` (287): add a sibling section,
    `## The Round on macOS`;
  - `## Open Work` (561): mark “Confirm on real trees” (584) done or partial;
  - `## What Is Not Measured` (685): Windows, Intel Macs, 4P + 4E Macs.
- **Platform tuning guide** (`platform-tuning.md`):
  - fix the stale Linux row of the Platform table (line 37). It says no native reader is
    profitable, but H169 now is one, profitable in user space;
  - add a subsection saying the one-shot tree plan (H172) and the matcher are portable
    choices measured on Linux, inherited on macOS until exp-202–206 and on Windows
    indefinitely;
  - update “What to measure next per platform” (line 300; macOS row at 309);
  - add `TreeRetention::MAX_FILES` to the constants table as a memory bound, not a
    timing value.
- **Loop registry** (`performance-loop.md`):
  - add a macOS result to the rows for H171 (893), H172 (894), H175 (897), H180 (902)
    and H183 (905);
  - H66 (962): note that H172 carries it, and give its macOS RSS verdict from exp-205;
  - H166 (888): the macOS `starved_ns` reading;
  - H169 (891): “not compiled on macOS; `getattrlistbulk` covers everything except
    in-place names (the borrowed-name listing)”;
  - new rows for the borrowed-name listing, and for a macOS tail or utilization
    hypothesis if M9 names one.
- **Runbook** (`performance-loop-runbook.md`):
  - `## Current Pickup` (988): the macOS confirmation goes first under “Next”, as a gate
    before 0.2.2;
  - `### Darwin Subjects` (523): add S1 and S2;
  - the Ids paragraph (1047): exp-202+ used, and H191+.
- **0.2.2 plan** (`plan-2026-09-29-linux-parity-0.2.2.md`):
  - Stage 3 (212): add a checklist item for the APFS differential and the macOS
    non-inferiority confirmation before release;
  - Rollout Plan (268): make it the release gate;
  - Testing Strategy, “Differential against git” (247): pin
    `core.ignorecase=false core.precomposeunicode=false` on macOS.
- **macOS peer report** (`report-2026-09-26-fdu-live-tool-comparison.md`): a new
  section, “Real trees, default command”, from M8.
- **Nominated subjects:** `nominated-subjects-darwin-arm64.json`, regenerated with
  `make perf-subjects`.
- **README speed claims:** change them only to the scope the new evidence establishes,
  following the `fdu-3ivx` precedent.

## 9. Beads

Filed 2026-09-29 under the macOS epic `fdu-9q80` (itself under the 0.2.2 epic
`fdu-8a8r`), except where noted.
`fdu-dv07` (H162 and H163 on macOS) is related: M6 can gain a leading 0.2.0 variant to
close it too.

| Bead | Title |
| --- | --- |
| `fdu-k3gx` | macOS: end-to-end release gate for the round on APFS (`e5a71c8a` against the final head, with an A/A arm) |
| `fdu-8rid` | macOS: APFS answer-identity differential for the round (171 variants, Q0 against final) |
| `fdu-4tr6` | Nominate `linux-v6.12` and `node-modules-dense` as Darwin subjects on APFS |
| `fdu-446h` | macOS: peer standings for the default command on real trees |
| `fdu-ijkv` | macOS: utilization and handoff profile of the post-round head (the macOS half of `fdu-j4p7`) |
| `fdu-7qz3` | Borrowed-name `getattrlistbulk` listings on macOS (screen; the next free hypothesis id) |
| `fdu-bida` | The macOS half of `fdu-puk7`: does the bulk reader’s `lstat` fallback trigger autofs mounts? |
| `fdu-64cu` | Lint and test the Linux native reader on aarch64 (under `fdu-8a8r`) |
| `fdu-jyed` | Record in `platform-tuning.md` which of the round’s changes are inherited, not measured, on macOS and Windows (under `fdu-8a8r`) |
| `fdu-mqzs` | Document byte-exact `.gitignore` matching under Unicode normalization on macOS (under `fdu-8a8r`) |
| `fdu-bdhe` | Windows: a first speed screen of the default tree (under `fdu-8a8r`) |
| `fdu-8158` | Replicate the macOS end-to-end cell on a 4P + 4E Apple Silicon Mac |

## 10. What is inferred rather than read

- The macOS wall predictions.
  They rest on the macOS CPU split in earlier records and on the conversion rule in
  `performance-loop.md` ("was the saving on the critical path"), not on any measurement
  of this round.
- The cost of H172’s walker-side name frees on libmalloc.
- That std’s `Condvar` on macOS avoids a kernel call when no thread waits (H181).
- That APFS lists entries in hash order (H182).
- That consumers land on E-cores on 4P + 4E Macs.
- That H51’s RSS mechanism cannot recur under H180’s exact-capacity join.
- How the kernel tree’s case collisions resolve on APFS: the exact list must be read
  from the clone.
- That the aarch64 build of `linux_dents` is correct: it is plausible, but never
  executed.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
