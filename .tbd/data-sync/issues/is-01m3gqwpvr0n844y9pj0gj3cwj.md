---
type: is
id: is-01m3gqwpvr0n844y9pj0gj3cwj
title: Avoid duplicate aggregate watch output after invisible metadata changes
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-27T06:12:29.678Z
updated_at: 2026-09-30T04:07:02.399Z
closed_at: 2026-09-30T04:07:02.399Z
close_reason: "Fixed in ea5a1af3: watch_session::Session::changed_report renders the answer with its generation instant pinned, appends its tree status and its source and freshness, and answers None when that identity is the one it last handed out; the command line asks for its initial answer and every repaint through it (render_live), so an idle tree, an mtime-only touch under a size-only tree, and a change to an entry the selection leaves out print nothing, while a visible change, a JSON repaint after a touch (JSON carries newest_mtime_ns), and a status change repaint. Change records are never deduplicated; Session::report always answers. Tests (scripted watcher, real files): an_aggregate_repaint_is_skipped_when_a_reader_would_see_no_change (idle, touch via File::set_modified, filtered change, visible change, only-once), a_repaint_identity_is_what_the_format_renders (JSON), an_invalidation_keeps_its_change_record_and_repaints_by_the_answer. CLI tests and the watch goldens pass unchanged. CHANGELOG Fixed entry; the design principles' watch section states the rule. Python's Watch.report() is unchanged and explicit; the engine method is available to it but not yet exposed as a Python method."
resolution: null
duplicate_of: null
---
Observed on macOS at c5e2f6ed: run fdu on a one-file fixture with --watch --cache off --interval 200ms --color never. Idle produces no output. Touching the file without changing its 4096-byte size emits a new timestamp separator and the identical 4.0 KiB / 1 file tree. Growing it to 16384 bytes correctly emits 16 KiB. Session::next_batch marks any effective commit dirty before selection filtering; CLI run_watch reevaluates and writes the report for every dirty batch. Desired behavior: preserve native-event processing and index correctness, suppress unchanged aggregate presentation, and investigate query-aware invalidation to avoid unnecessary evaluation. Define output identity explicitly without volatile generated timestamps; retain meaningful tree-status, freshness, and invalidation changes and never deduplicate required file change-stream records. Add tests for idle, mtime-only default-tree changes, filtered changes, visible changes, and invalidation. This is a diagnosis only; no watch code changed in PR #132.
