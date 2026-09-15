---
type: is
id: is-01m2jxj1g5139zvb931w9pkren
title: ci.yml never runs scripts/check-uv-version.test.mjs
kind: bug
status: open
priority: 3
version: 1
labels:
  - stack-followup
  - release
dependencies: []
created_at: 2026-09-15T16:14:18.628Z
updated_at: 2026-09-15T16:14:18.628Z
---
Found while fixing fdu-pd1b on branch claude/release-workflow-fixes (commit eb89150). .github/workflows/ci.yml:28, in the supply-chain job, runs 'node --test scripts/check-supply-chain.test.mjs' only. scripts/check-uv-version.test.mjs has no CI caller: it runs via 'npm run test:supply-chain' (package.json:11) inside 'make supply-chain' and 'make check', and in release.yml:35 (the plan job of a rehearsal that has never been dispatched). It holds the uv-version guard tests, the recipe-coverage test (every uv-backed Make target depends on uv-version), the one-reviewed-uv-version policy test, and now the WHEEL_PYTHON pin test (every environment a wheel smoke creates names a GIL-enabled interpreter). A PR can therefore break any of those Makefile contracts with CI green. Fix: add scripts/check-uv-version.test.mjs to the node --test line in ci.yml's supply-chain job (the tests need only make and node; they stub uv), and watch it pass on ubuntu before relying on it in the release plan job.
