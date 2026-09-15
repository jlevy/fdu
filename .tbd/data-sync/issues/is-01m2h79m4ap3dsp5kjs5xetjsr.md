---
type: is
id: is-01m2h79m4ap3dsp5kjs5xetjsr
title: "PR #57 review PR57D-MERGE-1: opened.py spells the journal floor as a literal 512 bytes"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2h796xe2knpd4z0hz3cqwnr
created_at: 2026-09-15T00:25:59.688Z
updated_at: 2026-09-15T00:36:08.921Z
closed_at: 2026-09-15T00:36:08.920Z
close_reason: "dc50116: the journal_capacity_bytes comment names the engine's minimum, MIN_JOURNAL_CAPACITY_BYTES, instead of 512 bytes, and notes the refusal states the minimum."
resolution: null
duplicate_of: null
---
PR #57 review https://github.com/jlevy/fdu/pull/57#pullrequestreview-5204082880, P3. At f05f17e, crates/fdu-py/python/fdu/opened.py:283: the only literal spelling of MIN_JOURNAL_CAPACITY_BYTES; goes stale when #56 raises the floor.
