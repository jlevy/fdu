---
type: is
id: is-01m49a8zrfceba7c7vfb6r76mh
title: 'The --depth refusal quotes the value as empty: invalid --depth ""'
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m48xs24ec44m1prxyt7xany5
created_at: 2026-10-06T19:15:32.750Z
updated_at: 2026-10-06T19:15:32.750Z
---
On main 55d66863, 'fdu . --analyze=code --depth=2' (and --depth=1) prints 'invalid --depth "": requires a hierarchical view'; the refused value is shown empty instead of '2'. Quote the value the user passed. Found by the README senior reviews on PR #183 (2026-10-06).
