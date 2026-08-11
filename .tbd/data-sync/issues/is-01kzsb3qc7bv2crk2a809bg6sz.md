---
type: is
id: is-01kzsb3qc7bv2crk2a809bg6sz
title: "PR#6 R6: ledger verdict still gates on the deprecated significant flag"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:19:30.694Z
updated_at: 2026-08-11T21:29:59.673Z
closed_at: 2026-08-11T21:29:59.673Z
close_reason: "Fixed and verified on PR #6; disposition posted to the PR"
---
benchmarks/realtree/ledger.py:73-74. verdict() reads entry['significant'] directly; a comparison carrying only passes_acceptance raises KeyError or mis-decides. Low.
