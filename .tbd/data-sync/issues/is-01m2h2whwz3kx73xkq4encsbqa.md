---
type: is
id: is-01m2h2whwz3kx73xkq4encsbqa
title: "PR #58 review PR58-REC-2: H102 row and exp-104 misdescribe what exp-069's 8% covered"
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m2h2t7k65srszyhv02tag8ba
created_at: 2026-09-14T23:08:57.117Z
updated_at: 2026-09-14T23:27:39.940Z
closed_at: 2026-09-14T23:27:39.939Z
close_reason: "bd8ed0b: H102 row and exp-104 now say exp-069's 8% covered the roll-up map and the loader candidate map (content_cache.rs), H103 took only the roll-up half, the loader half is untested; the oracle-share claim about exp-069's profile is removed."
resolution: null
duplicate_of: null
---
PR #58, P3. `docs/project/guides/performance-loop.md:797` (H102 row) and the exp-104 artifact's "The 8% estimate" section, at 3a67552.

exp-069's "What is left on this tier" named two sites for the 8%: the roll-up `HashMap` and the candidate map the sidecar loader builds (`crates/fdu-core/src/content/content_cache.rs:145-152`). H103 changed only the roll-up map. The 21.06% oracle share was measured in exp-104 on the registry subject, not in exp-069's profile.

Fix: restate the correction so neither text misdescribes exp-069.
