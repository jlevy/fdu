---
type: is
id: is-01kzsa5g94w28vsfda0fvevabn
title: "PR#6 D6: partial-friendly serialization declared as invariant without a persistence design"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:03:00.387Z
updated_at: 2026-08-11T21:03:00.387Z
---
AGENTS.md:77-86 says serialization accepts partial structures, but snapshot save rejects a non-fresh index and no format encodes frontier/unknown children/eviction/cancellation. Either specify it or narrow the invariant now. Medium.
