---
type: is
id: is-01m2h79m4ap3dsp5kjs5xetjsr
title: "PR #57 review PR57D-MERGE-1: opened.py spells the journal floor as a literal 512 bytes"
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m2h796xe2knpd4z0hz3cqwnr
created_at: 2026-09-15T00:25:59.688Z
updated_at: 2026-09-15T00:25:59.688Z
---
PR #57 review https://github.com/jlevy/fdu/pull/57#pullrequestreview-5204082880, P3. At f05f17e, crates/fdu-py/python/fdu/opened.py:283: the only literal spelling of MIN_JOURNAL_CAPACITY_BYTES; goes stale when #56 raises the floor.
