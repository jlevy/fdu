---
type: is
id: is-01m0t5szzjt8kr7yqkzg78cxhm
title: "Gitignore rule: the feature-gated ignore dependency and its evaluator"
kind: feature
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prgyv1eq0g0mzgntn1p4n6
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T15:21:45.174Z
updated_at: 2026-08-26T07:01:50.826Z
closed_at: 2026-08-24T19:09:07.240Z
close_reason: |
  Shipped. `make check` green, and the parity harness records all four new golden sessions
  as exact matches rather than as a declared deviation.

  THE RULE. `crates/fdu-core/src/tags/gitignore.rs`: `.gitignore` files read per directory,
  keyed by the directory they govern, decided by walking a path's ancestor chain
  deepest-first. Per-directory is not incidental -- a pattern in `docs/.gitignore` is
  relative to `docs/`, so one composed matcher would read `/build` there as the root's
  `build`. Deepest-first is git's own precedence and is what lets a nested `!keep.log` beat
  a broader `*.log` above it.

  The evaluator takes the entry kind. `target/` matches directories only, so passing `false`
  for one leaves `target` untagged while everything inside it is tagged through the ancestor
  match -- a directory a consumer was told to show, containing nothing it was told to show.
  `TagRules::evaluate` gained `is_dir` and every insert site passes it.

  CONTROL-FILE LIFECYCLE. `TagRules::rebound` re-reads the files without touching the
  enabled set, so the fingerprint is carried across and a `.gitignore` save costs no
  snapshot. `IndexHandle::rebind_tag_rules` swaps and re-tags under one write guard;
  `Session::next_batch` calls it when a batch touched a control file and emits
  `InvalidateSubtree` for the governed directories. Those escalations go out even though
  nothing beneath them was upserted or removed: that is the point, since a consumer holding
  rows for the subtree has no other way to learn its tags moved.

  FEATURE GATE. `gitignore = ["dep:ignore"]`, default-on, passed through from both `fdu` and
  `fdu-py`. Without it the `Matcher::Path` variant does not exist, so the Path-tier arms
  vanish rather than becoming unreachable code, and naming the rule is refused with
  `TagRuleError::Unavailable` saying which feature carries it. Accepting it and answering
  "nothing is ignored" would be a wrong answer rather than a missing one.

  DEPENDENCY, and the correction that mattered. The bead said pin `=0.4.30`. Checking by
  building found 0.4.30 declares no `rust-version` and still fails on 1.85, because it uses
  let-chains -- an "unstable feature" error from inside the crate, not a resolver message. A
  missing `rust-version` is the absence of a claim, not a promise of compatibility. The
  owner then asked whether the 1.85 floor was right; it was not (it is edition 2024's own
  minimum, with no consumer behind it), so **the workspace MSRV moved to 1.88 and the pin is
  gone**: `ignore` 0.4.33, globset 0.4.20, both current, both clear of the cool-off.
  `make audit` confirms `Unlicense OR MIT` resolves through the existing MIT allowance with
  no allowlist change. `make supply-chain` verifies 81 Cargo packages.

  TESTS. Six gitignore-evaluator cases (empty tree, root governance, nested negation,
  anchored patterns, control-file recognition, governed directories), plus rebinding
  preserving the fingerprint, the control-file question being asked of the enabled set
  rather than the catalogue, and the unavailable-rule message. Four golden sessions covering
  the rule, directory tagging, exclusion, and rule composition.

  OUT OF SCOPE v1, as recorded and still true: global `core.excludesFile`, `.git/info/exclude`,
  nested-repository boundary semantics.

  FOLLOW-UP WORTH FILING. `ignore` 0.4.30 added `WalkBuilder::build_matchers()` returning
  `IncrementalIgnore`, documented for exactly this case -- "avoid the work of re-traversing
  an entire directory tree when only a few changes are detected" -- and it covers all three
  of the out-of-scope items above for free, compiling per-directory matchers lazily rather
  than pre-walking for control files as `GitignoreSet::build` does. Its own warning is that
  it does more work per path than a traversal, so a swap is a measurement rather than a
  refactor. Not taken here: this evaluator is tested and working, and the comparison belongs
  in the performance loop.
resolution: null
duplicate_of: null
---
The first Path-tier tag rule, and the only one carrying a dependency. Decided
2026-08-24: fdu-core takes the `ignore` crate behind a `gitignore` cargo feature,
DEFAULT-ON beside `watch` -- notify's exact precedent: "the shipped binary matches
gitignore; --no-default-features and library consumers do not." The tag model itself
(fdu-mvt3) is always-on and dependency-free; only this rule costs.

MEASURED EVIDENCE the decision rests on (2026-08-24): +1.06 MiB on a stripped LTO
release binary against a realistic use (GitignoreBuilder + match), 9 new crates
(ignore, globset, aho-corasick, bstr, regex-automata, regex-syntax,
crossbeam-deque/-epoch/-utils), lockfile 73 -> 82, ~13s cold compile against fdu's ~59s
full release build, and no lean mode (ignore has one feature flag; regex-automata is
mandatory). fd takes the same crates among 16 direct deps; ripgrep owns them as
workspace members. The library/binary asymmetry is what the feature gate answers.

MSRV TRAP, found by checking rather than assuming: ignore 0.4.31+ and globset 0.4.20
declare rust-version = 1.88, above fdu's MSRV 1.85. PIN `ignore = "=0.4.30"`
(published 2026-07-17, clears the 14-day cool-off; rust_version null) and hold globset
at 0.4.19 (2026-07-15, also clear) in the lockfile via `cargo update -p globset
--precise 0.4.19`. The exact-pin precedent is fdu-core's own `pulldown-cmark =
"=0.13.4"`; comment the pin with the MSRV reason so the next upgrade attempt reads it.
A null rust_version is unenforced, so verify the whole subtree with
`cargo +1.85.0 check --all-features`, and run `make cross-lint` -- this code is not
platform-gated but the gate exists for exactly this kind of addition.

WHAT LANDS:
- Cargo: feature `gitignore = ["dep:ignore"]` in default features; the pins above;
  Cargo.lock committed; deny.toml confirmed passing (BurntSushi crates are
  Unlicense OR MIT -- verify the license allowlist covers Unlicense).
- The rule: Path tier, id `gitignore`. An index-held evaluator builds
  ignore::gitignore::Gitignore from the .gitignore files under the root and answers
  matched_path_or_any_parents at apply time -- one computation site, so a watch upsert
  is tagged identically to a scan upsert. Correct negation is the point; the closed
  spike fdu-p35d (0.39-1.76 us/entry) proved the matcher and its cases become tests.
- Control-file lifecycle: an upsert, modify, or remove of a .gitignore rebuilds the
  evaluator and escalates InvalidateSubtree for the directory it governs, so watch
  re-tagging re-walks exactly the affected scope.
- Enabling the rule without the feature is an enable-time error naming the feature.
- tag_rules_fingerprint covers the enabled set including this rule, so a snapshot taken
  without gitignore is not reused for a scan with it.
- OUT OF SCOPE v1, recorded rather than silent: global core.excludesFile,
  .git/info/exclude, nested-repository boundary semantics. A later bead extends the
  evaluator, not the model.

Blocked by fdu-mvt3 (the model this rule plugs into).

## Notes

MSRV FINDING CORRECTED 2026-08-24, and the answer reversed.

The bead recorded: pin `ignore = "=0.4.30"` because 0.4.31+ declare `rust-version = 1.88`
above the 1.85 MSRV. Checking by building found that wrong in a way worth keeping:
**0.4.30 declares no `rust-version` at all and still fails on 1.85**, because
`src/incremental.rs` uses let-chains. The error comes from inside the crate as an
"unstable feature" message rather than from the resolver, so it does not look like an MSRV
problem. A missing `rust-version` is the absence of a claim, not a promise of
compatibility; the only way to learn a crate's real floor is to build against it. 0.4.29
was the newest that actually compiled on 1.85.

The owner then asked whether the floor itself was right. It was not, and it had no
evidence behind it: 1.85 is exactly edition 2024's own minimum, chosen as "the lowest that
compiles our edition" rather than for any consumer. So **the workspace MSRV moved to
1.88** and the pin is gone -- `ignore = "0.4.33"`, globset 0.4.20, both current.

What that changed, all verified: `rust-toolchain.toml` is untouched at 1.97.1 and every CI
job except the MSRV lane already used it, so nothing about the shipped binary or wheel
moved. Clippy is MSRV-aware, so raising the floor made `collapsible_if` and
`is_multiple_of` fire on existing code; those are applied. `supply-chain-policy.json`
pins the 1.88.0 channel manifest by verified sha256.

WORTH KNOWING FOR A LATER BEAD. `ignore` 0.4.30 added `WalkBuilder::build_matchers()`
returning `IncrementalIgnore`, documented for exactly this use case: "avoid the work of
re-traversing an entire directory tree when only a few changes are detected". It also
covers `.git/info/exclude`, global excludes and custom ignore filenames -- the three
things this bead listed as OUT OF SCOPE v1 because implementing them by hand was too
much -- and compiles per-directory matchers lazily rather than pre-walking for control
files as `GitignoreSet::build` does. Its own warning is that it does more work per path
than a traversal, so a swap is a measurement, not a refactor. Filed as a follow-up
rather than taken here, because this bead's evaluator is tested and working and the
comparison deserves the performance loop.
