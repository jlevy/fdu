---
type: is
id: is-01m0tdy8b6h17fqk7mqge56svh
title: Complete the coherent read envelope and version-pinned paging
kind: bug
status: closed
priority: 1
version: 15
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels:
  - pr47-review
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
  - type: blocks
    target: is-01m0tdy9ceep2byvbtyvwc2vky
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:53.445Z
updated_at: 2026-08-25T02:27:18.590Z
closed_at: 2026-08-25T02:27:18.589Z
close_reason: |
  Shipped. `make check` green, parity holds (23 recorded deviations matched).

  All three remaining items from the reopen at a3960fb.

  ITEM 2 was resolved by `fdu-jxs0` rather than here: `set_run_facts` is a clocked commit
  that enters the journal, so one cursor names exactly one envelope. Recorded in this bead's
  notes at the time, with the reason the "narrow fix" could not be literal -- analysis runs
  above the engine and contributes issues after reconciliation, so the envelope necessarily
  commits after the rows. What made that safe is that the interim window is honestly labelled
  `Reconciling` rather than silently current.

  ITEM 3, `as_of`. `build_query` resolved relative bounds against a fresh `SystemTime::now()`
  per call, so an expected cursor pinned the tree and not the cutoff: a version-pinned
  assembly changed membership while its version stood still, and nothing reported it because
  the version genuinely had not moved. `Query.as_of` is the reference instant a caller carries
  across the pages of one answer; absent, it is now, which is right for a one-shot. Test: a
  file half a second inside a one-second window, the wall clock crossing the boundary, and the
  same question asked again -- unpinned it falls out, pinned it stays, including through a
  real `read(expected=...)` page two so the two pins are shown to compose. Mutation-checked.

  ITEM 1, THE ENVELOPE. Three parts.

  Typed issues. `RunFacts.errors: Vec<String>` -> `Vec<Issue>`, where `Issue` carries an
  `IssueKind`, the path, the rendered message, and the OS error number. The kind is what a
  consumer branches on -- retry, prompt for access, drop a subtree -- and reading that out of
  prose makes the decision depend on the wording. I/O failures are classified by the operating
  system's own error kind, because that is the only place the distinction lives: a permission
  failure and a vanished path arrive through the same `Error::Io` variant.

  The binding's own `ErrorDetail` is gone. It had three coarse kinds and existed only in the
  Python surface, so the CLI and a library consumer saw different vocabularies for the same
  condition. One vocabulary, in the engine, shared by every surface. `Provenance.errors` stays
  rendered strings: a report is text, and goldens are unmoved by this.

  Coverage reason. `ReadBundle.coverage: Status` beside `freshness`. `complete` says *that* an
  answer is partial; this says why, which is the half a consumer can act on.

  Lifecycle phase. `Phase` declared whole, following the `CoverageReason` precedent, with
  three members reachable today -- `Ready`, `Reconciling`, `Watching` -- and four documented
  as needing the session (`fdu-4o0m`). Derived rather than stored, since each part is already
  a fact the index holds; a sweep in flight outranks an attached watch, because the fact a
  consumer must act on is that the answers are moving under it.

  `Watching` needed the index to know a watch exists: a counter, attached by `Session::new`
  after the watcher binds (a session that failed to start never claimed to watch) and given
  back in `Drop`. That transition commits, per `fdu-jxs0` -- it changes what a read answers.
  `StateChange::Phase` is emitted only where the phase is a fact of its own: a sweep already
  commits a `Freshness` transition and the phase derives from it, so emitting both would
  report one fact twice and let a consumer see them disagree.

  Test: `check_the_envelope_is_typed_and_its_facts_are_independent` pins that a watch
  attaching moves the phase and *nothing else* -- coverage and freshness are exactly what they
  were -- that the transition reaches the change feed, and that dropping the watch puts the
  phase back, so the state is a fact rather than a latch. Mutation-checked both ways: never
  attaching fails it, and never detaching fails it.

  NOT DONE, and deliberately: progress. Entries-applied-so-far is only meaningful while a walk
  is publishing, which is `fdu-4o0m`. A progress field that could only ever report a finished
  run would lie by implication.

  A PROCESS NOTE. I reverted this file's uncommitted work with `git checkout` while undoing a
  mutation check, and had to redo it. Mutation checks restore from a scratch copy of the file;
  `git checkout` restores from HEAD, which is everything since the last commit.
resolution: null
duplicate_of: null
---
At PR 47 head e658915, the core ReadBundle captures clock, scope, freshness, and projections under one guard, but PyIndex.read releases that guard and then locks RunState to attach complete, source, and errors. A refresh can therefore pair old data with new status or new data with old status. ReadRequest also has no requested clock or version, so a multi-page catalog can silently mix states after a mutation. Fix: return lifecycle, coverage, freshness, source, progress, and typed issues from the same versioned engine image; add an expected session and clock to a read and return VersionUnavailable on mismatch. A provider may retain only the current version: page two either sees the exact version or fails, never advances silently. Add forced interleaving and mutation-between-pages tests. This is follow-up to closed fdu-2ivi and should precede the wider algebra in fdu-samw. Review finding FDU47-R4.

## Notes

ITEM 2 RESOLVED by fdu-jxs0. `set_run_facts` is now a clocked commit that enters the
journal, so a cursor names exactly one envelope: the refresh's rows commit, then the
envelope commits at a distinct clock, and a consumer between them holds a position whose
envelope is the prior one -- coherent, and it learns of the transition from the feed.

Worth recording because it changes what the "narrow fix" can be. Run facts cannot
literally share the rows' commit on the Python path: analysis runs after reconciliation
and contributes errors the engine has no view of. Since freshness is `Reconciling` from
`begin_reconcile` until `finish_reconcile`, the interim window is honestly labelled
in-flight rather than mislabelled current, which is what item 2 was actually about.

STILL OPEN: item 1 (lifecycle phase, progress, coverage reason, and typed issues in the
envelope -- `errors: Vec<String>` is not a vocabulary a consumer can branch on) and item 3
(`build_query` resolves relative `modified_since`/`modified_before` against a fresh
`SystemTime::now()` per call, so a version-pinned multi-page recency assembly can change
membership without the version moving).
