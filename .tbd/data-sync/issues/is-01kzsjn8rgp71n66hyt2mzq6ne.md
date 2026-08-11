---
type: is
id: is-01kzsjn8rgp71n66hyt2mzq6ne
title: Machine output lost per-entry raw identity in the five-axis rewrite
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-11T23:31:25.583Z
updated_at: 2026-08-11T23:41:34.777Z
---
Found while resolving a merge conflict from PR #6, not by any test. The pre-five-axis CLI emitted name_raw beside every entry name in JSON, carrying the raw OS bytes for a path that to_string_lossy cannot represent. The rewrite kept raw_identity_json and kept root_raw on the envelope, but no view emitted it per entry, so two files whose names differ only in bytes that are not valid Unicode rendered identically in machine output with no way to tell them apart. That is the lossless-identity guarantee, and it is exactly the class of silent wrongness the project forbids.

The tests that would have caught it were dropped in the same rewrite. The spec's Testing Strategy says the existing partial-result, non-UTF-8 identity, broken-pipe, and stack-depth process tests are retargeted, not weakened; the stack-depth one was retargeted and the identity ones were not. They only reappeared because PR #6's branch still carried them and the merge put them in conflict.

Fixed: file rows and tree nodes now carry path_raw on the same terms root_raw is carried - present only when the path is not valid Unicode, so a well-formed tree pays nothing. Tests restored and retargeted to the report path rather than the old write_json, since the guarantee belongs to the format rather than to the flag that used to select it.

Open question for the maintainer: whether adding an optional path_raw field warrants an fdu.report/2 bump. Treated here as additive within v1 because root_raw already establishes the shape in that version, the field is absent for every path that renders losslessly, and it restores rather than changes behaviour. A bump would churn every golden and every consumer for a field that only appears where the data was previously unrepresentable.

## Notes

Follow-up 2026-08-11: the first fix for this shipped invalid JSON and was caught by review as R12. raw_identity_json returned a fragment with the envelope's separator and indent baked in, so reusing it inside a one-line row produced a duplicated comma and an embedded newline - the document did not parse. Root cause was a helper that coupled a value to where it was written; it now returns only the object and each call site supplies its own layout. Fixed in 163046f.

The test lesson is the durable part. The original assertion was contains("path_raw": {...}), which stayed true while the row around it was malformed: a substring check cannot see broken punctuation outside the substring. It now pins the whole row, so the surrounding syntax is part of the contract, plus a structural guard against an empty element anywhere in the document. The tree writer had the identical defect and was equally untested; covering it needed a directory whose name is not valid Unicode, because a tree lists directories rather than files.

Note for anyone re-testing this by hand on macOS: APFS enforces UTF-8 filenames, so a non-UTF-8 name cannot be created there at all. That is why these tests build a synthetic index rather than a real tree, and why an end-to-end check of this guarantee only works on a filesystem that permits arbitrary bytes, such as ext4.
