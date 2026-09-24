---
type: is
id: is-01m2pj0p31w8pg5wahbayvvy8d
title: PyPI OIDC publish job over the validated artifacts
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2pj0q04xsqtjqj2v62vhhq5
parent_id: is-01m2pj0jfrvm78rpmv6rwc2ktv
created_at: 2026-09-17T02:09:33.280Z
updated_at: 2026-09-24T07:50:46.375Z
closed_at: 2026-09-24T07:50:46.374Z
close_reason: "Implemented by PR #123 as the single publish job: uv publish --trusted-publishing always --check-url over the re-verified rehearsal files, wait-pypi on the manifest digests."
resolution: null
duplicate_of: null
---
No checkout; decide gh-action-pypi-publish (PEP 740 attestations) or uv publish.
