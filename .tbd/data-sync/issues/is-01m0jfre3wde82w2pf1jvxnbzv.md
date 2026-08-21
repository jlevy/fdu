---
type: is
id: is-01m0jfre3wde82w2pf1jvxnbzv
title: "H95: The type-rules cascade cannot exit early, and runs twice per file"
kind: task
status: closed
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-08-21T15:41:44.443Z
updated_at: 2026-08-21T16:05:24.069Z
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

**Confirmed on the warm path; its cold-transfer prediction refuted** (commit `9fb6a33`, PR #38).

Measured against the **post-H94** base, since H94 shrank the denominator.
40 adjacent interleaved pairs, `content-cache-hit`:

| Metric | Post-H94 | H95 | Change | 95% CI |
| --- | --- | --- | --- | --- |
| wall | 345.6 ms | 326.8 ms | -5.08% | [-6.39%, -3.60%] |
| component | 300.4 ms | 281.5 ms | -5.44% | [-7.27%, -4.11%] |
| peak RSS | 42.5 MB | 42.5 MB | neutral | |

Mechanism: instructions 2,266,646,925 -> 2,106,908,485 (-7.05%);
`classify_path_with_prefix` inclusive 384,389,975 -> 225,056,747 Ir, -41.4% absolute,
16.96% -> 10.68% of profile.

**The cold-path transfer this hypothesis predicted did not hold, and that is the more
reusable half of this result.** The cascade runs on the analysis path too, so `content-basic`
was expected to move similarly. It measured **-4.20% [-6.10%, -0.06%] at 24 pairs** and then
**-2.34% [-5.05%, -0.64%] at 40 pairs** -- direction right, interval below zero, median
below the 3% bar once the estimate settled. Not claimed. Anyone tempted to quote the
24-pair figure should note it moved 1.9 points on the same host with nothing changed but
sample count; that is the drift the loop's 3% floor exists to absorb.

Deliberately not done: `Index::apply_analysis` still re-runs `classify_path` on a path
`analysis_candidates` just classified, so classification is computed twice per warm file.
`AnalysisCandidate` is public and a caller can hand-build an inconsistent one, so that
guard is a real contract check. Making classification cheap was the way to pay for it
without weakening it. If a future change makes candidate construction internal, the
second call becomes removable and is worth roughly the same again.

Residue on this path: `with_flags`, which walks path components for the vendored and
documentation flags on every file -- it was 4.42% of the pre-H94 profile and is untouched.
Scope guard held: the shebang tier has the same non-short-circuiting shape but sits on the
content-prefix path and was left alone.
