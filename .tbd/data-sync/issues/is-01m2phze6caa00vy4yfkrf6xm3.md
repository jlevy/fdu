---
type: is
id: is-01m2phze6caa00vy4yfkrf6xm3
title: SSH tag-signing identity for v0.1.0
kind: chore
status: open
priority: 0
version: 3
labels:
  - release
  - security
dependencies:
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
  - type: blocks
    target: is-01m2phzkwcvz3fwk2nh150bdmj
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T02:08:52.428Z
updated_at: 2026-09-17T02:08:58.251Z
---
The runbook requires `git tag -s` and `git tag -v`. This host has no GPG secret keys and no git
signing config; an SSH test tag signed and verified with ~/.ssh/id_ed25519.pub using one-off flags.
Remaining (maintainer actions):
- `gh auth refresh -h github.com -s write:ssh_signing_key`, then `gh ssh-key add ~/.ssh/id_ed25519.pub --type signing`
  (GitHub currently lists no signing keys for jlevy) so the tag shows Verified; the tagger email must be verified on the account.
- `git config --global gpg.format ssh`, `user.signingkey`, and `gpg.ssh.allowedSignersFile` with an allowed-signers line.
