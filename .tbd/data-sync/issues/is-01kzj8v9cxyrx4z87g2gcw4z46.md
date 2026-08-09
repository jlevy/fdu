---
type: is
id: is-01kzj8v9cxyrx4z87g2gcw4z46
title: "Address review: PR #1 — delta, cache, freshness, and surface contracts"
kind: task
status: closed
priority: 1
version: 22
labels:
  - pr-review
dependencies: []
child_order_hints:
  - is-01kzj8wcf1ajpry8ackwher1q7
  - is-01kzj8wcps39f24hgtv04942yw
  - is-01kzj8wcxwykkx7h4beh271p1p
  - is-01kzj8wd52dh1pxg4834fb2q0v
  - is-01kzj8wdcey7qb1bkq7y9m2f3q
  - is-01kzj8wdks9n0fx4559896d6fc
  - is-01kzj8wdv4jsmw5msz246hqbbc
  - is-01kzj8we2pr4g03f5tdt8n7t08
  - is-01kzj8wea1ah7t8dx4d5tyfn3v
  - is-01kzj8wehbddw264kg8bz0zy11
  - is-01kzj8weryq7v9tf11vr9q5psv
  - is-01kzj8wf0jzt8asjpae7eynvp1
  - is-01kzj8wf8380p0rf53pzemj4w7
  - is-01kzj8wffqtbceqnxt2718wg9q
  - is-01kzj8wfq2ft9scx1gw0sf347p
  - is-01kzj8wfybj8pm6j0f5cxmvwr4
  - is-01kzj8wg5gkkrkkdsvsnrjw5tw
  - is-01kzj8wgcxppt8qpvkzw907j0s
created_at: 2026-08-09T03:25:16.060Z
updated_at: 2026-08-09T04:25:58.862Z
closed_at: 2026-08-09T04:25:58.862Z
close_reason: "All R1-R9, C1-C6, S1/S3, and D1 review implementation beads are complete; S2 is deduplicated with its PR slice complete and S4 remains explicitly deferred to the existing packed-record phase-1 gate. Commits 89e5d4a, e3b8f90, and ce3033c are pushed; full local make check and GitHub CI run 31294466206 are green; final disposition: https://github.com/jlevy/fdu/pull/1#issuecomment-5229752603"
---
Address every unresolved finding in the senior engineering review and Cursor inline threads for PR #1. Source review: https://github.com/jlevy/fdu/pull/1#issuecomment-5229523550. Deliver explicit per-finding dispositions, regression coverage, full local validation, pushed commits, green CI, and a final PR comment.

## Notes

Disposition map uses R1-R9 and C1-C6 as published. All blocking findings and Cursor defects are implemented and locally validated. S1 is complete; S2 is deduplicated into fdu-jej9 with its PR acceptance slice complete; S3 is complete; S4 remains intentionally deferred under fdu-1gbl as the phase-1 packed-record memory gate; D1 is complete. Additional hardening found during implementation covers root-escape rejection, partial-directory deletion safety, ambiguous rename escalation, snapshot structural validation, and allocated-size JSON ordering. Remaining parent exit criteria: commit/push, green post-push CI, and final PR disposition comment.
