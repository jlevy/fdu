---
type: is
id: is-01m0kazpvjhxt3b3436wrw0et3
title: "Python parity CLI: argv-compatible fdu over the public Python package"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
parent_id: is-01m0k965p7hx4dy6t0cj29rsae
created_at: 2026-08-21T23:37:34.321Z
updated_at: 2026-08-22T06:53:19.849Z
closed_at: 2026-08-22T06:53:19.848Z
close_reason: "tests/parity/py/parity_cli.py serves fdu's argv through the public Python package alone -- not a wrapper around the binary. 88 of 126 sessions reach parity immediately, with report bodies byte-identical across every view, format, and selection axis. The 38 that do not are tracked gaps: fdu-1kw3, fdu-gw5b, fdu-jozr."
---
A test-only executable accepting fdu's argv and producing fdu's bytes, over the public
`fdu` Python package and nothing else.

Not a wrapper around the binary -- that would test nothing. It parses argv, builds
ScanOptions / AnalysisOptions / Selection / Query, calls open/scan/report, and renders.
Machine formats come from Report.as_dict(), which already promises "an independent copy of
the exact CLI JSON schema".

Its diagnostics say `fdu:` and it exits 77 for what the surface cannot serve (cache
lifecycle, --watch, --skill, --docs).

The shim's length is a measurement. Elaborate glue, private helpers, or a capability the
package does not expose is a finding about the API, not an implementation detail.
