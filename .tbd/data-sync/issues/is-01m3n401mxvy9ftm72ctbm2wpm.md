---
type: is
id: is-01m3n401mxvy9ftm72ctbm2wpm
title: "Release workflow: pin the tag signer's identity, not just GitHub's verified flag"
kind: task
status: open
priority: 1
version: 1
labels:
  - release
  - security
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T23:00:59.677Z
updated_at: 2026-09-28T23:00:59.677Z
---
Review of 885ce451 (fdu-808x): scripts/release/resolve_plan.py validate_published_tag accepts verification.verified == true, which only says some GitHub account's registered key signed the tag. A collaborator able to create v* tags could delete and recreate v0.2.1 signed by their own key on the same main commit; plan and publish would pass and the maintainer would see a green run to approve. make release-verify-tag (maintainer.py ~841-856) binds to SIGNING_KEY and the tagger email, so the workflow is weaker than the local check. Options (maintainer decision): (a) require the tag's signing key fingerprint or tagger identity to equal a constant committed in the workflow or policy file; (b) a repository ruleset restricting creation and deletion of v* tags to the maintainer, documented in release-process.md; (c) both.
