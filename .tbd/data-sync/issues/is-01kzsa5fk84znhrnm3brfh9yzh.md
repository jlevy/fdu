---
type: is
id: is-01kzsa5fk84znhrnm3brfh9yzh
title: "PR#6 D3: exp-012 artifact does not validate and retains template prose"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:59.687Z
updated_at: 2026-08-11T21:29:59.648Z
closed_at: 2026-08-11T21:29:59.648Z
close_reason: "Fixed and verified on PR #6; disposition posted to the PR"
---
softschema validate returns 'timestamps are not supported' (unquoted ISO date parsed as YAML timestamp). Body still generated placeholder text. Fix renderer quoting, write real hypothesis/interpretation, validate every artifact in the handoff gate, pin the validator. High.
