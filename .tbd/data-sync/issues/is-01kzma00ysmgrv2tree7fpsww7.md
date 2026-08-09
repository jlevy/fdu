---
type: is
id: is-01kzma00ysmgrv2tree7fpsww7
title: Authenticate supply-chain provenance checks without weakening PR permissions
kind: bug
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - supply-chain
  - merge-blocker
dependencies:
  - type: blocks
    target: is-01kzm3t12dcq5h7n92xztnhcyd
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T22:23:48.696Z
updated_at: 2026-08-09T22:24:40.796Z
closed_at: 2026-08-09T22:24:40.795Z
close_reason: The online gate now selects explicit GITHUB_TOKEN/GH_TOKEN first and securely falls back to the already authenticated local gh credential via execFileSync without a shell or credential output. Workflow validation requires the existing least-privilege contents:read token wiring on the supply-chain job. Added token-selection and missing-CI-auth tests; 10 policy tests and the live check of 66 Cargo packages, 31 npm packages, 2 Python packages, 21 action uses, and all bootstrap pins pass despite the unauthenticated quota being exhausted.
---
The online provenance gate exhausted GitHub's shared unauthenticated 60-request quota and failed make check with HTTP 403. Keep fail-closed provenance validation, but use a local gh credential when no token environment variable is present and explicitly pass the workflow's least-privilege github.token only to the supply-chain job. Test token selection and require the authenticated CI wiring without broadening contents:read permissions or exposing credentials in output.
