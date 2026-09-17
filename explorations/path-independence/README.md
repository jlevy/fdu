# Path-Independence Matrix

The first empirical check that an fdu answer does not depend on how it was produced:
which requests warmed the cache, which cache policy was used, which files changed since,
or which surface asked.
It is the evidence behind
[the explicit core models plan](../../docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md).

The scripts have moved into
[`tests/path_independence`](../../tests/path_independence/README.md), where the matrix
runs as a test with a registry of known violations.
Git history holds the original exploration scripts.

## Results

[`results/2026-09-17-fdu-0.1.0-rc-5f2d36d-summary.txt`](results/2026-09-17-fdu-0.1.0-rc-5f2d36d-summary.txt)
is the summary for release candidate `5f2d36d`: metadata-only requests were identical to
cold on every history, policy, mutation, and surface; content-analysis requests were
not. The plan’s Background section interprets it.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
