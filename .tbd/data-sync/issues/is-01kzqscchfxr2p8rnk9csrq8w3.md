---
type: is
id: is-01kzqscchfxr2p8rnk9csrq8w3
title: Local date-time support in parse_when needs a time-zone decision
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T06:50:25.710Z
updated_at: 2026-08-11T06:50:25.710Z
---
The spec's WHEN grammar includes a bare local form (2026-08-10 [12:30[:45]]) interpreted as local time, matching fd. parse_when currently rejects it with a message pointing at RFC 3339 with an offset or @epoch, because resolving a local civil time needs a time-zone database and the alternatives all cost something the project has rules about: libc localtime_r needs a dependency plus unsafe (workspace denies unsafe_code), a first-party TZif reader is ~150 lines of RFC 8536 parsing plus DST-transition correctness, and assuming UTC would answer a New York prompt hours off in silence, which the freshness rules forbid. Options to decide: (a) accept the rejection permanently and document the grammar as offset-required; (b) take jiff through the 14-day cool-off (the spec's own documented fallback, and BurntSushi's crate with a bundled tzdb); (c) write the TZif reader. Note the goldens pin TZ=UTC, so tests pass under any choice - the difference is only visible to real users at a prompt, which is exactly why this needs a deliberate decision rather than a default.
