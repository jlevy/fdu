# Release Candidate QA — 2026-09-25

The signed `v0.1.0` tag names commit `7cf7f1b4b39930ebcf54df7ebd2645d995e4e458`. This
run checks its installed macOS wheel, release artifacts, and file-size answers before
publication. The procedure is the
[installed CLI playbook](../../../tests/qa/cli-installed-e2e.qa.md) and
[release process](../guides/release-process.md).

## Verdict

The final candidate passed `make check`, `make cross-lint`, `make release-rehearse`, and
`make docs-format`. The
[five-platform rehearsal](https://github.com/jlevy/fdu/actions/runs/36203963537) passed
from the same commit.
All eight downloaded files matched its `SHA256SUMS`: two crates, one source
distribution, and five wheels.

The CI-built macOS arm64 wheel was installed with CPython 3.12 and reported `fdu 0.1.0`.
The sequential installed CLI harness recorded 51 checks, with no failures.
Its one warning is the expected exit 2 for the `documents` view without `--analyze`. The
depth-two Library read also returned the documented partial exit 2 for protected folders
and was classified as passing.
Three terminal tests passed against the installed wheel; a live 80-column terminal run
left a clean report and prompt.

A separate temporary fixture gave the same summary under `auto`, `read-only`, `only`,
`refresh`, and `off` cache policies.
It confirmed `.gitignore` selection and equality between the installed Python package
and command line for the summary report.

## Peer Agreement

The peer comparison used a wheel built from the same commit.
Its synthetic self-test matched all 13 reference readings exactly.
On the repository, the Rust toolchain, Applications, and the full Library tree, all 52
readings were explained: 39 matched exactly, 11 fell within measured live-tree movement,
and two were short by the directories their tools reported skipping.
No difference was unexplained.
On the three quiet trees, every top-level allocated total matched GNU `du -l` exactly.
The Library tree returned partial status because protected entries could not be read.
The earlier [peer report](report-2026-09-25-peer-agreement.md) describes the counting
models and the limits of comparisons on a live tree.

## Run Conditions

The harness scanned this repository for the view and analysis matrix, a larger working
tree for bounded metadata runs, and the home Library at limited depth.
The peer comparison also scanned the full Library without content analysis.
Each cache arm had its own cache location, and the harness ran one `fdu` process at a
time. This was an Apple silicon Mac on APFS under normal desktop load.
Timings are single observations, not performance claims.

The terminal playbook’s manual window-resize and color judgments were not repeated.
Its pseudo-terminal tests covered drawing, erasing, nonterminal output, and Ctrl-C.

## Installed CLI Results

| Phase | Name | Verdict | Exit | Real s | RSS MiB | Note |
| --- | --- | --- | ---: | ---: | ---: | --- |
| sanity | help | ok | 0 | 0.070 | 22.8 |  |
| sanity | version | ok | 0 | 0.070 | 22.5 |  |
| sanity | docs | ok | 0 | 0.070 | 22.6 |  |
| sanity | skill | ok | 0 | 0.060 | 22.7 |  |
| views | tree-cold | ok | 0 | 0.230 | 35.1 | cold scan; fresh=0; cached=0 |
| views | tree-warm | ok | 0 | 0.170 | 35.7 | cold scan; fresh=0; cached=0 |
| views | summary | ok | 0 | 0.140 | 36.2 | cold scan; fresh=0; cached=0 |
| views | languages | ok | 0 | 0.180 | 42.2 | cold scan; fresh=0; cached=0 |
| views | families | ok | 0 | 0.200 | 45.6 | cold scan; fresh=0; cached=0 |
| views | types | ok | 0 | 0.420 | 44.6 | cold scan; fresh=0; cached=0 |
| views | extensions | ok | 0 | 0.190 | 36.1 | cold scan; fresh=0; cached=0 |
| views | documents-no-analyze | warn | 2 | 0.070 | 22.9 | empty stdout |
| views | recent | ok | 0 | 0.220 | 44.8 | cold scan; fresh=0; cached=0 |
| views | largest | ok | 0 | 0.290 | 44.5 | cold scan; fresh=0; cached=0 |
| views | files | ok | 0 | 0.250 | 58.0 | cold scan; fresh=0; cached=0 |
| views | full | ok | 0 | 0.230 | 52.6 | cold scan; fresh=0; cached=0 |
| views | combo-kinds | ok | 0 | 0.190 | 42.8 | cold scan; fresh=0; cached=0 |
| views | exclude-ignored-summary | ok | 0 | 0.160 | 37.3 | cold scan; fresh=0; cached=0 |
| views | depth-limit-tree | ok | 0 | 0.180 | 35.0 | cold scan; fresh=0; cached=0 |
| views | scan-depth-1-summary | ok | 0 | 0.080 | 23.7 | cold scan; fresh=0; cached=0 |
| views | json-summary | ok | 0 | 0.150 | 35.5 |  |
| views | yaml-summary | ok | 0 | 0.180 | 35.5 |  |
| cache-analyze | off-code-1 | ok | 0 | 1.710 | 66.2 | cold scan; fresh=21280; cached=0 |
| cache-analyze | off-code-2 | ok | 0 | 0.830 | 64.5 | cold scan; fresh=21280; cached=0 |
| cache-analyze | off-lines-1 | ok | 0 | 0.620 | 64.5 | cold scan; fresh=21280; cached=0 |
| cache-analyze | off-lines-2 | ok | 0 | 0.610 | 64.0 | cold scan; fresh=21280; cached=0 |
| cache-analyze | off-cache-status | ok | 0 | 0.060 | 22.7 |  |
| cache-analyze | on-code-1 | ok | 0 | 0.790 | 76.7 | cold scan; fresh=21280; cached=0 |
| cache-analyze | on-code-2 | ok | 0 | 0.380 | 74.2 | warm revalidation; fresh=0; cached=21280 |
| cache-analyze | on-lines-1 | ok | 0 | 1.360 | 72.4 | warm revalidation; fresh=21280; cached=0 |
| cache-analyze | on-lines-2 | ok | 0 | 0.500 | 76.3 | warm revalidation; fresh=0; cached=21280 |
| cache-analyze | on-cache-status | ok | 0 | 0.220 | 23.0 |  |
| analyze-extra | analyze-words | ok | 0 | 1.720 | 79.3 | warm revalidation; fresh=21280; cached=0 |
| analyze-extra | analyze-all | ok | 0 | 1.890 | 76.2 | warm revalidation; fresh=21280; cached=0 |
| analyze-extra | json-analyze-code | ok | 0 | 1.810 | 68.3 |  |
| analyze-extra | yaml-analyze-lines | ok | 0 | 1.100 | 66.4 |  |
| analyze-extra | text-analyze-code-summary | ok | 0 | 0.840 | 72.9 | warm revalidation; fresh=21280; cached=0 |
| watch | watch-sigint | ok | -2 | 3.010 |  | SIGINT after file create; elapsed=3.01s |
| medium | med-tree-cold | ok | 0 | 16.110 | 728.6 | cold scan; fresh=0; cached=0 |
| medium | med-tree-warm | ok | 0 | 15.940 | 738.3 | cold scan; fresh=0; cached=0 |
| medium | med-summary | ok | 0 | 16.100 | 727.0 | cold scan; fresh=0; cached=0 |
| medium | med-languages | ok | 0 | 16.240 | 1166.8 | cold scan; fresh=0; cached=0 |
| medium | med-combo-kinds | ok | 0 | 16.800 | 1230.5 | cold scan; fresh=0; cached=0 |
| medium | med-recent | ok | 0 | 15.780 | 1233.0 | cold scan; fresh=0; cached=0 |
| medium | med-json-summary | ok | 0 | 15.550 | 723.0 |  |
| medium | med-analyze-code-subdir | ok | 0 | 0.240 | 26.5 | cold scan; fresh=692; cached=0 |
| medium | med-analyze-code-subdir-warm | ok | 0 | 0.070 | 25.5 | warm revalidation; fresh=0; cached=692 |
| large | large-summary-depth1 | ok | 0 | 0.090 | 23.6 | cold scan; fresh=0; cached=0 |
| large | large-summary-depth2 | ok | 2 | 0.230 | 27.5 | cold scan; fresh=0; cached=0 |
| large | large-tree-preferences | ok | 0 | 0.100 | 25.2 | cold scan; fresh=0; cached=0 |
| large | large-tree-logs | ok | 0 | 0.080 | 23.8 | cold scan; fresh=0; cached=0 |

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
