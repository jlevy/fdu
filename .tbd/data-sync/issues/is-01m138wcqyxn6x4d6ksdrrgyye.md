---
type: is
id: is-01m138wcqyxn6x4d6ksdrrgyye
title: Decide whether the portable encoding should be total
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0y1shykye8sc7h7e9rkk6kh
created_at: 2026-08-28T04:08:39.420Z
updated_at: 2026-08-29T05:52:52.850Z
closed_at: 2026-08-29T05:52:52.849Z
close_reason: |-
  Decided yes, and implemented in 13fe8b4 'feat: every path has a portable name'. The encoding is now total: every path has a portable name, produced by escaping undecodable bytes as %XX and escaping % itself as %25 so the mapping stays injective. Valid UTF-8 runs are preserved, so a mostly-readable name stays mostly readable.

  Making it total collapsed the portable/native population split exactly as this bead anticipated, and deleted the machinery that split required: PortablePathIssue, PortablePathExample, PortablePathEncoding, MAX_PORTABLE_PATH_EXAMPLES, portable_omitted, portable_examples, TreePage's second completeness flag, and the 'unknown instead of absent' branch in absence_is_known. EntryValue.portable_path is a PortablePath rather than an Option.

  Precedent for totality: git's quoted paths, Python's surrogate escapes, and the file:// URIs LSP and desktop file managers exchange all make the derived name total. None tells a caller that a file has no name.

  Closes the follow-on fdu-mokz, which described an omission-example list that no longer exists.
resolution: null
duplicate_of: null
---
portable_path returns Option<String>: an entry whose native path is not valid UTF-8 has no portable form and is absent from every ordered projection. That partiality is the sole source of the two-population problem, and everything built to manage it exists only because of it: portable_omitted, portable_examples, the separate native_complete and portable_complete flags, unknown-instead-of-absent lookups below a non-portable directory, and a conformance case whose job is to stop the difference reading as a defect.

Every mature system that faces this problem uses a TOTAL derived encoding. Git stores path bytes and renders a reversible C-escape for display (core.quotePath); Python's PEP 383 surrogateescape round-trips undecodable bytes losslessly; Rust's OsStr on Windows is WTF-8. None of them ever says a file has no name. The systems that share fdu's exact shape - filesystem contents crossing into a JSON/HTTP boundary, which is MetaBrowser - converged on percent-encoded URIs: LSP, VS Code, and the freedesktop/GIO file managers.

Jujutsu's simplification is NOT available here and the reason is worth recording. jj requires UTF-8 in RepoPath because a version control system is a gatekeeper: you choose what enters the repository, so refusing a bad name is a legitimate answer. fdu is an observer pointed at an arbitrary directory and cannot refuse reality. Git's answer does not fit either: git never faces a Unicode-only sink, and fdu's primary consumer is one.

Proposal: make portable_path return PortablePath, percent-escaping bytes that are not valid UTF-8 plus the '%' character itself. This is not URI encoding - it is a JSON string, not a URL, so spaces, '#', '?' and all Unicode pass through untouched. Valid UTF-8 paths stay byte-identical except for a literal '%'.

Escaping '%' everywhere is not optional, and the collision proves it. File A named literally 'caf%FF.txt' is valid UTF-8. File B named 'caf<0xFF>.txt' has one invalid byte. Escaping only invalid bytes maps both to 'caf%FF.txt': two files, one wire name, which is the aliasing bug of lossy conversion wearing better output. A side flag {path, escaped} does not rescue it either - the pair is unique but ordering is defined on the string, and the strings still collide.

The cost is therefore exactly one thing: a file named '100%.txt' transmits as '100%25.txt'. Invisible in practice because the adapter decodes before display, visible if someone reads raw JSON or forgets to decode. Git has the same property.

Performance is not an argument against it. The UTF-8 validation already runs on every component today, and the portable string is already allocated and stored. The fast path is unchanged; only the failure branch differs.

What it deletes: portable_omitted, portable_examples and its three cfg-gated hex encoders, one of the two completeness flags, unknown-instead-of-absent below a bad directory, the two-population conformance case, and the rule that one stray byte in a directory name hides its whole subtree.

What it requires: prove injectivity with a fixture holding a literal '%' beside an escaped byte; confine decoding to the adapter; and revisit PortablePath::to_native_relative_path and both its callers, which are sound only while the derivation is the identity. entries_rebuild_to_their_native_paths carries '100%.txt' precisely so it fails first when this lands.

Sequence AFTER the ordered pages and the conformance packet, so the packet can prove both models agree on every all-UTF-8 corpus before switching. Reopening contract.py twice in quick succession is the main reason not to do it now.
