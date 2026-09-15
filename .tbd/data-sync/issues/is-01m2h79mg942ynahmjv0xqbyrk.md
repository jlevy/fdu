---
type: is
id: is-01m2h79mg942ynahmjv0xqbyrk
title: "PR #57 review PR57D-DOC-1: fdu.open and Index::new docs under- or over-state their rules"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2h796xe2knpd4z0hz3cqwnr
created_at: 2026-09-15T00:26:00.072Z
updated_at: 2026-09-15T00:36:09.259Z
closed_at: 2026-09-15T00:36:09.258Z
close_reason: "c141285: fdu.open says a default open never starts from a report's snapshot and that a report answers from a default open's snapshot only under CachePolicy.ONLY (and the Rust open doc likewise); Index::new names the partition accessors among those that refuse and says a featureless control table refuses control input."
resolution: null
duplicate_of: null
---
PR #57 review https://github.com/jlevy/fdu/pull/57#pullrequestreview-5204082880, P3. At f05f17e, crates/fdu-py/python/fdu/_api.py:355 says an open never starts from a report's snapshot where a default open is meant; crates/fdu-core/src/index.rs:1562-1565 (Index::new) names only is_ignored and controls among the refusing accessors, omitting the partition accessors.
