---
type: is
id: is-01m3mfpqz3cd8w5f9289t6yejb
title: Make the publishing run refuse an unsigned tag or a tag off main
kind: task
status: open
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T17:06:23.330Z
updated_at: 2026-09-28T20:31:50.132Z
---
resolve_plan.py --mode release --validate-checkout (plan job and publish job) proves only that refs/tags/v{Cargo version} names the checked-out commit (git tag --points-at HEAD, which also matches a lightweight tag). Nothing in the workflow checks that the tag is annotated and SSH-signed, or that the commit is an ancestor of main; the only guards are the maintainer's local checks (make release-verify-tag, make release-preflight 'COMMIT on origin/main') and the environment approval. Proposal: in release mode, require the GitHub API's git/tags/<object> verification.verified == true for the tag object and git merge-base --is-ancestor HEAD origin/main (checkout with fetch-depth: 0 or fetch main). Workflow change, so it needs review, and test_metadata.py guards. Found in fdu-n2hc.
