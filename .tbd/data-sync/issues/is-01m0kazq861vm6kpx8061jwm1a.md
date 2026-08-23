---
type: is
id: is-01m0kazq861vm6kpx8061jwm1a
title: Parity by committed deviation file, non-empty by construction
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/done/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
parent_id: is-01m0k965p7hx4dy6t0cj29rsae
created_at: 2026-08-21T23:37:34.725Z
updated_at: 2026-08-23T00:05:56.691Z
closed_at: 2026-08-22T06:53:20.279Z
close_reason: "tests/parity/deviations-python.diff is committed, headed with how to read it, and non-empty by construction. Verified reproducible across runs after normalising sandbox paths, cache hashes, RFC3339 stamps and mtime_ns. Verified to catch the dangerous case: a shim secretly exec'ing the Rust binary fails the run."
---
The corpus is not duplicated and neither are the expected bytes. tests/golden stays as the
Rust recording. The parity run replays the same sessions against the shim and diffs the
result, and that diff is the committed artifact:

    tests/parity/deviations-python.diff

It is the specification of how the two surfaces legitimately differ, reviewed like any
golden, and it shrinks as parity improves.

The property that makes it safe: the file is NON-EMPTY by construction, because the shim
reports a different --version build string and the help renderers wrap differently. So an
empty diff means the shim never ran -- turning the most dangerous failure, a fallthrough
that looks exactly like perfect parity, into the loudest one.

  committed deviations exactly -> parity holds
  empty diff                   -> the shim never ran
  extra hunks                  -> Python drifted; the bug is in the diff
  missing hunks                -> a deviation was fixed; update it, visibly

Review rule: only a --version build string or a help-layout artifact is legitimate. Machine
formats must be byte-identical. Content, ordering, bounds, exit codes, and diagnostics are
contract, not presentation, and a hunk touching them is a bug.
