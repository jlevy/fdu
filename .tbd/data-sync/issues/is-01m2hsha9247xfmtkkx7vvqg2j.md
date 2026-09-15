---
type: is
id: is-01m2hsha9247xfmtkkx7vvqg2j
title: "PR #55 review PR55-ACCT-3: rename case states only one of the two outcomes that move hard-link attribution"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2hsgt9d35edzxt2rqvfc3pv
created_at: 2026-09-15T05:44:46.113Z
updated_at: 2026-09-15T05:50:00.355Z
closed_at: 2026-09-15T05:50:00.354Z
close_reason: "a2e1eba: Delta Accounting states attribution follows whichever in-scope link sorts first after a rename, with both outcomes that move bytes into or out of an unchanged directory; the expected-delta case lists all four rename outcomes; open question 1 says into or out of"
resolution: null
duplicate_of: null
---
PR #55, delta review 5205945198. Plan @55ce4a3 :571-575 and :171-174 state that renaming the attributed link so it no longer sorts first moves unique allocated bytes to another link's unchanged directory. The symmetric outcome is missing: a non-attributed link renamed so it now sorts first moves the bytes out of an unchanged directory. Fix: state both (attribution follows whichever in-scope link now sorts first, so it can leave or arrive at a directory where no entry changed) in both places.
