---
type: is
id: is-01kzj8wg5gkkrkkdsvsnrjw5tw
title: "PR #1 review S3: Align local and CI gates with pinned inputs"
kind: chore
status: closed
priority: 2
version: 3
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzj8wgcxppt8qpvkzw907j0s
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:55.759Z
updated_at: 2026-08-09T04:17:09.749Z
closed_at: 2026-08-09T04:17:09.748Z
close_reason: Pinned and frozen local/CI inputs, immutable Action SHAs, opt-in checksum-verified bootstrap, cargo-deny policy, and installed-wheel smoke are implemented; full local make check passes.
---
PR #1 non-blocking suggestion S3. Files: Makefile, .github/workflows/ci.yml, Python packaging, and session bootstrap scripts. Use locked or frozen resolution, commit-SHA-pinned Actions, an explicit wheel smoke gate, and opt-in session bootstrap without proxy bypass. Preserve the documented 14-day dependency policy.
