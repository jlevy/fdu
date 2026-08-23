---
type: is
id: is-01kzzbr1d091cjnjtvax16ssym
title: Classification is the dominant cost of a warm content open, and mostly redundant
kind: task
status: open
priority: 2
version: 6
labels: []
dependencies: []
created_at: 2026-08-14T05:26:02.912Z
updated_at: 2026-08-23T05:06:23.878Z
---
Index::analysis_candidates calls classify::classify_path on every file every time it enumerates candidates - including a pure --cache only run, where the sidecar already stores the classification that then replaces it. A callgrind profile of a warm 14,542-file content open attributes about 34 percent of instructions to std::path::compare_components, and its largest caller edge is classify_path_with_prefix at 1,397,754 comparison calls, roughly 96 per file, which is the signature of a linear scan over the file-type rules table. Two independent fixes, both worth measuring: index the type rules by extension so classification is a hash lookup rather than a scan, which helps cold runs too and can reuse the ext_id the index already interns; or defer classification out of candidate enumeration so a cache hit never pays for a result it discards. Do not repeat this session's mistake of inferring the caller from a flat profile - the BTreeMap keyed by PathBuf in load_content_cache looked like the obvious culprit, was measured at about 0.9 percent of instructions, and swapping it for a HashMap returned only -3.0 percent.

## Notes

Re-scoped by exp-064 and exp-065 (2026-08-23). The ~34% figure that made this P0 came from a flat callgrind profile; the caller tree puts classification at 11.11% inclusive, and H95's indexed tiers have since taken -41.4% absolute off classify_path_with_prefix. What remains is the double classification in apply_analysis's staleness guard -- a public-contract change (AnalysisCandidate is caller-constructible) for a corner of 11%. Dropped P0 -> P2; re-scope or close.
