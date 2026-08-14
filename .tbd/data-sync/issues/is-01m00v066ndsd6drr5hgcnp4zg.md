---
type: is
id: is-01m00v066ndsd6drr5hgcnp4zg
title: "PR #22 review R8: make tbd setup reproducible"
kind: bug
status: closed
priority: 1
version: 4
labels: []
dependencies: []
parent_id: is-01m00tzk6myk9ba0110gv86kdz
created_at: 2026-08-14T19:11:53.044Z
updated_at: 2026-08-14T19:54:34.206Z
closed_at: 2026-08-14T19:34:27.936Z
close_reason: "Fixed with exact stable get-tbd 0.6.2: setup regenerated config/scripts, dirty upgrade provenance was removed, all fallbacks pin 0.6.2 and scope the repository's first-party npm-before exception, exact npx reports 0.6.2, and the 13-test plus online provenance gate passes."
---
Medium. PR #22 review R8. .tbd/config.yml:2-9 and zero-install scripts. Align one stable tbd version, remove dirty-development provenance, and validate bootstrap under the documented first-party supply-chain exception.

## Notes

Regenerated from exact stable get-tbd 0.6.2 after 0.6.0 itself reported dirty development provenance. Config, all four managed scripts, and verified supply-chain provenance now agree on 0.6.2; same-core prerelease or dirty versions do not satisfy the stable local-first check.
