---
type: is
id: is-01kzzbr1d091cjnjtvax16ssym
title: Classification is the dominant cost of a warm content open, and mostly redundant
kind: task
status: open
priority: 0
version: 4
labels: []
dependencies: []
created_at: 2026-08-14T05:26:02.912Z
updated_at: 2026-08-21T16:05:24.505Z
---
Index::analysis_candidates calls classify::classify_path on every file every time it enumerates candidates - including a pure --cache only run, where the sidecar already stores the classification that then replaces it. A callgrind profile of a warm 14,542-file content open attributes about 34 percent of instructions to std::path::compare_components, and its largest caller edge is classify_path_with_prefix at 1,397,754 comparison calls, roughly 96 per file, which is the signature of a linear scan over the file-type rules table. Two independent fixes, both worth measuring: index the type rules by extension so classification is a hash lookup rather than a scan, which helps cold runs too and can reuse the ext_id the index already interns; or defer classification out of candidate enumeration so a cache hit never pays for a result it discards. Do not repeat this session's mistake of inferring the caller from a flat profile - the BTreeMap keyed by PathBuf in load_content_cache looked like the obvious culprit, was measured at about 0.9 percent of instructions, and swapping it for a HashMap returned only -3.0 percent.

## Notes

2026-08-21 (Linux session): **the diagnosis in this bead is wrong, and it is wrong in exactly the way the bead warns about.**

Re-profiled `content-cache-hit` under callgrind on a 15,977-file generated tree
(gen_tree.py seed 42, 17,000 entries; 1,045 dirs), release + debuginfo, 3.462B Ir,
oracle digest identical to the seeding run. Read flat first, then `--tree=caller`.

The flat view reproduces the reported signature: `std::path::compare_components` sums to
about 36% of instructions across its inlined attributions, and `Components::next` a
further 21%. That is where this bead's 34% came from.

The caller tree says the dominant caller is **not** classification:

| Caller edge into `compare_components` | Ir | calls |
| --- | --- | --- |
| `ContentIndex::merge_ancestors` | 1,256,661,348 (36.30%) | 2,028,997 |
| `Index::apply_analysis` -> `BTreeMap<PathBuf, FileAnalysis>::remove` | 383,869,618 (11.09%) | 529,036 |
| `classify::classify_path_with_prefix` | 51,198,615 (1.48%) | 2,266,470 |
| `classify::with_flags` | 53,016,939 (1.53%) | 510,591 |

Inclusive costs on the same profile:

- `load_content_cache` 78.10%
- `Index::apply_analysis` 71.44%
- **`ContentIndex::merge_ancestors` 43.73%**
- `Index::analysis_candidates` 11.95%, of which `classify_path_with_prefix` 11.11%
  (`with_flags` 4.42% inside it)
- `perf_probe::summarize_index` 9.42% -- harness oracle, excluded from any engine claim
- `snapshot::parse_stream` 3.05%

So classification on this path is about **11%**, not 34%, and the redundant-work framing
is still correct but three times smaller than recorded. The 34% belongs to
`ContentIndex::merge_ancestors`, which walks every ancestor of every file and does an
independent `BTreeMap<PathBuf, _>` lookup at each level -- O(depth x log n) full
path-component comparisons per file -- plus one unconditional `path.to_path_buf()`
allocation per ancestor per file even when the key already exists.

The `~96 comparisons per file` figure was right about the count and wrong about the
owner: it is roughly 8 ancestors x log2(1,045 dirs), not a scan of the 65-rule table.
The rules-table scan is real (`.filter().max_by_key()` never short-circuits, so every
file pays the full 65 rules and 167 extension strings) but it is inside the 11%.

Split into two beads with the corrected sizes:
- `fdu-cq7t` (H94), the 43.73% ancestor-roll-up lookup. Screened first.
- this bead keeps the classification work at its true 11.95% ceiling; fix 2 (keep
  classification out of candidate enumeration) is still the better of its two fixes,
  since a cache hit discards the result entirely. Fix 1 (index the rules by extension)
  addresses only part of the 11%.

Note the two are independent costs in different functions, so unlike H13/H18 they do not
compete -- but re-screen this one after H94 lands anyway, because H94 shrinks the
denominator every percentage here is quoted against.

**Update, same session:** fix 1 landed as H95 (`fdu-9dcj`, commit `9fb6a33`) -- the
exact-name and extension tiers now resolve through two `LazyLock` hash tables instead of
a non-short-circuiting scan. Warm `content-cache-hit` -5.08% [-6.39%, -3.60%] against the
post-H94 base.

What remains open in this bead:

- Fix 2 (keep classification out of candidate enumeration) is **not** done and is now
  worth less than it was: with the cascade indexed, the discarded work is cheaper. It is
  also blocked on a contract question, since `Index::apply_analysis` re-runs
  `classify_path` as a staleness guard on the public `AnalysisCandidate`.
- `with_flags` is the untouched residue: it walks path components for the vendored and
  documentation flags on every file, 4.42% of the pre-H94 profile.
