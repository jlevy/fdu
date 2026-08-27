# Senior engineering review — PR #48, opened-root inventory engine (implementation)

## Scope

Reviewed the full implementation at head `22c43f6`, base `b75bf85` (`origin/main`): +17,600 / −980
across 47 files. This is the implementation follow-through of the design reviewed on 2026-08-26
(all 15 findings of which were accepted and dispositioned in `c4716ec`), so this pass reads the
code against the design authorities it committed to, not against the diff alone.

Ground truth used: the PR-branch worktree; the new durable authorities
(`fdu-engine-architecture.md`, the revised `fdu-design-principles.md`, `fdu-surface-architecture.md`);
the active plan `plan-2026-08-25-fdu-opened-root-inventory-engine.md`; MetaBrowser PR #74 at
`3183888` (`inventory_engine/contract.py`, `providers/python_inventory.py`), cloned locally rather
than read through the web UI; and `git check-ignore` from real git as the gitignore oracle. Every
correctness finding below was reproduced against the checked-out code — the `**` blow-up, the four
matcher divergences, and the `revalidate` regression were each run, not inferred from the diff.

The all-feature `fdu-core` suite passes at this head locally, and GitHub reports all 19 required
checks green (see CI status). The findings below are what green CI does not catch: an untrusted-input
DoS, gitignore semantics that diverge from git on common inputs, a one-line correctness regression on
a public API, and two gate/CI parity gaps.

## Verdict

**Strong architecture, correctly realized; do not leave draft until the gitignore matcher and the
`revalidate` regression are fixed.** The rewrite delivers what the design promised: one fact model,
one exact commit boundary, one cloneable `OpenedIndex` over one shared authority, honest three-valued
knowledge, bounded reads and journal, and a no-gap discovery-to-observation handoff — and it is
backed by an unusually disciplined test suite (an independent reference model, deterministic
scripted-observer sessions, worker-failure and race coverage). The concurrency core is careful and
correct: cancellation holds the wait lock so a wakeup cannot be lost, filesystem I/O stays outside
the index guards, and every mutation funnels through `commit_prepared*`.

The defects cluster in exactly one place the design did not have a reference oracle for: the
hand-written `gitignore` matcher (`crates/fdu-core/src/control/gitignore.rs`), which is new,
enabled by default in the shipped `fdu` and `fdu-py` binaries, has no runtime opt-out, and runs
inside the index write guard during reclassification. One input there is a denial of service; two
more are silent wrong answers on the most common `.gitignore` idioms. Separately, one line in
`revalidate` reorders a `seen.insert` after a fallible `stat`, turning a transient per-entry error
into a false deletion — a clean regression from `main`.

Findings: 1 Blocker, 3 High, 5 Medium, 8 Low. Counts are a snapshot; the Blocker and the three Highs
are the gate.

## Findings

### F1 (Blocker) — A ~130-byte `.gitignore` line hangs the whole engine (`**` exponential match)

`crates/fdu-core/src/control/gitignore.rs:164-175`. The `Segment::DoubleStar` arm of
`segment_path_matches` recurses `visit(pattern_at+1, path_at) || visit(pattern_at, path_at+1)` with
no memoization, and `normalize_glob` (117-136) collapses consecutive `*` only *within* one segment,
never adjacent `**` *segments*. A line of `**/` repeated ~40 times followed by a non-matching literal,
matched against a ~25-deep path, is on the order of `C(64,24) ≈ 10¹⁷` visits.

Reproduced: built a standalone harness from this exact file; the pattern `("**/" × 40) + "x"` against
`("a/" × 24) + "b"` did not finish in 10s (nor 60s in the subagent's run). Reachable by a plain
`fdu <path>` over any tree containing such a `.gitignore` (44 `**/` segments fit in one 130-byte line,
far under the 4 MiB control budget), because `gitignore` is a default feature of the shipped `fdu` and
`fdu-py`, `ScanConfig` has no controls toggle, and `reclassify_controlled_subtrees` runs inside
`Index::apply` under the write guard (`index.rs:2237`) — so the hang wedges the entire opened root,
not just one directory.

**Fix:** collapse adjacent `DoubleStar` segments at parse time (semantically idempotent), and memoize
`(pattern_at, path_at)` in `segment_path_matches` to bound it to `O(segments × components)` — mirroring
the memo `glob_matches` already uses one function below. Consider a named per-line pattern-length cap
as belt-and-braces. Add a git-derived conformance case for it. Bump `IGNORE_RULES_FINGERPRINT` (see F2).

### F2 (High) — The gitignore matcher diverges from git on the most common un-ignore idiom

`crates/fdu-core/src/control/gitignore.rs:104-114` (basename ancestor branch) and `:156` (path branch
grants descendant matches to negations too). In git, exclusion propagates from a matched ancestor
directory, but a *negation* applies only when the pattern matches the queried path itself. The matcher
applies negation to any descendant of a matched ancestor.

Reproduced against real `git check-ignore`:

| `.gitignore` | path | git | fdu (this PR) |
| --- | --- | --- | --- |
| `*.txt` then `!docs/` | `docs/readme.txt` | ignored | **not ignored** |
| `*.tmp` then `/*` then `!/src` | `src/x.tmp` | ignored | **not ignored** |

The first is the everyday "ignore a type, but un-ignore a directory" idiom (`!dir/`). Any repository
using it gets a wrong `unignored` partition — silently, in the headline feature of this PR — and the
public `ControlTable::is_ignored` inherits the error.

**Fix:** make `Pattern::matches` strict per-path for both polarities: a basename pattern matches the
final component; a path pattern consumes the whole path (keeping the trailing-slash directory rule).
Ancestor exclusion is already handled one layer up (`ControlTable::is_ignored`'s prefix loop, and
`index.rs:2874`'s `parent.ignored || matcher(...)`), so the in-matcher ancestor propagation is
redundant for positives and wrong for negatives. Bump `IGNORE_RULES_FINGERPRINT` to 2 (it versions
these semantics), and add a `git check-ignore`-derived corpus — the reference model has no ignore
coverage today, so nothing regression-guards this.

### F3 (High) — Glob memo allocation is sized by `pattern_len × component_len` (untrusted-input allocation)

`crates/fdu-core/src/control/gitignore.rs:244-246`: `vec![None; (pattern.len()+1) * (text.len()+1)]`,
allocated fresh on every `glob_matches` call. Each factor is individually bounded, but the *product*
is attacker-chosen: a single-line `.gitignore` pattern up to the 4 MiB control budget, matched against
a 255-byte name, allocates on the order of a gigabyte — per component, per pattern, per governing
prefix, per entry. A subagent measured 1.58s for a single `matches()` call with a 1 MiB pattern.

This directly violates the design's own First Principle *Never Size an Allocation from Untrusted Input*
(`fdu-design-principles.md`), which the ingestion boundaries elsewhere in this PR honor carefully
(bounded `take(limit+1)` control reads, snapshot control-count validated before its read loop).

**Fix:** replace the memoized recursion with an iterative two-pointer wildcard match (O(1) space, the
standard backtracking-on-`*` algorithm), or enforce a named per-pattern byte cap at parse. A single
reused scratch buffer is the minimum. F1 and F3 are ~150 lines apart in the same file and are best
fixed together.

### F4 (High) — `revalidate` turns a transient per-entry `stat` error into a false removal (regression)

`crates/fdu-core/src/scan.rs:3043-3053` with the not-seen sweep at `:3116-3128`. `seen.insert(name)`
now runs at line 3079 — *after* the fallible `metadata_for_fingerprint` at 3047-3053 — so an entry
whose name `read_dir` returned but whose `stat` fails (EACCES on a searchable-but-unstattable child,
transient EIO) is never inserted into `seen`, `listing_complete` stays `true`, and the sweep emits an
`if_state(Op::Remove, …)` whose Present-precondition still matches, so the removal commits.

Reproduced by diff: on `origin/main` (`scan.rs:2774`) `seen.insert(name.clone())` ran *before* the
metadata call, so this could not happen — this PR reordered it. The result is a definite absence claim
with no evidence of absence, which the architecture explicitly forbids (`absent` only when coverage is
complete). The sibling paths this same PR adds get it right and even document the rule
(`reconcile_target_inner`, scan.rs:3608-3614: "Seeing the name proves it is not absent even if the
following metadata lookup fails"; `reconcile_wave_worker`, scan.rs:4009-4011), which is what makes the
`revalidate` case read as an oversight rather than a decision.

Blast radius: `revalidate` is a public engine API with no in-repo production caller on this branch
(warm open uses `reconcile`), so today it bites external/library consumers only — but it is a silent
wrong-answer bug on a shipped API.

**Fix:** move `seen.insert(name.clone())` to immediately after `let name = item.file_name();`, matching
`main` and the sibling walkers. While there, handle the `Reject`/`ControlOnly` dispositions the way
`reconcile_target_inner` does (explicit `if_state(Op::Remove, baseline)` when the baseline is not
Absent) rather than leaning on the not-seen sweep to remove de-admitted entries.

### F5 (Medium) — Observation handoff fails terminally on one unreadable directory; the baseline tolerates it

`ScanReport::is_complete` = `errors.is_empty()` (`scan.rs:360-362`); consumed by `run_observation`
(`opened.rs:1197-1231`) and `reconcile_conflict_is_retryable` (`opened.rs:1307-1309`). `handoff_complete`
requires a zero-error full-root reconciliation, so one `read_dir` failure anywhere (a single 0700
root-owned dir under `$HOME`, a restricted `.cache` subdir) makes the final pass incomplete, the retry
predicate also requires `scan.is_complete()`, the loop breaks on the first attempt, and the worker
publishes `ObservationTransition::Failed` — while baseline discovery for the identical tree completes
with `phase=Ready, coverage=Partial(Inaccessible)` (`index.rs:1575-1578`). The live feature is therefore
unavailable on a very common class of home directory, and the actual EACCES paths are dropped:
`ObservationHandoffIncomplete` carries no cause.

It fails loudly rather than diverging silently, and the plan's handoff steps don't address persistent
inaccessible scopes — so this is a design decision to make, not a clear bug, hence Medium.

**Fix (decide):** treat walk errors matching the baseline's known-inaccessible boundaries as non-blocking
for the watching transition (enter Watching with `Partial(Inaccessible)`, as the baseline does), or, if
failing is intended, attach the walk errors to the `Failed` issue and document the cliff with a test.

### F6 (Medium) — `refresh()` aborts the whole batch when one accepted path has a symlink-shadowed ancestor

`crates/fdu-core/src/scan.rs:3253-3255` (`resolve_subtree_root(...)?` inside `reconcile_paths_target`,
which returns `Err(SubtreeOutsideScanScope)` on a symlink ancestor at `:4320-4325`); caller
`opened.rs:300-307`. Refresh's contract is per-path accept/reject, and paths are individually classified
first — but ancestry resolution runs *after* classification and `?`-propagates, so one path under an
on-disk symlink (a directory replaced by a symlink since baseline; routine in pnpm-style `node_modules`)
aborts the entire `refresh()` with `SubtreeOutsideScanScope`, doing no work for the other paths.
The per-path symlink refusal is deliberate, tested policy; the wrong part is the batch granularity, and
`RefreshRejection` has no variant for it.

**Fix:** catch `SubtreeOutsideScanScope` per requested root in the resolve loop and convert it to a
rejected path (reuse `OutsideRoot`, or add a `SymlinkAncestor` variant), continuing with the rest.

### F7 (Medium) — Control-file `open` can block a walk/watch/discovery worker forever (lstat→open race)

`crates/fdu-core/src/scan.rs:2288-2301` (`read_control_op`): `kind` comes from an earlier non-following
`stat`, but `File::open` runs later with no re-verification. If `.gitignore` is replaced by a FIFO
between the two, `open(2)` (O_RDONLY, no writer) blocks indefinitely, hanging a cold-walk worker, the
watch verifier, or the opened-root discovery thread. The byte-limit `take` handles infinite *readable*
devices but not this pre-read block.

**Fix:** on unix, `open` with `O_NONBLOCK`, or `fstat` the opened handle and bail unless it is still a
regular file (this also closes the device-node variant cleanly).

### F8 (Medium) — `admission-sites` gate runs in `make check` but nowhere in CI, and cannot see the new producer

Two compounding gaps in one safety net:

1. `scripts/check-admission-sites.mjs` is a prerequisite of `make check` (`Makefile:91,164-165`) but no
   CI job runs it — CI executes discrete steps and never `make check` (confirmed: no `admission` match in
   `.github/workflows/ci.yml`). This contradicts the `check` target's own "everything CI enforces"
   framing and defeats the script's stated purpose (catching platform-gated listing loops "reviewed from
   Linux"): a producer that bypasses `admission::decide` merges green and only trips on the next local
   `make check`.
2. The checker reads only `scan.rs` and `watch.rs` (`check-admission-sites.mjs:11-12`), but this PR adds
   a new directory-listing producer at `opened.rs:964` (`for item in listing {`). It routes correctly
   today (via `prepare_walk_entry`), but the one gate meant to prove that cannot see it. Secondary: the
   brace-counting `blockBody` counts braces inside string literals, so it can mis-scope.

**Fix:** add `node scripts/check-admission-sites.mjs` to the CI `test` job beside the other script checks,
and extend the checker to scan the whole `crates/fdu-core/src` tree (allowlist each file that legitimately
lists a directory, with its expected routed-call set), so `opened.rs` is covered.

### F9 (Medium) — The golden-observability gate misses the natural two-line "redirect then filter" evasion

`scripts/check-golden-observability.mjs:19,27-36`. The audit is line-local, so the most natural
accidental recurrence — `$ fdu … > out.json` on one line, `$ jq '.complete' out.json` on the next —
passes undetected (verified: `auditGoldenText` returns `[]` for it). The `^`-anchored alternative in
`SHELL_FILTER` is also dead for tryscript command lines (they begin with `$ `), so a *standalone* filter
command can never match; only pipe/semicolon/ampersand forms on the fdu line itself are caught. The unit
test named "rejects redirection followed by a product-output filter" only covers the same-line `; jq`
variant, manufacturing false confidence that redirection is handled.

**Fix:** track per-session file state (a redirect target written by an fdu command taints later lines
naming that file), or at minimum flag any golden command whose program is one of the filters when its
argument names a file an fdu command wrote; correct the test to cover the two-line form. A one-line
comment declaring the check heuristic (and adding `execSync`, `cut`, `sort`, `wc` to the blacklists)
would also help.

### F10 (Low) — Malformed `///` line ignores the entire tree

`gitignore.rs:86-96`: a line `///` survives parsing (dir-slash strip → anchor strip → non-empty body
`"/"`) and yields an *empty* `segments` vec, and `segment_path_matches` with an empty pattern returns
`true` for every non-empty path. Reproduced: fdu matches everything; `git check-ignore` matches nothing
(exit 1). **Fix:** return `None` from `Pattern::parse` when the segment list is empty.

### F11 (Low) — Trailing `**` drops the directory-only constraint

`gitignore.rs:159-163`: the `DoubleStar`-last fast path returns `path_at < path.len()` without consulting
`directory_only`/`target_is_dir`, so `a/**/` matches the *file* `a/f`. Reproduced: fdu `Some(true)`, git
not ignored. **Fix:** apply the same trailing-slash directory check the general terminal case uses.

### F12 (Low) — `[]]` character class

`gitignore.rs:249-253`: the first `]` is taken as the class terminator, so `[]]` fails to match `]`
(git's wildmatch treats a leading `]` as a literal member). Obscure; fix or document in the module doc.

### F13 (Low) — `identity()` documents FNV-1a but multiplies by a wrong, unnamed prime

`control.rs:310-317` multiplies by `0x1000_0000_01b3` (= 2⁴⁴+0x1b3); the FNV-1a 64 prime is
`0x100000001b3` (2⁴⁰+0x1b3), which `admission.rs:15-17` spells correctly as named constants two files
away. Deterministic and self-consistent (control identities are commit-stream metadata, never persisted),
so not a runtime bug — but the `ControlIdentity` doc is false and these are magic numbers. **Fix:** correct
the literal and name it, or share admission's `FNV_*` constants.

### F14 (Low) — `MAX_RETAINED_ISSUES` / `all_dirty` clamp behavior is untested

The reference model (`tests/reference_model.rs:216-222,594-611`) models the issue-clamp and all-dirty
branches, but the generated workload never approaches 64 issues or 256 dirty paths, so those bounds are
checked one-sidedly (a *lowered* clamp would be caught, a *raised* one would not), and `retain_issue`
(`index.rs:1467`) has no direct test. Low, but it is the one materially untested branch in an otherwise
exemplary suite. **Fix:** one named test driving >64 gap invalidations through both model and index.

### F15 (Low) — Duplicated magic `64`, dead `walked` field, and a closed-set drift risk

Three unrelated tidies: (a) `CONTROL_SOURCE_OVERHEAD = 64` is named in `control.rs:44` but hardcoded as
a literal at `scan.rs:2304` and `snapshot.rs:611`, so the error accounting and snapshot guard silently
drift if it changes — export and reference the constant. (b) `ReconcilePathsReport::walked`
(`scan.rs:3192-3193,3257-3258`) is `#[cfg(test)]`-populated but read by no test (the `Debug` derive hides
the dead-code lint) — remove it or add its test. (c) the closed family set is hand-synced across
`MANIFEST_FAMILIES`, `family_from_name`, and `build.rs::family_variant`; a family that validation admits
but `family_from_name` lacks would `expect("validated family")`-panic (`classify.rs:261`) — consistent
today, so latent, but add `MANIFEST_FAMILIES.iter().all(|f| family_from_name(f).is_some())` as a
bijection guard.

## Suggestions (non-blocking)

- **The reference model shares two production helpers.** It calls `Commit::retained_cost()` inside its
  own journal eviction and `Commit::applied_delta()` to build both outcomes and `since` deltas, so the
  `deltas` equality assertion is tautological and a `retained_cost` bug would shift both journals
  identically. Both functions have their own direct unit tests, so coverage is not lost — but either
  re-derive them independently in the model or drop the header's "deliberately does not call index
  helpers" claim, which is currently inaccurate.
- **A git-derived gitignore conformance corpus** (a table of `git check-ignore` cases: negation of
  ancestor, the whitelist idiom `*` / `!/src`, trailing `**`, `[]]`, all-slash) would have caught
  F2/F10/F11/F12 and should land with the F1/F2/F3 fixes. Note also that matching is byte-exact while
  git honors `core.ignorecase` (default-on for macOS/Windows clones); a *fixed* case-sensitive semantic
  is defensible but should be stated in the matcher's module doc.
- **Fingerprint migration is a one-time cache invalidation worth a changelog line.** `TYPE_RULE_FINGERPRINT`
  moved from a digest of normalized source bytes to `manifest_fingerprint` over parsed values, so it
  changes for the unchanged manifest and invalidates every existing snapshot and content sidecar exactly
  once on upgrade. Deliberate and contract-conformant; just note it.

## False positives — checked, do not "fix"

- **`ReadProjection::Aggregate` counting over `portable_entries` is intentional, not a partial-index scan.**
  It looks like it might miss unrepresentable paths, but it is bounded by `count_cap`/`max_work` and
  returns `Exact`/`AtLeast` explicitly (`opened/read.rs:572-625`); native roll-ups still count every
  entry. Correct per the plan's aggregate contract.
- **The `apply_and_notify` test helper bypassing the producer path is sanctioned.** `opened.rs` tests use
  `index.apply` + manual `journal.notify_commit()`; this is the architecture doc's controlled-session
  doctrine (drive real state, observe production values), and producer-path invariants are separately held
  by the reference model and the scripted-observer sessions.
- **`fdu-core` `default = []` with `watch`+`gitignore` enabled only in the `fdu`/`fdu-py` packages is the
  design, not a regression.** It is exactly the "engine pays for nothing; surfaces opt in" rule from the
  surface architecture, and `make check`'s `lib-only` matrix exercises all four combinations.
- **`OpenedIndex::read` taking the lifecycle mutex is not a read-serialization bug.** It checks phase under
  the short lifecycle lock and then does the actual projection under the index `RwLock` read guard
  (`opened/read.rs:22`), so concurrent reads still proceed; the lifecycle lock is not held across the walk.

## Design assessment

**What is right, and worth defending.** The rewrite is a faithful realization of the design's central
thesis — one owner per concept, one exact commit path — and the code makes that structural rather than
aspirational. `OpenedIndex` is a thin `Arc<OpenedState>` with no parallel service API; every producer
(cold discovery, refresh, observation verification, control changes) funnels through `commit_prepared_with`,
and impact is derived from effective changes rather than copied from requests, so the "no second account of
what changed" rule holds by construction. The concurrency primitives are the strongest part of the
implementation: `Cancellation::cancel` takes the wait lock before storing the flag and notifying, closing
the classic lost-wakeup window; the `BaselineLatch` and `JournalWait` follow the same discipline; workers
hold only cancellation, never a strong owner reference, so close can always join them; and filesystem I/O
is provably outside the index guards, with a dedicated test that blocks the verifier and proves readers
and a competing writer still progress. The three-valued knowledge model, the bounded continuation table
with version pinning and eviction, and the separation of provider loss (`WatchOverflow` → reconcile) from
consumer lag (journal floor → reset) all match the architecture doc precisely and are pinned by named
tests. This is the ownership diagnosis from the design review, delivered.

**Where it is weak.** The design had a reference oracle for the index (an independent model) and for the
walkers (parity goldens), but none for the one genuinely new algorithm: the gitignore matcher. That is
exactly where the defects landed — a DoS and two silent divergences from git, none of which any existing
test would catch because there is no `git check-ignore`-derived corpus. The matcher is also the one place
this PR's otherwise-excellent untrusted-input hygiene lapses (F1, F3), against a rule the design states as
a First Principle. The lesson mirrors the prior design review's own conclusion — the plan was strong where
it had an oracle and loose where it did not — and the fix is the same shape: give the new component an
independent oracle (real git) before trusting its output.

**Cross-provider agreement is still genuinely deferred, and that is honest.** MetaBrowser PR #74 at
`3183888` still spells `OPENING_CACHE` and still sorts its catalog by `record.path`; the plan's Phase 3
(the contract reconciliation that renames the enum and pins byte order) is marked not-started, and the code
matches that status rather than pretending the agreement already exists. Worth noting for a future reviewer:
fdu's flat-page order is "canonical POSIX-relative UTF-8 bytes", which for valid Unicode coincides with
Python's default code-point string sort — so the R2 order question the design review raised is reconcilable
without either side resorting, but the enum-vocabulary alignment (R7) is real Phase 3 work that this PR
correctly does not attempt yet.

**One thing this PR does not claim, and should not be read as claiming.** There is still no measured
evidence that the fdu provider beats the Python provider on the MetaBrowser corpus — Phase 3A (the
unchanged-contract cost spike) and Phase 4 measure that, and both are open. The sequencing is deliberate
and documented, but the project remains committed to the vertical slice before the premise is measured.

## Documentation

- The prior design review's documentation findings are all resolved: `TODO.md` now carries the `fdu-snej`
  epic and the opened-root spec row and narrows `fdu-wpa0`; `AGENTS.md` points at the new engine
  architecture doc; the three durable authorities (`fdu-design-principles`, `fdu-engine-architecture`,
  `fdu-surface-architecture`) are internally consistent and correctly cross-linked.
- New doc gaps to close alongside the fixes: state the byte-exact / case-sensitive gitignore semantics in
  the `gitignore.rs` module doc (see Suggestions); add a changelog line for the one-time fingerprint
  invalidation; and, if F5 is resolved by keeping the terminal failure, document the inaccessible-scope
  cliff. Relative links in the new docs resolve and Documentation CI is green.

## CI status

All 19 required checks green at `22c43f6` (run 33012662005), including Documentation, Format check, Clippy,
MSRV 1.85, dependency audit, supply-chain provenance, Python CLI parity, and the wheel matrix on
ubuntu/macos/windows for Python 3.12 and 3.14. `mergeable: MERGEABLE`. The all-feature `fdu-core` suite
also passes locally at this head. Draft, as intended. None of the findings above are visible to CI — F1 is
an input-dependent hang, F2/F4/F10-F12 are correctness divergences with no conformance corpus, and F8/F9
are gaps in the gates themselves — which is the argument for the git-derived corpus and the CI wiring in
F8/F9 before this leaves draft.

---
_Generated by [Claude Code](https://claude.ai/code)_
