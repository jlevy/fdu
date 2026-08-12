---
type: is
id: is-01kzt7d6pak3phy4mnhwbtg2en
title: "PR#6 R7: record.py validation skip path raises instead of skipping"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-12T05:34:01.417Z
updated_at: 2026-08-12T05:37:16.426Z
closed_at: 2026-08-12T05:37:16.425Z
close_reason: "Fixed and mutation-checked in faeb7df; disposition posted to PR #6"
---
benchmarks/realtree/record.py:259-266. _validate documents a skip when softschema is unreachable, but _validator() raises SummaryError before subprocess.run and that is not in the caught tuple (FileNotFoundError, TimeoutExpired). softschema lives in the dev group and record is normally invoked without it, so a missing validator turns optional post-write validation into a hard crash. Medium.
