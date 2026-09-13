---
type: is
id: is-01m2eb2cnp3v1qyh4zkbqqv6br
title: "PR #54 review H86-5: rejected evidence stage makes the page plot the pre-H86 control as Linux's product state"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eb1cnhke030h15e79fyzvc
created_at: 2026-09-13T21:34:13.685Z
updated_at: 2026-09-13T22:07:34.144Z
closed_at: 2026-09-13T22:07:34.143Z
close_reason: "Fixed in 4dccad8 with the minimum option: CLAIM_ONLY_EXPERIMENTS = {exp-103} makes kept_variant return None and figure_per_entry skip the record, so the page no longer draws 4.23 us/entry as Linux's current cost. Rejected recording 'accepted' (would say the Linux stage passed) and a new decision value (contract change; merged vocabulary tracked in fdu-02a0); plotting the candidate would misstate the shipped binary. Two tests written first; performance-loop.md documents the hand-maintained list."
resolution: null
duplicate_of: null
---
Medium. timeline.kept_variant maps every non-accepted decision to control, and report_html.figure_per_entry plots the kept arm as a subject's current cost. For linux-450k the page shows 4.23 us/entry unchanged, while #52 ships the candidate (3.42 us/entry). The experiment rejects a pre-registered floor claim about code that ships on its Darwin acceptance; the decision vocabulary has no value for 'candidate retained, floor claim unmet'. Fix (pick one): record the decision as the relative-gate outcome; extend the decision vocabulary; or at minimum exclude this record from the kept-arm trajectory.
