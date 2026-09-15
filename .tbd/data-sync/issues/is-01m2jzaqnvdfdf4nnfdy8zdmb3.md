---
type: is
id: is-01m2jzaqnvdfdf4nnfdy8zdmb3
title: "PR #61 review PR61-GUIDE-2: no written recovery for a conflict or failure after fdu-core is published"
kind: bug
status: closed
priority: 2
version: 3
labels:
  - release
dependencies: []
parent_id: is-01m2jzadk7w8m1xcsewzwg5wj1
created_at: 2026-09-15T16:45:16.342Z
updated_at: 2026-09-15T17:10:14.914Z
closed_at: 2026-09-15T17:10:14.913Z
close_reason: "7b3fe90: new Recover From a Partial Publication section (audit verdict table; conflict or new-commit fix ends 0.1.0: stop, record, yank only wrong bytes, never retag, release 0.1.1); crates.io token gains yank scope"
resolution: null
duplicate_of: null
---
PR #61 at eb89150, docs/project/guides/release-process.md:226-232 and :306-308. The one irreversible outcome, registry_state.py reporting conflict for fdu-core (or a failure after fdu-core is on crates.io), has only 'stop'. Write the recovery: do not publish fdu; yank fdu-core 0.1.0 if its bytes are wrong; never retag v0.1.0; bump to 0.1.1, re-rehearse, and start over on the new tag; record what happened.
