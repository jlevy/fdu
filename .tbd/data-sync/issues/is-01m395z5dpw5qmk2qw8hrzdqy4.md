---
type: is
id: is-01m395z5dpw5qmk2qw8hrzdqy4
title: "Decide: PyPI trusted-publishing preflight before the first crates.io write"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m395wcbdbx8ndh16agfhmrhy
created_at: 2026-09-24T07:44:34.740Z
updated_at: 2026-09-24T07:44:34.740Z
---
PR #123 review S1. The irreversible first write is crates.io; the never-exercised path is PyPI's OIDC mint against the pending publisher, which comes after it. Under uv 0.12.1, 'uv publish --dry-run --trusted-publishing always --check-url https://pypi.org/simple/ <files>' mints the token before validating files; for a pending publisher that converts it and creates the (empty) project, which reserves the name and proves the subject before any crate is written. It is a dry run with a side effect, so it needs a maintainer yes; if taken, add it as a step before 'Audit both registries' in the publish job. Not applied by the addresser.
