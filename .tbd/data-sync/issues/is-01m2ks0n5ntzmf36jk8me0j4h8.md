---
type: is
id: is-01m2ks0n5ntzmf36jk8me0j4h8
title: "PR #67 review PR67-6: the_layout_names_what_default_cache_path_produces passes vacuously"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2krzt6endqrw5gq2phas5g6
created_at: 2026-09-16T00:14:09.076Z
updated_at: 2026-09-16T03:50:02.786Z
closed_at: 2026-09-16T03:50:02.785Z
close_reason: "f39b701: the test expects default_cache_path rather than guarding it with if let, so it fails where no cache directory resolves instead of asserting nothing; the negative name cases moved to a_name_is_shaped_like_one_of_fdus_files_or_like_nothing, which covers all four shapes."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/cache.rs:588-592@816fcf7. The one positive assertion sits behind if let Some(path), so with no cache directory resolvable the test asserts only its three negative cases. DECISION (user): give the test an explicit cache root, or assert the environment it needs.
