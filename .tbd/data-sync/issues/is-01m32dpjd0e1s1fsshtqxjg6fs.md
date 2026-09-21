---
type: is
id: is-01m32dpjd0e1s1fsshtqxjg6fs
title: "Subset projection precondition: every reader must project to request.basis.content"
kind: task
status: open
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T16:45:00.704Z
updated_at: 2026-09-21T17:06:20.905Z
---
`crates/fdu-core/src/query/query_report.rs:440` — `Query` carries `selection`, `views`, `omitted_views`, `axes` and `words_per_page`, but no analyzer set. `query::report` therefore reads `index.content()` and cannot distinguish what was requested from what is stored.

That is currently harmless only because `stored_state.rs` serves content by equality (`Serves::{Exact, Refuse}`), so the served tier's analyzer set always equals the request's. The report is incidentally correct, not correct by construction.

The plan defers subset projection ("Serving fewer analyzers from a record set holding more arrives after per-analyzer records ship; until then a different analyzer set re-reads"). The deferral has an unstated precondition: the day serving is widened again, the report resumes presenting whatever the index holds, which is exactly the 2026-08-21 defect in #37.

Make the precondition explicit in the plan's deferral entry and in the design principles: subset projection may not be un-deferred until the analyzer set is part of the request model. This costs nothing now and removes the trap.

## Notes

CORRECTED BY REVIEW, 2026-09-21. The precondition as originally written named the wrong thing and was already satisfiable, so it would have provided no protection.

The request model ALREADY carries the analyzer set: `Request.basis.content` (`query_request.rs:46`), read by `report` at `query_report.rs:1017` and enforced by `validate_read` (`query_request.rs:401-407`, `ContentMismatch`). `Query` lacking it is immaterial. So "may not be un-deferred until the analyzer set is part of the request model" could be declared met today while the trap remained.

The trap is on the reader side: readers still consume stored records wholesale — `query_report.rs:1486` (`record.metrics`, `record.profile`) and `:1613` — and the only in-report tripwire is a `debug_assert_eq!` at `:1054`, absent in release builds.

Correct precondition: subset projection may not be un-deferred until every reader in `query_report.rs` derives metrics, `document_words`, `analysis.analyzers` and coverage from `request.basis.content` through a projection, that projection is exercised by the harness on every route, and `validate_read` uses the same projection rather than an equality check.

The same stored-not-requested shape exists for four more request elements, and the clause should cover them:
- `.gitignore` observation: `every_entry` reads `index.observes_controls()` (`:1707`) and `report.scope = index.scope()` (`:1046`), projected on one route only (see fdu-qsos).
- type rules: `index.classify` (`:1488`) uses the stored rules.
- reducers: roll-ups use the stored ones.
- `Request.now` (`query_request.rs:154`) is consumed nowhere in `query_report.rs`; relative windows resolve in `cli.rs:648` at parse time, so the plan item "carry content and now to the reader" is genuinely unchecked.

Size mode and `Selection` are read from the request and are fine.
