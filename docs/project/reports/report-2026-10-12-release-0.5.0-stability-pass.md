# 0.5.0 Stability Pass — 2026-10-12

This is the full record of the 0.5.0 release checklist’s stability pass (step 2 of the
[release process](../guides/release-process.md#the-steps)), written by
`make release-stability`. The summaries belong beside the procedures, in the
[QA playbook’s Current Status](../../../tests/qa/cli-installed-e2e.qa.md) and the
[correctness runbook’s Last Recorded Run](../guides/correctness-runbook.md#last-recorded-run);
this report keeps every table they summarize.

## Verdict

The pass tested commit `d0862487939ea653351808d6143b4fe8e1f92351`, tree
`67e3ca0f6475d371cec13465569dadc9bdbe3216`. Merged preparation commit
`33a3a890cefb4848d3b609925e9682e483246b46` and commit checked again after merging main
`6117ae41108311f3650dc165abd7424ca2cf7277` have that same tree.
These comparisons identify the tested source; they do not clear failed peer QA.

**Tagging is blocked.** The required wider peer pass failed on the live home Library in
both runs. `fdu-wo7c` needs diagnosis and maintainer disposition; the separate Linux
summary attribution in `fdu-4arl` also needs a pre-tag decision.
Neither has been waived.
All original 15 automated driver stages passed, but the peer stage covered only two
subjects and did not meet the required subject coverage.
The five-subject reruns supersede that narrower peer verdict: the first had four failing
checks and the repeat had two.
The table below and full peer tables record the repeat.

| Gate or phase | Verdict |
| --- | --- |
| `make check` | Passed: `make check` exited 0 |
| `make cross-lint` | Passed: `make cross-lint` exited 0: linted x86_64-apple-darwin, x86_64-pc-windows-msvc, i686-unknown-linux-gnu, x86_64-unknown-linux-musl, aarch64-unknown-linux-gnu |
| `make semver-check` | Passed: `make semver-check` exited 0: fdu-core 0.5.0: starts a new compatibility series; nothing to check; fdu 0.5.0: starts a new compatibility series; nothing to check |
| `make release-rehearse` | Passed: `make release-rehearse` exited 0 |
| Candidate | Passed: `fdu --version` printed `fdu 0.5.0-dev+gd08624879`, from `fdu-0.5.0-cp312-abi3-macosx_11_0_arm64.whl` |
| QA harness, phases 1–5 and 8 | Passed: 49 checks: 49 ok, 0 warn, 0 fail |
| QA phase 6, terminal progress | Pending: a person has not watched a real window, and Windows progress QA has not run. Terminal tests passed: 3 tests ran against the candidate; exit status 0; pty probe passed: 38 of 38 checks passed |
| QA phase 7, peer agreement self-test | Passed: 13 readings agree exactly; 0 failed; exit status 0 |
| QA phase 7, peer agreement on real trees | Failed: 52 of 65 readings on 5 trees agree exactly, 2 `UNEXPLAINED`; 2 readings failed; exit status 1 |
| Correctness, refusal tree as an unprivileged user | Passed: 26 of 26 cases partial and withheld; 0 answer mismatches, 0 mechanism failures, 0 stale reference instants; exit status 0 |
| Correctness, served tree | Passed: 26 of 26 served `cache_only` and labelled `stale`; 0 answer mismatches, 0 mechanism failures, 0 stale reference instants; exit status 0 |
| Correctness, cross-warm matrix | Passed: 72 of 72 pairs matched the cold answer; 0 violations; exit status 0 |
| Correctness, break: no snapshot stored | Passed: `warm_cold.py` exited 1, 26 of 26 cases caught (17 `NO-SNAPSHOT`, 9 `NOT-WARM`); `cross_warm.py` exited 1, 17 of 17 same-analyzer pairs `NOT-WARM` |
| Correctness, break: partial answer stored | Passed: `warm_cold.py --refusals-only` exited 1, 26 of 26 cases `PARTIAL-STORED` |

## Peer Failure and Release Decisions

Both five-subject runs covered the checkout, Rust toolchains, Applications, home
Library, and Linux v6.12. On the four quieter trees, all 52 readings agreed exactly in
each run. Library remained failed under the unchanged comparison rules.
An exit-2 partial result is expected under macOS permissions; an unexplained peer
difference or non-permission error still fails the peer check.

- **First wider run.** GNU du -l’s allocated total and top-level Application Support
  comparison left 6,205,440 bytes unexplained.
  BSD du had a smaller allocated discrepancy, and one fdu observation reported
  `Interrupted`. The driver recorded three `UNEXPLAINED` notes and four failing checks,
  exiting 1.
- **Allocation probe.** Two focused Application Support sequences bracketed GNU du -l
  with fdu reads. Neither retained a stable allocated mismatch; this narrows the first
  observation but does not establish its cause.
- **Repeat.** The earlier GNU du, BSD du, and top-level discrepancies cleared.
  After the measured directory and symbolic-link correction, pdu’s apparent total
  remained 29,232 bytes above the following fdu reading, outside the existing movement
  rule. One `Interrupted` observation recurred.
  The driver recorded two `UNEXPLAINED` notes and two failing checks, exiting 1.
- **Refused paths.** In each interrupted observation, fdu detailed the same 64 refused
  paths as in its reads before and after it.
  There were 511 total refusals and 447 omitted details.
  The transient kind occurred on an already refused path; this does not explain that
  diagnostic or the size discrepancy under the QA rules.
- **Published-version probe.** Four alternating reads of published 0.4.0 and four of
  candidate 0.5.0 all exited 2 with the same 511-refusal count.
  Published 0.4.0 produced one `provider_failure`, OS error 4 (`Interrupted`). That same
  path produced `permission`, OS error 1, in the other seven observations, including all
  four candidate reads.
  The symptom therefore also occurs in the published version.
  This does not attribute the pdu discrepancy or turn peer QA green.

Raw JSON and logs, including private paths and application identities, remain in the
local evidence archive.
Published tables use role labels and counts.
`fdu-wo7c` remains open for the Library diagnosis and maintainer decision; no thresholds
or code were changed to clear these observations.

Separately, `fdu-4arl` retains the Linux dense-tree summary attribution and pre-tag
non-inferiority decision.
The approved demo recording establishes its visual review; it supplies no quiet-host
performance result.
Human observation of Phase 6 in a real window and Windows progress QA
remain pending.

## Integration and Rehearsal Status

`make check` passed again at `6117ae41108311f3650dc165abd7424ca2cf7277`. All 20 jobs in
[merged-main CI run 38187381778](https://github.com/jlevy/fdu/actions/runs/38187381778)
passed before this evidence-only update.
The maintainer’s `make release-body` and `make release-preflight` also passed on merged
preparation commit `33a3a890cefb4848d3b609925e9682e483246b46`.

The later GitHub
[release rehearsal, run 38187708393](https://github.com/jlevy/fdu/actions/runs/38187708393)
passed on that merged commit.
Five platform wheels, one sdist, and two crates were downloaded: all eight files passed
SHA-256 verification and artifact inspection.
Publishing and announcement were skipped.
The table’s passing `make release-rehearse` entry describes the earlier local driver
stage. Successful integration and packaging checks do not disposition either pre-tag
finding.

## Regime

- **Host.** Bare metal: MacBookPro18,1, 10 physical and 10 logical CPUs, 32 GiB of
  memory, macOS 26.5.2 (Darwin arm64, kernel 25.5.0; no hypervisor reported), running as
  an unprivileged user.
  Other tasks and application activity continued.
  The host snapshot recorded load averages 11.60, 9.30, and 8.60; these timings are
  observations on a loaded host, not quiet-host benchmark evidence.
- **Storage.** Correctness fixtures, build targets, and task caches were on external
  APFS scratch. The host snapshot recorded 20,516,245,504 bytes free there.
  Correctness cases used fresh cache directories.
- **Declared preconditions.** none.
- **Candidate.** The `fdu-0.5.0-cp312-abi3-macosx_11_0_arm64.whl` wheel built from the
  commit, installed with `uv tool install` into an isolated tool directory;
  `fdu --version` printed `fdu 0.5.0-dev+gd08624879`.
- **Peers.** du (GNU coreutils) 9.9, Dust 1.2.4, pdu 0.24.0, dua 2.41.1, diskus 0.9.0.
- **Trees.** small: `fdu-checkout`; medium: `linux-v6.12`; medium analyze:
  `linux-documentation`; large: `hostile-application-support`; progress:
  `github-workspaces`; progress analyze: `linux-v6.12`; peer 1: `fdu-checkout`; peer 2:
  `rust-toolchains`; peer 3: `applications`; peer 4: `home-library`; peer 5:
  `linux-v6.12`.
- **Correctness trees.** tree: 14 kinds present, 3 absent (chardev, blockdev,
  non-utf8-name); served: 13 kinds present, 3 absent (chardev, blockdev, non-utf8-name).
- **Tooling.** The runs used more than one revision of the scripts: `d08624879` for
  check, cross-lint, semver-check, release-rehearse, candidate, harness, peer-self-test,
  pty-probe, terminal, refusals, served, cross-warm, break-no-snapshot,
  break-partial-stored; `6117ae411` for peer-trees.

## Reproduce

Replace the role paths with existing directories; keep scratch short enough for the
macOS Unix socket fixture, and keep build and cache paths on the intended volume.

```shell
export COMMIT=d0862487939ea653351808d6143b4fe8e1f92351
export RELEASE="/path/to/release-work"
export TMPDIR="/path/to/short-scratch"
export CARGO_TARGET_DIR="$RELEASE/target"
export UV_CACHE_DIR="$RELEASE/uv-cache"
export FDU_QA_SMALL=/path/to/fdu-checkout
export FDU_QA_MEDIUM=/path/to/linux-v6.12
export FDU_QA_MEDIUM_ANALYZE=/path/to/linux-documentation
export FDU_QA_LARGE=/path/to/hostile-application-support
export FDU_QA_PROGRESS_TREE=/path/to/github-workspaces
export FDU_QA_PROGRESS_ANALYZE=/path/to/linux-v6.12
export FDU_QA_PEER_TREES=/path/to/fdu-checkout:/path/to/rust-toolchains:/path/to/applications:/path/to/home-library:/path/to/linux-v6.12
make release-stability
```

## Full Tables

Paths are replaced by the labels above.

### Installed-CLI harness (fdu 0.5.0-dev+gd08624879)

| Phase | Name | Verdict | Exit | Real s | RSS MiB | Note |
| --- | --- | --- | ---: | ---: | ---: | --- |
| sanity | help | ok | 0 | 0.020 | 17.6 |  |
| sanity | version | ok | 0 | 0.020 | 17.2 |  |
| sanity | docs | ok | 0 | 0.020 | 17.4 |  |
| sanity | skill | ok | 0 | 0.020 | 17.2 |  |
| views | tree-cold | ok | 0 | 0.030 | 19.5 | cold scan; fresh=0; cached=0 |
| views | tree-warm | ok | 0 | 0.020 | 19.6 | cold scan; fresh=0; cached=0 |
| views | summary | ok | 0 | 0.020 | 19.0 | cold scan; fresh=0; cached=0 |
| views | languages | ok | 0 | 0.020 | 20.4 | cold scan; fresh=0; cached=0 |
| views | families | ok | 0 | 0.030 | 20.4 | cold scan; fresh=0; cached=0 |
| views | types | ok | 0 | 0.020 | 20.4 | cold scan; fresh=0; cached=0 |
| views | extensions | ok | 0 | 0.020 | 20.0 | cold scan; fresh=0; cached=0 |
| views | documents | ok | 0 | 0.210 | 33.7 | cold scan; fresh=1361; cached=0 |
| views | recent | ok | 0 | 0.020 | 20.6 | cold scan; fresh=0; cached=0 |
| views | largest | ok | 0 | 0.020 | 20.4 | cold scan; fresh=0; cached=0 |
| views | files | ok | 0 | 0.030 | 21.4 | cold scan; fresh=0; cached=0 |
| views | full | ok | 0 | 0.020 | 21.6 | cold scan; fresh=0; cached=0 |
| views | combo-kinds | ok | 0 | 0.020 | 20.6 | cold scan; fresh=0; cached=0 |
| views | exclude-ignored-summary | ok | 0 | 0.040 | 21.5 | cold scan; fresh=0; cached=0 |
| views | depth-limit-tree | ok | 0 | 0.020 | 19.6 | cold scan; fresh=0; cached=0 |
| views | scan-depth-1-summary | ok | 0 | 0.020 | 18.4 | cold scan; fresh=0; cached=0 |
| views | json-summary | ok | 0 | 0.020 | 19.1 |  |
| views | yaml-summary | ok | 0 | 0.020 | 19.1 |  |
| cache-analyze | off-code-1 | ok | 0 | 0.070 | 24.0 | cold scan; fresh=1361; cached=0 |
| cache-analyze | off-code-2 | ok | 0 | 0.060 | 24.1 | cold scan; fresh=1361; cached=0 |
| cache-analyze | off-lines-1 | ok | 0 | 0.050 | 23.8 | cold scan; fresh=1361; cached=0 |
| cache-analyze | off-lines-2 | ok | 0 | 0.050 | 23.5 | cold scan; fresh=1361; cached=0 |
| cache-analyze | off-cache-status | ok | 0 | 0.020 | 17.6 |  |
| cache-analyze | on-code-1 | ok | 0 | 0.080 | 23.9 | cold scan; fresh=1361; cached=0 |
| cache-analyze | on-code-2 | ok | 0 | 0.030 | 22.6 | warm revalidation; fresh=0; cached=1361 |
| cache-analyze | on-lines-1 | ok | 0 | 0.060 | 24.9 | warm revalidation; fresh=1361; cached=0 |
| cache-analyze | on-lines-2 | ok | 0 | 0.030 | 22.4 | warm revalidation; fresh=0; cached=1361 |
| cache-analyze | on-cache-status | ok | 0 | 0.020 | 17.8 |  |
| analyze-extra | analyze-words | ok | 0 | 0.070 | 33.2 | warm revalidation; fresh=1361; cached=0 |
| analyze-extra | analyze-all | ok | 0 | 0.080 | 32.6 | warm revalidation; fresh=1361; cached=0 |
| analyze-extra | json-analyze-code | ok | 0 | 0.080 | 24.2 | physical_lines=212988 |
| analyze-extra | yaml-analyze-lines | ok | 0 | 0.060 | 24.0 |  |
| analyze-extra | text-analyze-code-summary | ok | 0 | 0.070 | 24.3 | warm revalidation; fresh=1361; cached=0 |
| watch | watch-sigint | ok | -2 | 3.021 |  | SIGINT after file create; elapsed=3.02s |
| medium | med-tree-cold | ok | 0 | 0.390 | 24.2 | cold scan; fresh=0; cached=0 |
| medium | med-tree-warm | ok | 0 | 0.240 | 24.5 | cold scan; fresh=0; cached=0 |
| medium | med-summary | ok | 0 | 0.240 | 22.6 | cold scan; fresh=0; cached=0 |
| medium | med-languages | ok | 0 | 0.300 | 67.1 | cold scan; fresh=0; cached=0 |
| medium | med-combo-kinds | ok | 0 | 0.320 | 67.7 | cold scan; fresh=0; cached=0 |
| medium | med-recent | ok | 0 | 0.320 | 75.8 | cold scan; fresh=0; cached=0 |
| medium | med-json-summary | ok | 0 | 0.240 | 22.7 |  |
| medium | med-analyze-code-subdir | ok | 0 | 0.760 | 40.9 | cold scan; fresh=11438; cached=0 |
| medium | med-analyze-code-subdir-warm | ok | 0 | 0.080 | 41.2 | warm revalidation; fresh=0; cached=11438 |
| large | large-summary-depth1 | ok | 0 | 0.060 | 18.5 | cold scan; fresh=0; cached=0 |
| large | large-summary-depth2 | ok | 2 | 0.050 | 19.6 | cold scan; fresh=0; cached=0 |

### Peer agreement self-test

#### `<scratch>/probe root`

fdu read 40.0 KiB allocated and 1.0 GiB apparent, exiting 2; across its 14 readings, one
before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: 1 denied.

Measured from the tree: 1 hard-linked inodes, which per-path counting adds 24.0 KiB
allocated to; 4 symbolic links, 1.0 KiB apparent; 5 directories, 768 B apparent;
directories that could not be listed: 1 denied; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 40.0 KiB | 0 B | 0 B | — | agrees exactly | 1 denied | 0.0 s |
| GNU du | allocated | 16.0 KiB | -24.0 KiB | -24.0 KiB | hard links once -24.0 KiB | agrees exactly | 1 denied | 0.0 s |
| GNU du -l | apparent | 1.0 GiB | +1.0 KiB | +1.0 KiB | symbolic links +1.0 KiB | agrees exactly | 1 denied | 0.0 s |
| GNU du | apparent | 1.0 GiB | -18.5 KiB | -18.5 KiB | hard links once -19.5 KiB, symbolic links +1.0 KiB | agrees exactly | 1 denied | 0.0 s |
| dust | allocated | 16.0 KiB | -24.0 KiB | -24.0 KiB | hard links once -24.0 KiB | agrees exactly | 1 denied | 0.0 s |
| dust | apparent | 1.0 GiB | +1.8 KiB | +1.8 KiB | symbolic links +1.0 KiB, directories +768 B | agrees exactly | 1 denied | 0.0 s |
| pdu | allocated | 40.0 KiB | 0 B | 0 B | — | agrees exactly | 1 denied | 0.1 s |
| pdu | apparent | 1.0 GiB | +1.8 KiB | +1.8 KiB | symbolic links +1.0 KiB, directories +768 B | agrees exactly | 1 denied | 0.1 s |
| dua | allocated | 16.0 KiB | -24.0 KiB | -24.0 KiB | hard links once -24.0 KiB | agrees exactly | 1 unreadable, reason not given | 0.0 s |
| dua | apparent | 1.0 GiB | -19.1 KiB | -19.1 KiB | hard links once -19.5 KiB, symbolic links +37 B, directories +448 B | agrees exactly | 1 unreadable, reason not given | 0.0 s |
| diskus | allocated | 16.0 KiB | -24.0 KiB | -24.0 KiB | hard links once -24.0 KiB | agrees exactly | 1 unreadable, reason not given | 0.0 s |
| diskus | apparent | 1.0 GiB | -18.5 KiB | -18.5 KiB | hard links once -19.5 KiB, symbolic links +1.0 KiB | agrees exactly | 1 unreadable, reason not given | 0.0 s |
| BSD du | allocated | 16.0 KiB | -24.0 KiB | -24.0 KiB | hard links once -24.0 KiB | agrees exactly | 1 denied | 0.0 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
4\.

Self-test: 13 readings agree exactly; 0 failed.

### Peer agreement on real trees

#### `fdu-checkout`

fdu read 132.5 MiB allocated and 121.7 MiB apparent, exiting 0; across its 14 readings,
one before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 12 symbolic links, 288 B apparent; 551 directories, 189.6 KiB apparent; directories
that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 132.5 MiB | 0 B | 0 B | — | agrees exactly | none | 0.0 s |
| GNU du | allocated | 132.5 MiB | 0 B | 0 B | — | agrees exactly | none | 0.0 s |
| GNU du -l | apparent | 121.7 MiB | +288 B | +288 B | symbolic links +288 B | agrees exactly | none | 0.0 s |
| GNU du | apparent | 121.7 MiB | +288 B | +288 B | symbolic links +288 B | agrees exactly | none | 0.0 s |
| dust | allocated | 132.5 MiB | 0 B | 0 B | — | agrees exactly | none | 0.0 s |
| dust | apparent | 121.9 MiB | +189.9 KiB | +189.9 KiB | symbolic links +288 B, directories +189.6 KiB | agrees exactly | none | 0.0 s |
| pdu | allocated | 132.5 MiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| pdu | apparent | 121.9 MiB | +189.9 KiB | +189.9 KiB | symbolic links +288 B, directories +189.6 KiB | agrees exactly | none | 0.1 s |
| dua | allocated | 132.5 MiB | 0 B | 0 B | — | agrees exactly | none | 0.0 s |
| dua | apparent | 121.9 MiB | +188.8 KiB | +188.8 KiB | symbolic links +288 B, directories +188.5 KiB | agrees exactly | none | 0.0 s |
| diskus | allocated | 132.5 MiB | 0 B | 0 B | — | agrees exactly | none | 0.0 s |
| diskus | apparent | 121.7 MiB | +288 B | +288 B | symbolic links +288 B | agrees exactly | none | 0.0 s |
| BSD du | allocated | 132.5 MiB | 0 B | 0 B | — | agrees exactly | none | 0.0 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
12\.

#### `rust-toolchains`

fdu read 3.7 GiB allocated and 3.5 GiB apparent, exiting 0; across its 14 readings, one
before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 0 symbolic links, 0 B apparent; 3,427 directories, 2.6 MiB apparent; directories
that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 3.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.3 s |
| GNU du | allocated | 3.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.3 s |
| GNU du -l | apparent | 3.5 GiB | 0 B | 0 B | — | agrees exactly | none | 0.3 s |
| GNU du | apparent | 3.5 GiB | 0 B | 0 B | — | agrees exactly | none | 0.3 s |
| dust | allocated | 3.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |
| dust | apparent | 3.5 GiB | +2.6 MiB | +2.6 MiB | directories +2.6 MiB | agrees exactly | none | 0.1 s |
| pdu | allocated | 3.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |
| pdu | apparent | 3.5 GiB | +2.6 MiB | +2.6 MiB | directories +2.6 MiB | agrees exactly | none | 0.2 s |
| dua | allocated | 3.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |
| dua | apparent | 3.5 GiB | +2.6 MiB | +2.6 MiB | directories +2.6 MiB | agrees exactly | none | 0.2 s |
| diskus | allocated | 3.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| diskus | apparent | 3.5 GiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| BSD du | allocated | 3.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
4\.

#### `applications`

fdu read 43.9 GiB allocated and 48.8 GiB apparent, exiting 0; across its 14 readings,
one before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 16,049 symbolic links, 308.5 KiB apparent; 83,222 directories, 23.8 MiB apparent;
directories that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 10.4 s |
| GNU du | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 11.6 s |
| GNU du -l | apparent | 48.8 GiB | +308.5 KiB | +308.5 KiB | symbolic links +308.5 KiB | agrees exactly | none | 11.5 s |
| GNU du | apparent | 48.8 GiB | +308.5 KiB | +308.5 KiB | symbolic links +308.5 KiB | agrees exactly | none | 11.5 s |
| dust | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 3.5 s |
| dust | apparent | 48.9 GiB | +24.1 MiB | +24.1 MiB | symbolic links +308.5 KiB, directories +23.8 MiB | agrees exactly | none | 3.5 s |
| pdu | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 4.1 s |
| pdu | apparent | 48.9 GiB | +24.1 MiB | +24.1 MiB | symbolic links +308.5 KiB, directories +23.8 MiB | agrees exactly | none | 3.5 s |
| dua | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 3.7 s |
| dua | apparent | 48.9 GiB | +24.1 MiB | +24.1 MiB | symbolic links +308.5 KiB, directories +23.8 MiB | agrees exactly | none | 3.8 s |
| diskus | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 3.3 s |
| diskus | apparent | 48.8 GiB | +308.5 KiB | +308.5 KiB | symbolic links +308.5 KiB | agrees exactly | none | 3.1 s |
| BSD du | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 5.6 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
164\.

#### `home-library`

fdu read 68.9 GiB allocated and 144.9 GiB apparent, exiting 2; across its 14 readings,
one before each tool and one at the end, the tree moved 688.0 KiB allocated and 731.8
KiB apparent. fdu errors: 64 denied, 447 more, not detailed.

Measured from the tree: 119 hard-linked inodes, which per-path counting adds 22.6 MiB
allocated to; 26,270 symbolic links, 824.6 KiB apparent; 73,520 directories, 27.5 MiB
apparent; directories that could not be listed: 507 denied; entries that could not be
stat’d: 4.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 68.9 GiB | +484.0 KiB | 0 B | — | agrees within fdu’s nearby movement (324.0 KiB) | 511 denied | 40.6 s |
| GNU du | allocated | 68.8 GiB | -22.7 MiB | -22.6 MiB | hard links once -22.6 MiB | agrees within the tree’s movement | 511 denied | 28.2 s |
| GNU du -l | apparent | 144.9 GiB | +844.0 KiB | +824.6 KiB | symbolic links +824.6 KiB | agrees within the tree’s movement | 511 denied | 25.6 s |
| GNU du | apparent | 144.9 GiB | -21.2 MiB | -21.5 MiB | hard links once -22.3 MiB, symbolic links +824.6 KiB | agrees within the tree’s movement | 511 denied | 23.6 s |
| dust | allocated | 68.8 GiB | -22.6 MiB | -22.6 MiB | hard links once -22.6 MiB | agrees within the tree’s movement | 507 denied | 4.5 s |
| dust | apparent | 145.0 GiB | +28.3 MiB | +28.3 MiB | symbolic links +824.6 KiB, directories +27.5 MiB | agrees within fdu’s nearby movement (29.8 KiB) | 507 denied | 5.0 s |
| pdu | allocated | 68.9 GiB | +12.0 KiB | 0 B | — | agrees within the tree’s movement | 511 denied | 4.8 s |
| pdu | apparent | 145.0 GiB | +28.3 MiB | +28.3 MiB | symbolic links +824.6 KiB, directories +27.5 MiB | UNEXPLAINED | 511 denied | 13.8 s |
| dua | allocated | 68.8 GiB | -22.6 MiB | -22.6 MiB | hard links once -22.6 MiB | agrees within fdu’s nearby movement (36.0 KiB) | 511 unreadable, reason not given | 5.1 s |
| dua | apparent | 144.9 GiB | +6.0 MiB | +6.0 MiB | hard links once -22.3 MiB, symbolic links +824.6 KiB, directories +27.5 MiB | agrees within fdu’s nearby movement (135.8 KiB) | 511 unreadable, reason not given | 5.1 s |
| diskus | allocated | 68.8 GiB | -22.8 MiB | -22.6 MiB | hard links once -22.6 MiB | agrees within fdu’s nearby movement (164.0 KiB) | 512 unreadable, reason not given | 10.0 s |
| diskus | apparent | 144.9 GiB | -21.3 MiB | -21.5 MiB | hard links once -22.3 MiB, symbolic links +824.6 KiB | agrees within fdu’s nearby movement (169.9 KiB) | 511 unreadable, reason not given | 6.7 s |
| BSD du | allocated | 68.8 GiB | -22.6 MiB | -22.6 MiB | hard links once -22.6 MiB | agrees within the tree’s movement | 511 denied | 18.3 s |

- UNEXPLAINED: fdu reported 1 interrupted errors.

Directories a tool reported it could not read and the walk could list:

- diskus (allocated), holding 0 B: one group-container subtree.
  Its application identity is omitted; the full path remains in the private raw
  evidence.

Top-level directories whose allocated size does not agree with GNU du -l within fdu’s
readings around it, ± 324.0 KiB: 0 of 149.

#### `linux-v6.12`

fdu read 1.7 GiB allocated and 1.5 GiB apparent, exiting 0; across its 14 readings, one
before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 102 symbolic links, 3.1 KiB apparent; 6,282 directories, 3.5 MiB apparent;
directories that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 1.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.4 s |
| GNU du | allocated | 1.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.4 s |
| GNU du -l | apparent | 1.5 GiB | +3.1 KiB | +3.1 KiB | symbolic links +3.1 KiB | agrees exactly | none | 0.4 s |
| GNU du | apparent | 1.5 GiB | +3.1 KiB | +3.1 KiB | symbolic links +3.1 KiB | agrees exactly | none | 0.4 s |
| dust | allocated | 1.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |
| dust | apparent | 1.5 GiB | +3.5 MiB | +3.5 MiB | symbolic links +3.1 KiB, directories +3.5 MiB | agrees exactly | none | 0.2 s |
| pdu | allocated | 1.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |
| pdu | apparent | 1.5 GiB | +3.5 MiB | +3.5 MiB | symbolic links +3.1 KiB, directories +3.5 MiB | agrees exactly | none | 0.2 s |
| dua | allocated | 1.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |
| dua | apparent | 1.5 GiB | +3.5 MiB | +3.5 MiB | symbolic links +3.1 KiB, directories +3.5 MiB | agrees exactly | none | 0.1 s |
| diskus | allocated | 1.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |
| diskus | apparent | 1.5 GiB | +3.1 KiB | +3.1 KiB | symbolic links +3.1 KiB | agrees exactly | none | 0.1 s |
| BSD du | allocated | 1.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.3 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
24\.

2 readings failed.

### Phase 6 pty probe

```
fdu: <work>/bin/fdu
slow tree: github-workspaces
small tree: fdu-checkout
analyze tree: linux-v6.12

ok   slow scan exits 0 (2.9 s)
ok   slow scan draws frames (23 frames)
ok   first frame after about half a second (0.52 s)
ok   a frame says Scanning
     phases seen: ['Scanning']
ok   the last frame is erased before the report
ok   the report follows the erase, with no frame left in it
ok   every frame fits 100 columns (at most 99)
ok   no frame contains a newline
ok   file counts climb ([86655] to [575353])
ok   the counts hold their column while Scanning ([40])
     sample frame: ⠙ <progress-tree>  Scanning        356,478 files ·    48,100 dirs ·   15 GiB · 1.7 s
ok   a small tree shows no indicator
ok   redirected stderr holds no carriage return or escape
ok   redirected stderr holds only note/warn/tip/perf lines (4 lines)
ok   nothing drawn on the terminal when stderr is a file
ok   --format json shows nothing
ok   --progress always --format json draws and erases
ok   --progress never shows nothing (text)
ok   --progress never shows nothing (json)
ok   --progress never shows nothing (yaml)
ok   CI=1 shows nothing
ok   TERM=dumb shows nothing
ok   NO_COLOR=1 keeps the animation
ok   NO_COLOR=1 frames carry no color
ok   without NO_COLOR the frames are colored
ok   Ctrl-C: dies by SIGINT (status 2; shell $? would be 130)
ok   Ctrl-C: prints fdu: interrupted
ok   Ctrl-C: the line is erased before the message
ok   Ctrl-C: no frame after the message
ok   60 columns: 57 frames fit, widest 59 ('⠦ <analyze-tree>  Summarizing  95,938 files · 1.7 GiB · 6.5 s')
ok   40 columns: 47 frames fit, widest 13 ('⠦ Summarizing')
ok   30 columns: 47 frames fit, widest 13 ('⠦ Summarizing')
ok   20 columns: 47 frames fit, widest 13 ('⠴ Summarizing')
ok   19 columns: 47 frames fit, spinner and phase word only ('⠦ Summarizing')
ok   12 columns: 46 frames fit, spinner and phase word only ('⠋ Analyzing')
ok   narrowing mid-run shrinks every later frame without wrapping
ok   --analyze all shows Analyzing with a percentage ([8] to [99])
ok   the percentage climbs
ok   the analyze indicator is erased before the report

38 of 38 checks passed
```

### Terminal tests

```
...
----------------------------------------------------------------------
Ran 3 tests in 67.108s

OK
```

### Correctness: refusals

```
case                     cold   warm   only  warm source      only source  fresh   verdict
--------------------------------------------------------------------------------------------------------
default                     2      2      1  cold_scan        -            -       withheld
summary                     2      2      1  cold_scan        -            -       withheld
tree                        2      2      1  cold_scan        -            -       withheld
types                       2      2      1  cold_scan        -            -       withheld
extensions                  2      2      1  cold_scan        -            -       withheld
largest                     2      2      1  cold_scan        -            -       withheld
files                       2      2      1  cold_scan        -            -       withheld
size-apparent               2      2      1  cold_scan        -            -       withheld
size-allocated              2      2      1  cold_scan        -            -       withheld
sort-mtime                  2      2      1  cold_scan        -            -       withheld
sort-name                   2      2      1  cold_scan        -            -       withheld
depth-1                     2      2      1  cold_scan        -            -       withheld
scan-depth-2                2      2      1  cold_scan        -            -       withheld
min-size                    2      2      1  cold_scan        -            -       withheld
no-gitignore                2      2      1  cold_scan        -            -       withheld
exclude-ignored             2      2      1  cold_scan        -            -       withheld
one-filesystem              2      2      1  cold_scan        -            -       withheld
analyze-lines               2      2      1  cold_scan        -            -       withheld
analyze-code                2      2      1  cold_scan        -            -       withheld
analyze-words               2      2      1  cold_scan        -            -       withheld
analyze-all                 2      2      1  cold_scan        -            -       withheld
languages                   2      2      1  cold_scan        -            -       withheld
documents                   2      2      1  cold_scan        -            -       withheld
view-code                   2      2      1  cold_scan        -            -       withheld
view-documents              2      2      1  cold_scan        -            -       withheld
view-code-documents         2      2      1  cold_scan        -            -       withheld

answer mismatches: 0
mechanism failures (cache did not serve): 0
stale reference instants: 0
cases the snapshot served: 0 of 26
```

### Correctness: served

```
case                     cold   warm   only  warm source      only source  fresh   verdict
--------------------------------------------------------------------------------------------------------
default                     0      0      0  cold_scan        cache_only   stale   ok
summary                     0      0      0  cold_scan        cache_only   stale   ok
tree                        0      0      0  cold_scan        cache_only   stale   ok
types                       0      0      0  cold_scan        cache_only   stale   ok
extensions                  0      0      0  cold_scan        cache_only   stale   ok
largest                     0      0      0  cold_scan        cache_only   stale   ok
files                       0      0      0  cold_scan        cache_only   stale   ok
size-apparent               0      0      0  cold_scan        cache_only   stale   ok
size-allocated              0      0      0  cold_scan        cache_only   stale   ok
sort-mtime                  0      0      0  cold_scan        cache_only   stale   ok
sort-name                   0      0      0  cold_scan        cache_only   stale   ok
depth-1                     0      0      0  cold_scan        cache_only   stale   ok
scan-depth-2                0      0      0  cold_scan        cache_only   stale   ok
min-size                    0      0      0  cold_scan        cache_only   stale   ok
no-gitignore                0      0      0  cold_scan        cache_only   stale   ok
exclude-ignored             0      0      0  cold_scan        cache_only   stale   ok
one-filesystem              0      0      0  cold_scan        cache_only   stale   ok
analyze-lines               0      0      0  warm_revalidate  cache_only   stale   ok
analyze-code                0      0      0  warm_revalidate  cache_only   stale   ok
analyze-words               0      0      0  warm_revalidate  cache_only   stale   ok
analyze-all                 0      0      0  warm_revalidate  cache_only   stale   ok
languages                   0      0      0  warm_revalidate  cache_only   stale   ok
documents                   0      0      0  warm_revalidate  cache_only   stale   ok
view-code                   0      0      0  warm_revalidate  cache_only   stale   ok
view-documents              0      0      0  warm_revalidate  cache_only   stale   ok
view-code-documents         0      0      0  warm_revalidate  cache_only   stale   ok

answer mismatches: 0
mechanism failures (cache did not serve): 0
stale reference instants: 0
cases the snapshot served: 26 of 26
```

### Correctness: cross-warm

```
warmer           ask                   rc  analysis.analyze           verdict
------------------------------------------------------------------------------------------
W_none           a_lines                0  ['lines']                  ok
W_none           a_code                 0  ['lines', 'code']          ok
W_none           a_words                0  ['lines', 'words']         ok
W_none           a_all                  0  ['lines', 'code', 'words'] ok
W_none           a_lines_documents      0  ['lines', 'words']         ok
W_none           a_code_languages       0  ['lines', 'code']          ok
W_none           a_view_code            0  ['lines', 'code']          ok
W_none           a_view_documents       0  ['lines', 'words']         ok
W_none           a_view_both            0  ['lines', 'code', 'words'] ok
W_lines          a_lines                0  ['lines']                  ok
W_lines          a_code                 0  ['lines', 'code']          ok
W_lines          a_words                0  ['lines', 'words']         ok
W_lines          a_all                  0  ['lines', 'code', 'words'] ok
W_lines          a_lines_documents      0  ['lines', 'words']         ok
W_lines          a_code_languages       0  ['lines', 'code']          ok
W_lines          a_view_code            0  ['lines', 'code']          ok
W_lines          a_view_documents       0  ['lines', 'words']         ok
W_lines          a_view_both            0  ['lines', 'code', 'words'] ok
W_code           a_lines                0  ['lines']                  ok
W_code           a_code                 0  ['lines', 'code']          ok
W_code           a_words                0  ['lines', 'words']         ok
W_code           a_all                  0  ['lines', 'code', 'words'] ok
W_code           a_lines_documents      0  ['lines', 'words']         ok
W_code           a_code_languages       0  ['lines', 'code']          ok
W_code           a_view_code            0  ['lines', 'code']          ok
W_code           a_view_documents       0  ['lines', 'words']         ok
W_code           a_view_both            0  ['lines', 'code', 'words'] ok
W_words          a_lines                0  ['lines']                  ok
W_words          a_code                 0  ['lines', 'code']          ok
W_words          a_words                0  ['lines', 'words']         ok
W_words          a_all                  0  ['lines', 'code', 'words'] ok
W_words          a_lines_documents      0  ['lines', 'words']         ok
W_words          a_code_languages       0  ['lines', 'code']          ok
W_words          a_view_code            0  ['lines', 'code']          ok
W_words          a_view_documents       0  ['lines', 'words']         ok
W_words          a_view_both            0  ['lines', 'code', 'words'] ok
W_all            a_lines                0  ['lines']                  ok
W_all            a_code                 0  ['lines', 'code']          ok
W_all            a_words                0  ['lines', 'words']         ok
W_all            a_all                  0  ['lines', 'code', 'words'] ok
W_all            a_lines_documents      0  ['lines', 'words']         ok
W_all            a_code_languages       0  ['lines', 'code']          ok
W_all            a_view_code            0  ['lines', 'code']          ok
W_all            a_view_documents       0  ['lines', 'words']         ok
W_all            a_view_both            0  ['lines', 'code', 'words'] ok
W_view_code      a_lines                0  ['lines']                  ok
W_view_code      a_code                 0  ['lines', 'code']          ok
W_view_code      a_words                0  ['lines', 'words']         ok
W_view_code      a_all                  0  ['lines', 'code', 'words'] ok
W_view_code      a_lines_documents      0  ['lines', 'words']         ok
W_view_code      a_code_languages       0  ['lines', 'code']          ok
W_view_code      a_view_code            0  ['lines', 'code']          ok
W_view_code      a_view_documents       0  ['lines', 'words']         ok
W_view_code      a_view_both            0  ['lines', 'code', 'words'] ok
W_view_documents a_lines                0  ['lines']                  ok
W_view_documents a_code                 0  ['lines', 'code']          ok
W_view_documents a_words                0  ['lines', 'words']         ok
W_view_documents a_all                  0  ['lines', 'code', 'words'] ok
W_view_documents a_lines_documents      0  ['lines', 'words']         ok
W_view_documents a_code_languages       0  ['lines', 'code']          ok
W_view_documents a_view_code            0  ['lines', 'code']          ok
W_view_documents a_view_documents       0  ['lines', 'words']         ok
W_view_documents a_view_both            0  ['lines', 'code', 'words'] ok
W_view_both      a_lines                0  ['lines']                  ok
W_view_both      a_code                 0  ['lines', 'code']          ok
W_view_both      a_words                0  ['lines', 'words']         ok
W_view_both      a_all                  0  ['lines', 'code', 'words'] ok
W_view_both      a_lines_documents      0  ['lines', 'words']         ok
W_view_both      a_code_languages       0  ['lines', 'code']          ok
W_view_both      a_view_code            0  ['lines', 'code']          ok
W_view_both      a_view_documents       0  ['lines', 'words']         ok
W_view_both      a_view_both            0  ['lines', 'code', 'words'] ok

pairs held to serving: 17
cross-warm violations: 0
```

### Correctness: break, no snapshot, warm against cold

```
case                     cold   warm   only  warm source      only source  fresh   verdict
--------------------------------------------------------------------------------------------------------
default                     0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
summary                     0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
tree                        0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
types                       0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
extensions                  0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
largest                     0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
files                       0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
size-apparent               0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
size-allocated              0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
sort-mtime                  0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
sort-name                   0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
depth-1                     0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
scan-depth-2                0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
min-size                    0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
no-gitignore                0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
exclude-ignored             0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
one-filesystem              0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
analyze-lines               0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
analyze-code                0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
analyze-words               0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
analyze-all                 0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
languages                   0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
documents                   0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
view-code                   0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
view-documents              0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
view-code-documents         0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)

answer mismatches: 0
mechanism failures (cache did not serve): 26
  - default: cache-only exited 1, so nothing was stored
  - summary: cache-only exited 1, so nothing was stored
  - tree: cache-only exited 1, so nothing was stored
  - types: cache-only exited 1, so nothing was stored
  - extensions: cache-only exited 1, so nothing was stored
  - largest: cache-only exited 1, so nothing was stored
  - files: cache-only exited 1, so nothing was stored
  - size-apparent: cache-only exited 1, so nothing was stored
  - size-allocated: cache-only exited 1, so nothing was stored
  - sort-mtime: cache-only exited 1, so nothing was stored
  - sort-name: cache-only exited 1, so nothing was stored
  - depth-1: cache-only exited 1, so nothing was stored
  - scan-depth-2: cache-only exited 1, so nothing was stored
  - min-size: cache-only exited 1, so nothing was stored
  - no-gitignore: cache-only exited 1, so nothing was stored
  - exclude-ignored: cache-only exited 1, so nothing was stored
  - one-filesystem: cache-only exited 1, so nothing was stored
  - analyze-lines: warm content tier scanned
  - analyze-code: warm content tier scanned
  - analyze-words: warm content tier scanned
  - analyze-all: warm content tier scanned
  - languages: warm content tier scanned
  - documents: warm content tier scanned
  - view-code: warm content tier scanned
  - view-documents: warm content tier scanned
  - view-code-documents: warm content tier scanned
stale reference instants: 0
cases the snapshot served: 9 of 26
```

### Correctness: break, no snapshot, cross-warm

```
warmer           ask                   rc  analysis.analyze           verdict
------------------------------------------------------------------------------------------
W_none           a_lines                0  ['lines']                  ok
W_none           a_code                 0  ['lines', 'code']          ok
W_none           a_words                0  ['lines', 'words']         ok
W_none           a_all                  0  ['lines', 'code', 'words'] ok
W_none           a_lines_documents      0  ['lines', 'words']         ok
W_none           a_code_languages       0  ['lines', 'code']          ok
W_none           a_view_code            0  ['lines', 'code']          ok
W_none           a_view_documents       0  ['lines', 'words']         ok
W_none           a_view_both            0  ['lines', 'code', 'words'] ok
W_lines          a_lines                0  ['lines']                  NOT-WARM(scanned)
W_lines          a_code                 0  ['lines', 'code']          ok
W_lines          a_words                0  ['lines', 'words']         ok
W_lines          a_all                  0  ['lines', 'code', 'words'] ok
W_lines          a_lines_documents      0  ['lines', 'words']         ok
W_lines          a_code_languages       0  ['lines', 'code']          ok
W_lines          a_view_code            0  ['lines', 'code']          ok
W_lines          a_view_documents       0  ['lines', 'words']         ok
W_lines          a_view_both            0  ['lines', 'code', 'words'] ok
W_code           a_lines                0  ['lines']                  ok
W_code           a_code                 0  ['lines', 'code']          NOT-WARM(scanned)
W_code           a_words                0  ['lines', 'words']         ok
W_code           a_all                  0  ['lines', 'code', 'words'] ok
W_code           a_lines_documents      0  ['lines', 'words']         ok
W_code           a_code_languages       0  ['lines', 'code']          NOT-WARM(scanned)
W_code           a_view_code            0  ['lines', 'code']          NOT-WARM(scanned)
W_code           a_view_documents       0  ['lines', 'words']         ok
W_code           a_view_both            0  ['lines', 'code', 'words'] ok
W_words          a_lines                0  ['lines']                  ok
W_words          a_code                 0  ['lines', 'code']          ok
W_words          a_words                0  ['lines', 'words']         NOT-WARM(scanned)
W_words          a_all                  0  ['lines', 'code', 'words'] ok
W_words          a_lines_documents      0  ['lines', 'words']         NOT-WARM(scanned)
W_words          a_code_languages       0  ['lines', 'code']          ok
W_words          a_view_code            0  ['lines', 'code']          ok
W_words          a_view_documents       0  ['lines', 'words']         NOT-WARM(scanned)
W_words          a_view_both            0  ['lines', 'code', 'words'] ok
W_all            a_lines                0  ['lines']                  ok
W_all            a_code                 0  ['lines', 'code']          ok
W_all            a_words                0  ['lines', 'words']         ok
W_all            a_all                  0  ['lines', 'code', 'words'] NOT-WARM(scanned)
W_all            a_lines_documents      0  ['lines', 'words']         ok
W_all            a_code_languages       0  ['lines', 'code']          ok
W_all            a_view_code            0  ['lines', 'code']          ok
W_all            a_view_documents       0  ['lines', 'words']         ok
W_all            a_view_both            0  ['lines', 'code', 'words'] NOT-WARM(scanned)
W_view_code      a_lines                0  ['lines']                  ok
W_view_code      a_code                 0  ['lines', 'code']          NOT-WARM(scanned)
W_view_code      a_words                0  ['lines', 'words']         ok
W_view_code      a_all                  0  ['lines', 'code', 'words'] ok
W_view_code      a_lines_documents      0  ['lines', 'words']         ok
W_view_code      a_code_languages       0  ['lines', 'code']          NOT-WARM(scanned)
W_view_code      a_view_code            0  ['lines', 'code']          NOT-WARM(scanned)
W_view_code      a_view_documents       0  ['lines', 'words']         ok
W_view_code      a_view_both            0  ['lines', 'code', 'words'] ok
W_view_documents a_lines                0  ['lines']                  ok
W_view_documents a_code                 0  ['lines', 'code']          ok
W_view_documents a_words                0  ['lines', 'words']         NOT-WARM(scanned)
W_view_documents a_all                  0  ['lines', 'code', 'words'] ok
W_view_documents a_lines_documents      0  ['lines', 'words']         NOT-WARM(scanned)
W_view_documents a_code_languages       0  ['lines', 'code']          ok
W_view_documents a_view_code            0  ['lines', 'code']          ok
W_view_documents a_view_documents       0  ['lines', 'words']         NOT-WARM(scanned)
W_view_documents a_view_both            0  ['lines', 'code', 'words'] ok
W_view_both      a_lines                0  ['lines']                  ok
W_view_both      a_code                 0  ['lines', 'code']          ok
W_view_both      a_words                0  ['lines', 'words']         ok
W_view_both      a_all                  0  ['lines', 'code', 'words'] NOT-WARM(scanned)
W_view_both      a_lines_documents      0  ['lines', 'words']         ok
W_view_both      a_code_languages       0  ['lines', 'code']          ok
W_view_both      a_view_code            0  ['lines', 'code']          ok
W_view_both      a_view_documents       0  ['lines', 'words']         ok
W_view_both      a_view_both            0  ['lines', 'code', 'words'] NOT-WARM(scanned)

pairs held to serving: 17
cross-warm violations: 17
  - W_lines -> a_lines: NOT-WARM(scanned)
  - W_code -> a_code: NOT-WARM(scanned)
  - W_code -> a_code_languages: NOT-WARM(scanned)
  - W_code -> a_view_code: NOT-WARM(scanned)
  - W_words -> a_words: NOT-WARM(scanned)
  - W_words -> a_lines_documents: NOT-WARM(scanned)
  - W_words -> a_view_documents: NOT-WARM(scanned)
  - W_all -> a_all: NOT-WARM(scanned)
  - W_all -> a_view_both: NOT-WARM(scanned)
  - W_view_code -> a_code: NOT-WARM(scanned)
  - W_view_code -> a_code_languages: NOT-WARM(scanned)
  - W_view_code -> a_view_code: NOT-WARM(scanned)
  - W_view_documents -> a_words: NOT-WARM(scanned)
  - W_view_documents -> a_lines_documents: NOT-WARM(scanned)
  - W_view_documents -> a_view_documents: NOT-WARM(scanned)
  - W_view_both -> a_all: NOT-WARM(scanned)
  - W_view_both -> a_view_both: NOT-WARM(scanned)
```

### Correctness: break, partial answer stored

```
case                     cold   warm   only  warm source      only source  fresh   verdict
--------------------------------------------------------------------------------------------------------
default                     2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
summary                     2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
tree                        2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
types                       2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
extensions                  2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
largest                     2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
files                       2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
size-apparent               2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
size-allocated              2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
sort-mtime                  2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
sort-name                   2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
depth-1                     2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
scan-depth-2                2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
min-size                    2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
no-gitignore                2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
exclude-ignored             2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
one-filesystem              2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
analyze-lines               2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
analyze-code                2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
analyze-words               2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
analyze-all                 2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
languages                   2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
documents                   2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
view-code                   2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
view-documents              2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
view-code-documents         2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED

answer mismatches: 0
mechanism failures (cache did not serve): 26
  - default: a partial scan was served from the cache
  - summary: a partial scan was served from the cache
  - tree: a partial scan was served from the cache
  - types: a partial scan was served from the cache
  - extensions: a partial scan was served from the cache
  - largest: a partial scan was served from the cache
  - files: a partial scan was served from the cache
  - size-apparent: a partial scan was served from the cache
  - size-allocated: a partial scan was served from the cache
  - sort-mtime: a partial scan was served from the cache
  - sort-name: a partial scan was served from the cache
  - depth-1: a partial scan was served from the cache
  - scan-depth-2: a partial scan was served from the cache
  - min-size: a partial scan was served from the cache
  - no-gitignore: a partial scan was served from the cache
  - exclude-ignored: a partial scan was served from the cache
  - one-filesystem: a partial scan was served from the cache
  - analyze-lines: a partial scan was served from the cache
  - analyze-code: a partial scan was served from the cache
  - analyze-words: a partial scan was served from the cache
  - analyze-all: a partial scan was served from the cache
  - languages: a partial scan was served from the cache
  - documents: a partial scan was served from the cache
  - view-code: a partial scan was served from the cache
  - view-documents: a partial scan was served from the cache
  - view-code-documents: a partial scan was served from the cache
stale reference instants: 0
cases the snapshot served: 0 of 26
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
