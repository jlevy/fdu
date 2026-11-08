---
type: is
id: is-01m6ydwr5tvt5bcgc0738y0tym
title: Remove the pulldown-cmark 0.13.6 cool-off exception after 2026-11-13
kind: bug
status: open
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-11-08T20:34:12.794Z
updated_at: 2026-11-08T20:39:46.524Z
---
cargo deny check advisories fails on main and every PR: pulldown-cmark 0.13.4 has RUSTSEC-2026-0414 (fixed 0.13.5, published 2026-10-21) and RUSTSEC-2026-0433 (fixed 0.13.6, published 2026-10-30, inside the 14-day cool-off until 2026-11-13T08:50Z). fdu is not exposed: both need Options::ENABLE_WIKILINKS or Options::all(), and crates/fdu-core/src/content/content_markdown_metrics.rs:139 enables neither. Options for the maintainer (agents do not approve exceptions): (1, recommended) documented deny.toml ignores for both IDs, removed when pulldown-cmark >= 0.13.6 is taken on/after 2026-11-13; (2) a release-age exception for 0.13.6 in supply-chain-policy.json; (3) wait until 2026-11-13. Either way, on/after 2026-11-13 run cargo update -p pulldown-cmark --precise 0.13.6 with --locked builds, rerun the documents/words goldens and make check, and remove any ignore. Raised on https://github.com/jlevy/fdu/pull/208#issuecomment-6726766860.

## Notes

2026-11-08: the maintainer chose option 2 (release-age exception) on the condition of an agent supply-chain review. Review done (checksum, publisher, byte-identical to upstream tag v0.13.6, diff 0.13.4->0.13.6 limited to parser hardening, no new deps/unsafe/I-O). Commit 3c2c0db2 on PR #208 takes 0.13.6 and records exception pulldown-cmark-0.13.6-rustsec-2026-0433 in supply-chain-policy.json (expires 2026-11-14T00:00:00Z). cargo deny check passes. Remaining: after 0.13.6 clears the cool-off at 2026-11-13T08:50Z, remove the exception entry and confirm 0.13.6 was not yanked; then close this bead.
