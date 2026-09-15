---
type: is
id: is-01m2h78ksjwgx2sjnev8byt3qe
title: "PR #57 review PR57-8W5K-3: Rust selection validation checks uniqueness last, Python and MetaBrowser first"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2h77qemaay1jkhbhfzh35me
created_at: 2026-09-15T00:25:26.576Z
updated_at: 2026-09-15T00:28:47.982Z
closed_at: 2026-09-15T00:28:47.978Z
close_reason: "4f78ab1: Rust admit_* and check_list test uniqueness first, matching CatalogQuery and Python; a list wrong twice gets one message on every surface, tested in Rust and pytest. Holds at c141285; a_list_wrong_twice_is_refused_in_the_catalog_query_order passes."
resolution: null
duplicate_of: null
---
PR #57 review https://github.com/jlevy/fdu/pull/57#pullrequestreview-5200240760, P3. At 7b804df, crates/fdu-core/src/query/query_selection.rs:299-332, :442-471 and crates/fdu-py/python/fdu/opened.py:544-560: a list both duplicated and malformed got a different refusal message per surface.
