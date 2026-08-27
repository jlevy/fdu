---
type: is
id: is-01m10nq3f65ssfz0jj2nkxavrn
title: Bootstrap the exact-revision MetaBrowser unchanged-contract spike
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nq3svzaygbyh3mvmkt0g7
parent_id: is-01m0y1sfedr9qf3sc7e4bf6fd7
created_at: 2026-08-27T03:55:14.277Z
updated_at: 2026-08-27T06:12:05.543Z
closed_at: 2026-08-27T06:12:05.542Z
close_reason: Completed on MetaBrowser branch codex/fdu-opened-root-e2e-spike at commit 2743064 against exact fdu wheel revision 0583a1a. The normalized evidence and reproduction commands are under explorations/fdu-inventory-adapter; MetaBrowser make verify and strict exact-wheel typing pass. The branch, revision pin, wheel digest, and import-isolation proof are recorded.
resolution: null
duplicate_of: null
---
Use MetaBrowser branch codex/fdu-opened-root-e2e-spike at PR #74 head 3183888808b366b5ba1c381dec1cbb18b49d969e. Add the exploration manifest and reproducible build/install command that produces an fdu wheel from the exact PR #48 revision, records both revisions, and prevents sibling-source imports; do not amend the provider contract.
