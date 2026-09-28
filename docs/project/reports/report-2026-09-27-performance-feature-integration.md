# Performance and Feature Integration Validation

## Scope

Merge [#136](https://github.com/jlevy/fdu/pull/136) at `5365e5af` into
[#137](https://github.com/jlevy/fdu/pull/137), whose performance head was `89efe879`.
This includes #136’s underlying feature stack, not only its final documentation changes.

The conflict resolution preserves H153’s shared classification and content lookup for
multiple unfiltered Types, Families, Languages, and Documents views.
It also preserves the incoming content-metric sorting, minimum-share filtering,
omitted-row counts, and tree measurements.
Code overview remains separate because its classification comes from admitted content
detection rather than the current path classification used by the metric views.

The regression test compares combined and independent reports across 18 combinations:
three ignored populations, three sorting choices, and two share thresholds, with a
one-row limit. Its fixture includes ignored code and an extensionless script.
The existing exact probe oracle remains outside the component timer.

The upstream branch was rewritten during validation to add size/rate terminology,
progress display, and cache-status color changes.
The final merge targets `5365e5af`, not the initially fetched `eaa60102`. Its failing
parity check was a stale recorded diff, not a newly unclassified deviation.
The updated record is byte-identical to the `deviations-python-linux` artifact from
[Linux CI run 36366853621](https://github.com/jlevy/fdu/actions/runs/36366853621). No
macOS parity recording was substituted for the Linux-owned evidence.

The local Python gate initially loaded an ignored editable native extension built before
the merge, causing ten `cache_dir` keyword failures.
Forcing
`uv run --directory crates/fdu-py --frozen --group dev --reinstall-package fdu pytest`
rebuilt the extension and passed all 70 tests.
The full `make check` gate then passed, including the installed wheel and source
archive, CLI/Python parity, and the 2,267-case path-independence subset.
`make cross-lint` passed for Intel macOS and Windows.
Deterministic editable-build freshness is tracked as `fdu-35b1` rather than worked
around in the public API.

The first pushed integration exposed an older macOS test race (`fdu-93gd`): two
independent native watchers can retain different legitimate setup-gap diagnostics.
Exact progress-observed versus unobserved report equality now uses the existing scripted
handoff seam with identical empty event streams and a nonempty Summary report.
The native test retains public-start, progress, and completeness assertions.
No production behavior changed, and no observation errors are masked.

## Cache Correctness Runbook

The [correctness runbook](../guides/correctness-runbook.md) was run on the combined
source using the debug CLI on bare-metal macOS 26.5.2, Darwin 25.5.0 arm64, as an
unprivileged user. Fixtures and temporary caches were on an external USB APFS SSD. Each
case started with an isolated cache; warm and cache-only answers were compared against
the requested cold answer, and the serving mechanism was checked separately.

| Pass | Result |
| --- | --- |
| Refusal tree | 23/23 partial cases withheld snapshots; no answer or mechanism failures |
| Complete tree | 23/23 cases served cache-only answers; no answer or mechanism failures |
| Cross-warm matrix | 30/30 combinations matched cold answers; zero violations |

Reference-instant checks found no stale instants.
The fixture exercised sparse files, hard links, symlinks, FIFOs, sockets, effective
permission refusals, awkward names, deep and wide directories, and ignore rules.
Character devices, block devices, and non-UTF-8 filenames were unavailable on this
host/filesystem combination; the run makes no claim about those cases or Linux runtime
behavior.

## Performance Evidence Boundary

This integration is correctness validation, not a new timing experiment.
exp-158 and exp-159 retain their recorded pre-integration revisions and measurements.
Neither their 2.5× report-component result nor their 1.9× whole-probe result establishes
the combined engine’s performance.

The next H153 confirmation must compare the integrated candidate with the same feature
stack without H153, keeping the exact report oracle on both sides.
Quiet-host major-fault confirmation and Linux replication remain open.
Bare `--analyze all` now selects Code and Documents, so it still does not request two of
the metric views needed to enter H153’s shared pass.
See [Current Pickup](../guides/performance-loop-runbook.md#current-pickup-2026-09-27).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
