---
type: is
id: is-01m32ewm52a3d4wrd6mhsx5dyd
title: The content tier decides reuse at five inline == sites and never touches Serves
kind: bug
status: open
priority: 0
version: 1
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T17:05:47.681Z
updated_at: 2026-09-21T17:05:47.681Z
---
Verified by reading in an adversarial review, 2026-09-21. This is the finding that invalidates the claim that equality-serve makes content reuse correct by construction.

Content reuse is decided by five independent equality comparisons, none of which goes through `stored_state::Serves`:
- `content/content_index.rs:333` (`prepare`)
- `content/content_cache.rs:163` (`load_content_cache`)
- `content/content_cache.rs:513` (`parse_header`)
- `index.rs:3543` (`pending_analysis_candidates`)
- `content/content_index.rs:278` (`holds_record`, record level)
plus `query/query_request.rs:402` (`validate_read`).

#37 (`2aa7da1`), the commit that widened equality to containment and caused fdu-gija, touched `content_cache.rs`, `content_index.rs`, `content_model.rs`, `index.rs` and `query_report.rs` — none of which would have gone through `Serves`. So the enum would NOT have prevented #37 and does not prevent a recurrence today.

Confirmed constructively: a re-widening mutant needed three edits (`lib.rs:773`, `query_request.rs:402`, `query_report.rs:1054`) and never touched `stored_state.rs`.

Fix: route all six content sites through one `serves_content` returning a value-carrying projection, so a record set cannot be consumed without the projection that makes it answer the request.

Until this lands, describe fdu-gija as fixed-by-equality, not as correct by construction.
