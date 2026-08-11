# Performance Evidence Bundles

Each JSON file is the immutable raw run behind the experiment with the same ID. It
contains the exact variant order, per-job schedule, warmups, paired trial vectors,
invalid-sample reasons, process metrics, oracle observations, aggregate distributions,
and bootstrap comparisons.
The experiment frontmatter records the file’s SHA-256.

The archive step replaces the operator’s tree label and path-derived root ID. It does
not rewrite timings or verdict inputs.
Run:

```shell
uv run --project benchmarks/realtree --frozen python -m benchmarks.realtree archive \
  --run benchmarks/results/realtree/run-example.json \
  --output docs/project/experiments/evidence/exp-NNN-run.json \
  --tree-label reference-tree
```

Files for exp-000 through exp-011 use `fdu-realtree-run-v1`. They preserve the original
raw pairs but are explicitly legacy evidence: v1 did not digest every named
per-directory roll-up, did not identify both source and harness revisions structurally,
and did not hash the expanded schedule.
Former accepted records are therefore marked superseded rather than retroactively
claiming proof the runner did not collect.

Claim-grade records use `fdu-realtree-run-v2`. The experiment schema requires exact
engine and harness commits, harness-source and binary digests, toolchain, target,
release profile, feature set, redacted build command, schedule algorithm and digest,
committed raw-run path and digest, zero oracle-invalid samples in the selected
control/candidate comparison, and the v2 full roll-up oracle before a verdict may be
`accepted`. Diagnostic variants may remain in the same immutable run: exp-012, for
example, preserves the literal PR base’s reducer failures while taking its index
headline only against the correctness-normalized control.

`snapshot-load-wide-scale-v1.json` is a separate, single-candidate topology curve.
It pins the candidate binary and semantic-probe source, generates wide zero-byte corpora
at 10k, 100k, 500k, and 1M entries, and requires both independent v2 digests on every
sample. It measures snapshot loading only; it is not a cold-versus-warm cache-policy
comparison.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
