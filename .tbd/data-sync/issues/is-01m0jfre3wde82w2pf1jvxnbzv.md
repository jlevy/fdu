---
type: is
id: is-01m0jfre3wde82w2pf1jvxnbzv
title: "H95: The type-rules cascade cannot exit early, and runs twice per file"
kind: task
status: closed
priority: 1
version: 3
labels: []
dependencies: []
created_at: 2026-08-21T15:41:44.443Z
updated_at: 2026-08-23T05:06:34.315Z
---
**Tier: content** (index plus content records), with expected transfer to cold scans too,
since the same cascade runs there.

After H94 (`fdu-cq7t`) landed, classification is the largest remaining engine cost on a
warm content open: 18.24% of the post-H94 profile in `Index::analysis_candidates`, plus a
further 3.68%-of-pre-H94 in `Index::apply_analysis`. Absolutely unchanged by H94 at about
413M Ir.

Two separate facts make it bigger than it looks:

1. **It runs twice per file.** `analysis_candidates` classifies every file to build the
   candidate, and `Index::apply_analysis` then re-runs
   `classify_path(&candidate.relative_path)` and compares it against
   `candidate.classification` as a staleness guard. `classify_path` is a pure function of
   the path, so within one `load_content_cache` the second call cannot disagree -- but the
   guard is validating a *public* struct a caller could have hand-built, so it is a real
   contract check, not dead code. Do not delete it.
2. **Neither pass can exit early.** Both scans are
   `GENERATED_RULES.iter().filter(..).max_by_key(..)`, and `max_by_key` consumes the whole
   iterator. Every file therefore pays all 65 rules and all 167 extension strings even
   when its extension matches the first rule in the table.

This is fix 1 from `fdu-926e`, and it is preferable to fix 2 (defer classification out of
candidate enumeration) precisely because it needs no contract change: it makes both call
sites cheap instead of removing one of them, and it helps cold runs, which fix 2 does not.

Resolve the exact-name and extension tiers through two `LazyLock` hash tables built once
from `GENERATED_RULES`. The tie-break has to be reproduced exactly:
`Iterator::max_by_key` returns the **last** of equally-maximum elements, so the table
builder must let a later rule win at equal priority.

Non-UTF-8 names stay equivalent for free: the rules table is pure ASCII, so a name that
fails `to_str()` matched no rule filename under the old byte comparison either.

**Predicted signal:** `content-cache-hit` component and wall down at least 3% with both
95% intervals below zero, against the post-H94 base; peak RSS no worse; classification
byte-identical on a differential check over every real path in the corpus and over every
key in the rules table.

**Scope guard:** index the exact-name and extension tiers only. The shebang scan has the
same non-short-circuiting shape but sits on the content-prefix path, which is not on this
hot path, so indexing it would add surface with no measured benefit.

## Notes

H95 confirmed on the warm path in exp-064; exp-065 adds that its denied cold-path transfer claim was doubly right to deny. The cold content-basic figure is subject-shaped: -13.56% on the generated subject (depth 16, 22.6x sparse) against -2.38% on dense real source, same binaries, same day.
