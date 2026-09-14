---
type: is
id: is-01m2h2wbh6c8zwwefyzcvyt5z9
title: Source::JournalScoped rustdoc asserts FSEvents history loss the evidence does not establish
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T23:08:50.595Z
updated_at: 2026-09-14T23:08:50.595Z
---
crates/fdu-core/src/engine_contract.rs:263-268 at dda7e6a (origin/main) says macOS FSEvents 'will report HistoryDone after silently dropping history, with no degradation flag'. The FSEvents plan's Phase 0 findings and the performance-frontier research (as revised in PR #55) hedge this: the 2026-08-10 spike had no known pre-mutation fence and did not retain its stream flags, including FullHistory, so silent purge is a possible explanation, not an established fact. Align the rustdoc with the hedged interpretation: say JournalScoped rests on the change history being complete, which neither UUID equality nor HistoryDone proves, and that the committed replay probe (fdu-uwhl) tests the loss mode. Keep the type's weaker-than-Revalidated ordering; this is a doc-comment change only. Found while addressing the PR #55 review (PR55-DOC-3, fdu-1ekf).
