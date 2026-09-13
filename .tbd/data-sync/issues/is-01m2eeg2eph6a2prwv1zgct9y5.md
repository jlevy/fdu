---
type: is
id: is-01m2eeg2eph6a2prwv1zgct9y5
title: "PR #52 review PERF-4: admission-site checker accepts a route named only in a comment"
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-13T22:34:07.701Z
updated_at: 2026-09-13T22:58:13.477Z
closed_at: 2026-09-13T22:58:13.475Z
close_reason: "Fixed in b1ae4e0 on PR #52 per the Fix line: blockBody returns comment- and string-stripped lines; every impl WalkEmission for a type must be named in EMISSIONS with its chokepoint, failing closed otherwise; red-green node tests for comment-only and string-only routes, a comment-only emission route, and an unaudited emission. The single listing-loop shape, outside the Fix line, remains bounded by exact loop counts."
resolution: null
duplicate_of: null
---
PR #52 review PERF-4 (Low). scripts/check-admission-sites.mjs:135-148, 175, 203-207, 45 at afbb2ee. blockBody walks braces over comment-stripped lines but returns raw source lines, so a route named only in a doc comment satisfies the containment check. LISTING_LOOP recognizes one loop shape, so a loop written differently goes uncounted while loops: 8 still matches. Only the two named emission impls are audited. The checker found no current false negative. Fix: test containment on comment-stripped lines, enumerate every impl WalkEmission for, and add a comment-only test case.
