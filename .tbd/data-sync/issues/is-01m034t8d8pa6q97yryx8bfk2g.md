---
type: is
id: is-01m034t8d8pa6q97yryx8bfk2g
title: Resolve the inconclusive quiet-host release-CLI cell before any positive dust claim
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - benchmark
  - validation
  - macos
dependencies: []
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-15T16:41:53.320Z
updated_at: 2026-08-15T16:41:53.320Z
---
fdu-j062 ran the Apple Silicon/APFS release-CLI noninferiority matrix against the pinned dust adapter and closed with an explicitly inconclusive result: the predeclared quiet native cell had two host-pressure invalidations, so its fixed-N matrix did not resolve. Uncontrolled diagnostic cells were exact and favored fdu (native 43.10%, wheel-installed 41.70%), but fdu-j062's own contract states that any supported cell which is inferior or inconclusive blocks a positive release-performance conclusion until resolved or explicitly removed from supported scope with product justification.

No open bead currently carries that obligation. fdu-j062 and every bead it blocks are closed, so the requirement is recorded only in a close reason and in the gap-closure report. This bead restores the tracking so a future release claim cannot silently inherit an unresolved cell.

This does not block the shipped no-change outcome of fdu-5rpt, and it does not block release itself. It blocks only the promotion of a comparative performance claim about fdu versus dust into the README, release notes, or any other user-facing surface.

Acceptance: either re-run the predeclared quiet native cell on a host that stays under the declared pressure ceiling for every timed process and report the +3% noninferiority decision with its interval convention, sample count, and stopping rule fixed before measurement; or explicitly remove that cell from supported scope with written product justification. Do not substitute the uncontrolled diagnostic cells for the quiet-host gate, and do not add repetitions or select fixtures after seeing a threshold result. Until one of those outcomes is recorded, no positive fdu-versus-dust performance claim may be published.
