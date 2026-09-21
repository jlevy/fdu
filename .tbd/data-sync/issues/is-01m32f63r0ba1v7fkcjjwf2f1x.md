---
type: is
id: is-01m32f63r0ba1v7fkcjjwf2f1x
title: "Parity classifier absorbs a changed answer: run-telemetry matches on counts, not content"
kind: bug
status: closed
priority: 0
version: 2
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T17:10:58.560Z
updated_at: 2026-09-21T17:22:34.219Z
closed_at: 2026-09-21T17:22:34.219Z
close_reason: Fixed in 784638be on claude/gate-integrity. The run-telemetry remainder must now be equal line for line, not merely equinumerous. All four recorded sessions still classify and nothing becomes unexplained. scripts/parity-classes.test.mjs covers it, including a case that adds one genuinely changed line to an otherwise explained session and asserts no class absorbs it; against the previous matcher two of its five tests fail. Wired into test-parity and parity-check.
resolution: null
duplicate_of: null
---
Verified by execution, 2026-09-21. This is the gate that backs "every surface gives the same answer", and it can be fooled.

`scripts/parity-classes.mjs`, class `run-telemetry`. The matcher is:

    removed.some(note|Performance) && removed.filter(!note|Performance).length === added.length

It compares COUNTS, not content. Probed directly by importing `classify`:
- `{removed:[note], added:[]}` -> run-telemetry (genuine)
- `{removed:[note, "256 B  7 files, 4 directories"], added:["512 B  9 files, 4 directories"]}` -> run-telemetry
- `{removed:["Performance: ...", "total 100"], added:["total 999"]}` -> run-telemetry

So any parity session where Python lacks a `note:` line can also carry a wrong tally and still be classified as explained. Several such sessions exist today in `tests/parity/deviations-python.diff` — for example "Content Cache Hits Preserve the Same Tallies" at lines 198-205.

`Performance:` is stripped from the corpus by `scripts/run-parity.mjs:86-93`, so `note:` is the live trigger.

The file's own header says a class must say what the difference IS. A length-only match does not.

Fix: the class must pair each removed line with the added line it explains and assert the remainder is empty, rather than counting. Add a unit test per class with a "one real change added" negative case, so a class that starts absorbing real differences fails.
