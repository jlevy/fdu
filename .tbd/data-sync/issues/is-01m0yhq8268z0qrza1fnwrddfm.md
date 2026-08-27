---
type: is
id: is-01m0yhq8268z0qrza1fnwrddfm
title: Complete the opened-root session goldens and contract coverage gate
kind: task
status: in_progress
priority: 1
version: 9
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sf2nph021wtx28p8ahxh
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T08:06:55.813Z
updated_at: 2026-08-27T02:43:24.506Z
closed_at: 2026-08-27T01:48:23.090Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
Compose the test seams added with each Phase 2 capability into one deterministic runner over the real OpenedIndex owner, workers, exact commit pipeline, journal, and five synchronous operations. Record the five bounded transparent-box session goldens; compare every commit with the independent model; derive automatic public-contract coverage from observed outcomes; add named-only update, lint, size, unstable-literal, duplicate, and orphan checks; prove deterministic barriers, recovery, continuation, and joined shutdown without a runtime trace bus or fact injection. This bead blocks the Python surface so the native contract is complete and reviewable first.

## Notes

CI run 33031258054 first exposed host read_dir ordering; run 33032529592 showed scenario-specific scheduling was insufficient, so e8c0961 centralized the test-only input schedule across every golden session. Run 33033327929 passed Linux, macOS, feature boundaries, and all wheel jobs but isolated a distinct Windows presentation gap: exact nested PathBuf values used Debug-escaped backslashes and canonical roots used an escaped extended-length prefix, neither of which matched the recorder's native path aliases. The final correction records both native and Debug-escaped alias spellings and converts only doubled Windows Debug separators after aliasing; it does not normalize commits, reorder output, or change engine behavior. Two direct unit tests pin both rules, and the five existing golden artifacts remain unchanged locally. Exact-tree validation passes: focused 5-session/159-record golden suite, full make check including all feature combinations, MSRV, installed wheel/sdist smoke, and CLI/Python parity, plus x86_64 Apple/Windows cross-lint. Awaiting replacement GitHub CI before re-closing.
