---
type: is
id: is-01m2eefpxxpbmsjrk83ntwncw2
title: "PR #52 review PERF-1: provenance certifies a .dirty binary as built from its clean commit"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-13T22:33:55.900Z
updated_at: 2026-09-13T22:48:24.960Z
closed_at: 2026-09-13T22:48:24.959Z
close_reason: "Fixed in b82a0e5 on PR #52: _fdu_revision_reasons parses the binary's own stamp, requiring -dev+g<revision> at the end of the version with the revision a prefix of HEAD (nine or more characters) and returning a reason for .dirty. Red-green Python tests: a claim-grade capture of a .dirty binary from a clean checkout, and a stamp table. Also closes the gap in the dirty-control rejection fdu-jsbz specified."
resolution: null
duplicate_of: null
---
PR #52 review PERF-1 (Medium). explorations/benchmarks/realtree/provenance.py:326-336 and crates/fdu-core/build.rs:69-78 at afbb2ee. _fdu_revision_reasons accepts a development binary when g<rev9> is a substring of its version, but build.rs stamps a dirty-tree build 0.1.0-dev+g<rev>.dirty, and dirty detection in capture and verify reads only the checkout's current worktree status. A binary built from uncommitted edits, copied out, then captured after reverting those edits is recorded claim_grade true against the clean commit. Gap in the dirty-source rejection fdu-jsbz specified. Fix: return a reason for a .dirty stamp, or require the exact -dev+g<rev> token at the end of the version; add a Python test for the .dirty case. Also affects #54, which uses the same harness.
