---
type: is
id: is-01m2s053k7cjccm6c6c4rwck2f
title: "Maintainer: crates.io and PyPI accounts, 2FA, and protected release environment"
kind: chore
status: open
priority: 0
version: 6
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
created_at: 2026-09-18T00:55:07.111Z
updated_at: 2026-09-18T00:56:45.120Z
---
Human-only console work before minting 0.1.0 tokens: confirm the Flowmark maintainer crates.io and PyPI accounts have verified email and 2FA; create the GitHub environment named release with a required reviewer and v* tag policy; do not register trusted publishers or a PyPI pending publisher until after 0.1.0 exists and the environment is protected. Procedure: docs/project/guides/release-process.md#first-time-channel-setup.
