---
type: is
id: is-01m2mdq6zscfr9yj9z7pz1ycv2
title: "PR #65 delta review PR65D-WATCH-2: a batch that inserts and removes an entry claims a classification for it"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
created_at: 2026-09-16T06:15:59.736Z
updated_at: 2026-09-16T07:15:44.944Z
closed_at: 2026-09-16T07:15:44.943Z
close_reason: "955ba6f: BatchFacts.ignored is a BTreeMap<PathBuf, bool> built from Ok(Some(bit)) only, so a path the batch removed answers None rather than false. A unit test pins the three answers."
resolution: null
duplicate_of: null
---
PR #65 delta review 5219107144. crates/fdu-core/src/watch_session.rs:240-245,:270,:279. An entry inserted and removed within one batch gets an upsert with ignored: Some(false): index.is_ignored returns Ok(None) for a path the index no longer holds, the matches!(.., Ok(Some(true))) filter leaves it out of the set, and BatchFacts::is_ignored reports Some(false), a classification claim about an entry that does not exist. Cosmetic; None would be exact.
