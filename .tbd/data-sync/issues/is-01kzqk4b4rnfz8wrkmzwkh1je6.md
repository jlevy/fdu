---
type: is
id: is-01kzqk4b4rnfz8wrkmzwkh1je6
title: "PR #3 review R12: require an explicit multi-variant headline pair"
kind: bug
status: closed
priority: 2
version: 4
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzqk4awe65tvmt9c6ynnb5zc
  - type: blocks
    target: is-01kzqk493tkcy6nwws6vf9md7f
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T05:01:10.679Z
updated_at: 2026-08-11T05:27:25.095Z
closed_at: 2026-08-11T05:27:25.094Z
close_reason: Recording now requires both explicit variant names whenever a job has multiple comparisons, rejects unknown/partial pairs, and uses that same pair for the headline and frontmatter. Added multi-comparison, reversed-order, and missing-pair tests.
---
FDU-PR3-R12 and Cursor discussion r3754432433. benchmarks/realtree/record.py silently takes the first comparison when variant flags are omitted. Require explicit variants for multi-comparison runs, derive all headline fields from the same pair, fail closed, and add reversed-order coverage. Review: https://github.com/jlevy/fdu/pull/3#issuecomment-5249058288.
