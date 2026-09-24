---
type: is
id: is-01m2s053k7cjccm6c6c4rwck2f
title: "Maintainer: crates.io and PyPI accounts, 2FA, and protected release environment"
kind: chore
status: open
priority: 0
version: 8
labels:
  - release
  - security
dependencies:
  - type: blocks
    target: is-01m2phzm7r5vw97jr9y4mtzxxs
  - type: blocks
    target: is-01m2phzmjh2sk8fzqya61v47rd
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
child_order_hints:
  - is-01m395wcbdbx8ndh16agfhmrhy
created_at: 2026-09-18T00:55:07.111Z
updated_at: 2026-09-24T07:46:05.784Z
---
Human-only console work before minting the 0.1.0 crates.io token: confirm the Flowmark maintainer crates.io and PyPI accounts have verified email and 2FA. Create the GitHub environment named release with a required reviewer (Prevent self-review off), one Tag rule v* and no branch rules, and administrator bypass off. It must exist and be protected before any publishing run: the PyPI pending publisher for fdu (registered 2026-09-24: owner jlevy, repository fdu, workflow release.yml, environment release) trusts whatever job names that environment, and GitHub creates an unprotected one the first time a job does. The workflow's release-environment job refuses to publish until all three settings hold. Register the crates.io trusted publishers only after 0.1.0 exists and the CARGO_REGISTRY_TOKEN secret is deleted. Procedure: docs/project/guides/release-process.md#first-time-channel-setup. (Description corrected 2026-09-24 while addressing PR #123 review R6.)
