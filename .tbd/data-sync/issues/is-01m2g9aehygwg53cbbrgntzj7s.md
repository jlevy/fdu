---
type: is
id: is-01m2g9aehygwg53cbbrgntzj7s
title: Decide whether a one-shot report refuses read_controls=true instead of ignoring it
kind: task
status: closed
priority: 3
version: 3
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T15:42:09.469Z
updated_at: 2026-09-15T22:18:00.478Z
closed_at: 2026-09-15T22:18:00.477Z
close_reason: "dcdec5a (PR #65): moot. plan_report no longer overrides read_controls, so a one-shot report honours read_controls: true and there is nothing to refuse; fdu.report forwards ScanOptions.read_controls"
resolution: null
duplicate_of: null
---
Found while implementing fdu-agb6 (commit c06fe47 on claude/contract-decisions).

A one-shot report ignores a caller's request for control state. `prepare_report` overwrites `config.scan.read_controls` with the planner's `false` (`crates/fdu-core/src/execution.rs`, `prepare_report_internal`, `planned.scan.read_controls = plan.read_controls`, at c06fe47). Python now exposes the same field as `ScanOptions.read_controls`, and `fdu.report` does not forward it (`crates/fdu-py/python/fdu/_api.py` `report`, at c06fe47).

So `prepare_report` with `read_controls: true`, or `fdu.report(scan=ScanOptions(read_controls=True))`, silently runs without control observation. Both are documented (`ScanConfig::read_controls`, `prepare_report`, `ScanOptions.read_controls`, `fdu.report`), and no report view reads ignore classification, so no report answer is wrong. But the caller's request is dropped rather than refused, which sits uneasily with "never silently".

Decision needed:
- Keep ignoring it, as documented today.
- Or refuse `read_controls: true` on the one-shot path with a named error. That might mean a typed `InvalidValue` from `prepare_report`, and a `ValueError` from `fdu.report`, so the flag is never a no-op.

Before choosing, check the command line and the parity corpus: neither sets the field on the one-shot path today.

## Notes

2026-09-14 DECISION (user, supersedes the default-off decision recorded earlier the same day): .gitignore information is built into the tool and the library, and is rolled up by default on every surface: CLI reports, --watch, library open, fdu.open/fdu.scan/fdu.report, and opened roots. Each request can turn it off (--no-gitignore on the CLI, read_controls=False in the library and Python). The typed 'not observed' answer from #57 stays, for requests that opt out. The CLI shows split totals, for example '1.2 GB (340 MB ignored)', plus --exclude-ignored and --only-ignored filters. Prerequisites before the default flips: fdu-1onj (the control budget degrades to partial instead of aborting), fdu-okne (a liftable bound named in the error), fdu-szkg (charges deduplicated by fingerprint), and a speed check against main with controls on. Consequence for this bead: one-shot reports now read controls by default, and read_controls=False turns that off. The planner forcing read_controls=false (execution::plan_report) goes away, so the 'report ignores read_controls: true' question is moot once the default flips.
