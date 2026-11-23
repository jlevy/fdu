---
type: is
id: is-01m83e4bbgt5vcav71xx34rh83
title: "Charted page: read verdict.kept for rejected speed claims that keep the candidate"
kind: task
status: open
priority: 2
version: 2
labels:
  - performance
dependencies: []
created_at: 2026-11-23T05:30:15.781Z
updated_at: 2026-11-23T05:30:57.458Z
---
iteration_kind in explorations/benchmarks/realtree/report_html.py counts every rejected record as 'tried, not kept' (red), but 15 rejected records keep the candidate (kept: candidate): exp-164, exp-165, exp-167, exp-196, and the 0.5.0 records exp-210 to exp-215, exp-217, exp-218, exp-220, exp-222 and exp-223, which price a capability shipped regardless; exp-223 is also the header's release standing. The publishing rule says the page reads verdict.kept. Options from review A on #216 (A7): classify rejected with kept == candidate as measured/grey (the red count falls from 75 to 60), or relabel the red series 'rejected verdicts'. Either changes a headline count on the page and needs the maintainer's choice; update the evidence report's Reading the Charted Page paragraph with it.
