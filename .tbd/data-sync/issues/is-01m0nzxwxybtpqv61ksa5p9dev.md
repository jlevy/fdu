---
type: is
id: is-01m0nzxwxybtpqv61ksa5p9dev
title: "PR #42 review R9: duplicated stale doc line on AnalysisSet::parse from the main merge"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:03.966Z
updated_at: 2026-08-23T00:39:55.870Z
closed_at: 2026-08-23T00:39:55.870Z
close_reason: Fixed. The duplicated pre-merge doc line is gone.
---
crates/fdu-core/src/content/content_model.rs:125 re-adds the pre-merge wording that line 120 already supersedes. Merge artifact from 7f914e7.
