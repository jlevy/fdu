# Output Design Review — 2026-09-27

## Scope and Verdict

Reviewed the uncommitted output-design increment on `codex/code-analysis-implementation`
using the `review-code` and precommit shortcuts.
Scope includes the shared remainder, report schema 9, Python values, `--full`,
diagnostic streams, colors, golden migration, and the manual acceptance fixes.

Two stderr-color findings and one repeated-tip finding were found and corrected.
Verification of their final committed candidate remains part of the handoff gate.
No further correctness finding was identified in disjoint remainder accounting or the
Python model mapping.
This review does not waive the acceptance limits in the
[manual validation report](../reports/report-2026-09-27-manual-acceptance.md).

## Findings and Disposition

### O1 — Medium: Machine Format Suppressed Fatal Diagnostic Color

`run_with_io` passed the stdout machine-format decision into the stderr error renderer.
A JSON request could therefore suppress explicitly requested error color even though its
result and diagnostics used separate streams.

**Fix:** Resolve fatal color from stderr’s own color/terminal context, as report
warnings already do.
Add a runtime missing-root regression that checks empty machine stdout and
forced/disabled stderr color.
Tracking: `fdu-y70h`.

### O2 — Low: Failed Output Bypassed Warning Color Policy

The path that reports a snapshot-save failure after a failed stdout write painted its
warning using raw stderr terminal status.
That bypassed explicit color settings and `NO_COLOR`.

**Fix:** Compute diagnostic color before joining the pending save and use it on both
normal and failed-output paths.
Preserve joining, warning details, and broken-pipe exit semantics.
Tracking: `fdu-y70h`.

### O3 — Medium: Row-Limit Tips Repeated Inside Result Sections

The bounded-section formatter still embedded `--limit all` in each header, and flat
diagnostics reused that text.
This bypassed the shared end-of-report tip collector.

**Fix:** Keep section counts factual and collect one row-limit remedy using the
requesting surface’s vocabulary.
Determine row truncation after share filtering so a share-only omission does not produce
a misleading row-limit suggestion.
A multi-view golden verifies the single tip.
Tracking: `fdu-0578`.

### Coverage Follow-Ups

Review requested executable flat `--full` cases in addition to hierarchical expansion,
and inspection of a zero-row remainder.
The shared corpus covers the flat shorthand; manual examples compare it with explicit
unlimited bounds and verify recursive roll-ups.
The zero-row path preserves a nullable whole-root remainder without inventing a visible
row. Partial coverage propagates unknown counts and bytes.

## Design Assessment

- **One measured model:** `TreeRemainder` sums disjoint omission boundaries in core.
  Text, JSON, JSONL, YAML, and Python consume those same facts.
  Directory totals are never summed with descendant totals.
- **Precise counts:** `entries` counts hidden roots; `files` counts regular files
  recursively. Row-cap pruning discards a hidden node’s internal omission records, so the
  new whole-subtree boundary replaces them rather than counting them twice.
- **Honest unknowns:** Any unknown or overflowing constituent makes that aggregate null.
  Partial coverage cannot yield a falsely exact remainder.
- **Composable shorthand:** `--full` supplies existing neutral bounds; explicit flags
  override it independently of ordering.
  Core accepts neutral values across views while rejecting finite hierarchy bounds on a
  flat projection.
- **Separate streams:** Renderers return results.
  Categorized notes/tips are separate engine values; the CLI inserts operational
  warnings and final human-run telemetry.
  Machine fields retain detailed boundaries and coverage for debugging.
- **Count accuracy:** The shell token-context corrections have hand-counted tests at
  every two-chunk split.
  The analyzer identity is bumped so persisted counts cannot retain the previous
  interpretation. No general shell-parser equivalence is claimed.

The compact terminal line intentionally hides per-boundary repetition, while structured
output preserves it.
Full expansion remains available without changing discovery or analysis.
These choices satisfy the requested presentation without introducing separate CLI-only
accounting.

## Documentation and Evidence

The code comments beside `report_format`, `report_epilogue`, and CLI stream handling
state the design contract.
AGENTS.md and README link the
[output design system](../architecture/fdu-output-design.md).
Current usage, machine schema, Python, bundled skill, and plan documentation cover full
recursion and find/fd-style inventories with their path and ignore-policy distinctions.

The [manual evidence](../reports/manual-acceptance-2026-09-27.json) preserves the
pre-fix installed candidate and subsequent debug checks separately.
The literal mixed remainder golden, full-expansion assertions, shared CLI/Python corpus,
and terminal checks provide complementary enforcement.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
