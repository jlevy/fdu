# 0.6.0 Stability Pass — 2026-11-12

This is the full record of the 0.6.0 release checklist’s stability pass (step 2 of the
[release process](../guides/release-process.md#the-steps)), written by
`make release-stability`. The summaries belong beside the procedures, in the
[QA playbook’s Current Status](../../../tests/qa/cli-installed-e2e.qa.md) and the
[correctness runbook’s Last Recorded Run](../guides/correctness-runbook.md#last-recorded-run);
this report keeps every table they summarize.

## Verdict

The pass tested commit `bf4981b9976ec0810fdde2a58841b9bb5618fd89`, tree
`2d4612556db40fae7fc7cba10f55e42434c5650d`. A release commit with that tree inherits
every result here by tree identity; check it with `git rev-parse <commit>^{tree}` before
tagging. Failed: check.

That commit was the head of [#213](https://github.com/jlevy/fdu/pull/213), which folded
[#211](https://github.com/jlevy/fdu/pull/211) into the release
([its record](https://github.com/jlevy/fdu/pull/213#issuecomment-6832044145)). The
release commit, the merge commit `4450be7e58b1e87a3d2b0fc8ffc0e8646e5b7a09`, has the
same tree, `2d461255`, so it inherits every result here.
This record supersedes the
[2026-11-11 pass](report-2026-11-11-release-0.6.0-stability-pass.md) of the earlier
release candidate `8a8febc7`, which was never tagged.

**The one failure is the host’s, not the commit’s.** `make check` stopped at its first
target, the supply-chain provenance check, when this host’s network proxy refused its
GitHub API requests for the pinned actions’ commits (HTTP 403). The whole of
`make check` was then run with `-k` at the same commit, as the same unprivileged user
and with no waiver. `supply-chain` was its only failed target, and every other target
passed, the permission fixtures and native watch tests included.
CI ran the provenance check at this commit: `Supply-chain provenance` passed in
[run 43271263330](https://github.com/jlevy/fdu/actions/runs/43271263330).

**A root supplement.** Run unprivileged, the correctness trees hold no device nodes.
The pass’s candidate and correctness stages were therefore run again as root at the same
commit, with the refusal pass as `nobody`. Every step passed, and the trees held 16
kinds; see [Root Supplement](#root-supplement).

| Gate or phase | Verdict |
| --- | --- |
| `make check` | Failed: `make check` exited 2 |
| `make cross-lint` | Passed: `make cross-lint` exited 0: linted x86_64-apple-darwin, x86_64-pc-windows-msvc, i686-unknown-linux-gnu, x86_64-unknown-linux-musl, aarch64-unknown-linux-gnu |
| `make semver-check` | Passed: `make semver-check` exited 0: fdu-core 0.6.0: starts a new compatibility series; nothing to check; fdu 0.6.0: starts a new compatibility series; nothing to check |
| `make release-rehearse` | Passed: `make release-rehearse` exited 0 |
| Candidate | Passed: `fdu --version` printed `fdu 0.6.0-dev+gbf4981b99`, from `fdu-0.6.0-cp312-abi3-manylinux_2_34_x86_64.whl` |
| QA harness, phases 1–5 and 8 | Passed: 49 checks: 49 ok, 0 warn, 0 fail |
| QA phase 6, terminal progress | Pending: a person has not watched a real window, and Windows has not run. Terminal tests passed: 3 tests ran against the candidate; exit status 0; pty probe passed: 38 of 38 checks passed |
| QA phase 7, peer agreement self-test | Passed: 12 readings agree exactly; 0 failed; exit status 0 |
| QA phase 7, peer agreement on real trees | Passed: 36 of 36 readings on 3 trees agree exactly, 0 `UNEXPLAINED`; Every reading is explained; exit status 0 |
| Correctness, refusal tree as an unprivileged user | Passed: 30 of 30 cases partial and withheld; 0 answer mismatches, 0 mechanism failures, 0 stale reference instants; exit status 0 |
| Correctness, served tree | Passed: 30 of 30 served `cache_only` and labelled `stale`; 0 answer mismatches, 0 mechanism failures, 0 stale reference instants; exit status 0 |
| Correctness, cross-warm matrix | Passed: 72 of 72 pairs matched the cold answer; 0 violations; exit status 0 |
| Correctness, break: no snapshot stored | Passed: `warm_cold.py` exited 1, 30 of 30 cases caught (21 `NO-SNAPSHOT`, 9 `NOT-WARM`); `cross_warm.py` exited 1, 17 of 17 same-analyzer pairs `NOT-WARM` |
| Correctness, break: partial answer stored | Passed: `warm_cold.py --refusals-only` exited 1, 30 of 30 cases `PARTIAL-STORED` |

## Regime

- **Host.** Linux x86_64, 4 CPUs, kernel 6.18.44-fc-v378: a Firecracker virtual machine
  with its own kernel, which the detector reports as `virtualized (docker)`, not bare
  metal. The work directory, the trees, and `TMPDIR` were on ext4. The pass ran as an
  ordinary user with `sudo`, made for the run, whom mode bits bind.
  Nothing else ran during the pass.
- **Declared preconditions.** none.
- **Candidate.** The `fdu-0.6.0-cp312-abi3-manylinux_2_34_x86_64.whl` wheel built from
  the commit, installed with `uv tool install` into an isolated tool directory;
  `fdu --version` printed `fdu 0.6.0-dev+gbf4981b99`.
- **Peers.** du (GNU coreutils) 9.4, Dust 1.2.5, pdu 0.24.0, dua 2.45.0, diskus 0.9.0.
- **Trees.** small: `fdu-checkout`; medium: `linux-v6.12`; medium analyze:
  `Documentation`; large: `/usr`; progress: `linux-v6.12-x20-hardlinked`; progress
  analyze: `linux-v6.12`; peer 1: `node-modules-dense`; peer 2: `linux-v6.12`; peer 3:
  `cargo-registry-src`.
- **Correctness trees.** tree: 15 kinds present, 2 absent (chardev, blockdev); served:
  14 kinds present, 2 absent (chardev, blockdev).
- **Tooling.** The QA and correctness scripts at `bf4981b99`.

## Reproduce

```shell
export COMMIT=bf4981b9976ec0810fdde2a58841b9bb5618fd89
export RELEASE=<a directory outside any checkout>
export FDU_QA_SMALL=fdu-checkout
export FDU_QA_MEDIUM=linux-v6.12
export FDU_QA_MEDIUM_ANALYZE=Documentation
export FDU_QA_LARGE=/usr
export FDU_QA_PROGRESS_TREE=linux-v6.12-x20-hardlinked
export FDU_QA_PROGRESS_ANALYZE=linux-v6.12
export FDU_QA_PEER_TREES=node-modules-dense:linux-v6.12:cargo-registry-src
make release-stability
```

## Full Tables

Paths are replaced by the labels above.

### Installed-CLI harness (fdu 0.6.0-dev+gbf4981b99)

| Phase | Name | Verdict | Exit | Real s | RSS MiB | Note |
| --- | --- | --- | ---: | ---: | ---: | --- |
| sanity | help | ok | 0 | 0.010 | 12.1 |  |
| sanity | version | ok | 0 | 0.010 | 11.9 |  |
| sanity | docs | ok | 0 | 0.010 | 12.0 |  |
| sanity | skill | ok | 0 | 0.010 | 12.2 |  |
| views | tree-cold | ok | 0 | 0.010 | 15.4 | cold scan; fresh=0; cached=0 |
| views | tree-warm | ok | 0 | 0.010 | 15.3 | cold scan; fresh=0; cached=0 |
| views | summary | ok | 0 | 0.010 | 14.9 | cold scan; fresh=0; cached=0 |
| views | languages | ok | 0 | 0.020 | 16.0 | cold scan; fresh=0; cached=0 |
| views | families | ok | 0 | 0.020 | 15.9 | cold scan; fresh=0; cached=0 |
| views | types | ok | 0 | 0.020 | 15.9 | cold scan; fresh=0; cached=0 |
| views | extensions | ok | 0 | 0.010 | 15.7 | cold scan; fresh=0; cached=0 |
| views | documents | ok | 0 | 0.100 | 21.8 | cold scan; fresh=1465; cached=0 |
| views | recent | ok | 0 | 0.020 | 16.3 | cold scan; fresh=0; cached=0 |
| views | largest | ok | 0 | 0.020 | 16.3 | cold scan; fresh=0; cached=0 |
| views | files | ok | 0 | 0.020 | 17.2 | cold scan; fresh=0; cached=0 |
| views | full | ok | 0 | 0.020 | 17.3 | cold scan; fresh=0; cached=0 |
| views | combo-kinds | ok | 0 | 0.020 | 16.2 | cold scan; fresh=0; cached=0 |
| views | exclude-ignored-summary | ok | 0 | 0.050 | 16.2 | cold scan; fresh=0; cached=0 |
| views | depth-limit-tree | ok | 0 | 0.020 | 15.3 | cold scan; fresh=0; cached=0 |
| views | scan-depth-1-summary | ok | 0 | 0.010 | 14.6 | cold scan; fresh=0; cached=0 |
| views | json-summary | ok | 0 | 0.020 | 14.9 |  |
| views | yaml-summary | ok | 0 | 0.010 | 15.0 |  |
| cache-analyze | off-code-1 | ok | 0 | 0.100 | 17.3 | cold scan; fresh=1465; cached=0 |
| cache-analyze | off-code-2 | ok | 0 | 0.100 | 17.3 | cold scan; fresh=1465; cached=0 |
| cache-analyze | off-lines-1 | ok | 0 | 0.070 | 17.3 | cold scan; fresh=1465; cached=0 |
| cache-analyze | off-lines-2 | ok | 0 | 0.090 | 17.0 | cold scan; fresh=1465; cached=0 |
| cache-analyze | off-cache-status | ok | 0 | 0.020 | 12.3 |  |
| cache-analyze | on-code-1 | ok | 0 | 0.120 | 17.8 | cold scan; fresh=1465; cached=0 |
| cache-analyze | on-code-2 | ok | 0 | 0.040 | 17.3 | warm revalidation; fresh=0; cached=1465 |
| cache-analyze | on-lines-1 | ok | 0 | 0.070 | 17.8 | warm revalidation; fresh=1465; cached=0 |
| cache-analyze | on-lines-2 | ok | 0 | 0.030 | 17.2 | warm revalidation; fresh=0; cached=1465 |
| cache-analyze | on-cache-status | ok | 0 | 0.010 | 13.6 |  |
| analyze-extra | analyze-words | ok | 0 | 0.100 | 21.5 | warm revalidation; fresh=1465; cached=0 |
| analyze-extra | analyze-all | ok | 0 | 0.130 | 21.5 | warm revalidation; fresh=1465; cached=0 |
| analyze-extra | json-analyze-code | ok | 0 | 0.100 | 17.9 | physical_lines=222360 |
| analyze-extra | yaml-analyze-lines | ok | 0 | 0.080 | 17.7 |  |
| analyze-extra | text-analyze-code-summary | ok | 0 | 0.100 | 17.7 | warm revalidation; fresh=1465; cached=0 |
| watch | watch-sigint | ok | -2 | 3.033 |  | SIGINT after file create; elapsed=3.03s |
| medium | med-tree-cold | ok | 0 | 0.150 | 19.2 | cold scan; fresh=0; cached=0 |
| medium | med-tree-warm | ok | 0 | 0.140 | 19.1 | cold scan; fresh=0; cached=0 |
| medium | med-summary | ok | 0 | 0.130 | 18.5 | cold scan; fresh=0; cached=0 |
| medium | med-languages | ok | 0 | 0.220 | 59.7 | cold scan; fresh=0; cached=0 |
| medium | med-combo-kinds | ok | 0 | 0.220 | 59.9 | cold scan; fresh=0; cached=0 |
| medium | med-recent | ok | 0 | 0.240 | 67.2 | cold scan; fresh=0; cached=0 |
| medium | med-json-summary | ok | 0 | 0.130 | 18.5 |  |
| medium | med-analyze-code-subdir | ok | 0 | 0.210 | 30.1 | cold scan; fresh=10121; cached=0 |
| medium | med-analyze-code-subdir-warm | ok | 0 | 0.070 | 30.0 | warm revalidation; fresh=0; cached=10121 |
| large | large-summary-depth1 | ok | 0 | 0.010 | 14.6 | cold scan; fresh=0; cached=0 |
| large | large-summary-depth2 | ok | 0 | 0.020 | 14.9 | cold scan; fresh=0; cached=0 |

### Peer agreement self-test

#### `<scratch>/probe root`

fdu read 40.0 KiB allocated and 1.0 GiB apparent, exiting 2; across its 13 readings, one
before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: 1 denied.

Measured from the tree: 1 hard-linked inodes, which per-path counting adds 24.0 KiB
allocated to; 4 symbolic links, 1.0 KiB apparent; 5 directories, 20.0 KiB apparent;
directories that could not be listed: 1 denied; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 64.0 KiB | +24.0 KiB | +24.0 KiB | symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | 1 denied | 0.0 s |
| GNU du | allocated | 40.0 KiB | 0 B | 0 B | hard links once -24.0 KiB, symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | 1 denied | 0.0 s |
| GNU du -l | apparent | 1.0 GiB | +1.0 KiB | +1.0 KiB | symbolic links +1.0 KiB | agrees exactly | 1 denied | 0.0 s |
| GNU du | apparent | 1.0 GiB | -18.5 KiB | -18.5 KiB | hard links once -19.5 KiB, symbolic links +1.0 KiB | agrees exactly | 1 denied | 0.0 s |
| dust | allocated | 40.0 KiB | 0 B | 0 B | hard links once -24.0 KiB, symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | 1 denied | 0.0 s |
| dust | apparent | 1.0 GiB | +21.0 KiB | +21.0 KiB | symbolic links +1.0 KiB, directories +20.0 KiB | agrees exactly | 1 denied | 0.0 s |
| pdu | allocated | 64.0 KiB | +24.0 KiB | +24.0 KiB | symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | 1 denied | 0.0 s |
| pdu | apparent | 1.0 GiB | +21.0 KiB | +21.0 KiB | symbolic links +1.0 KiB, directories +20.0 KiB | agrees exactly | 1 denied | 0.0 s |
| dua | allocated | 32.0 KiB | -8.0 KiB | -8.0 KiB | hard links once -24.0 KiB, directories +16.0 KiB | agrees exactly | 1 unreadable, reason not given | 0.0 s |
| dua | apparent | 1.0 GiB | -3.5 KiB | -3.5 KiB | hard links once -19.5 KiB, symbolic links +37 B, directories +16.0 KiB | agrees exactly | 1 unreadable, reason not given | 0.0 s |
| diskus | allocated | 40.0 KiB | 0 B | 0 B | hard links once -24.0 KiB, symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | 1 unreadable, reason not given | 0.0 s |
| diskus | apparent | 1.0 GiB | -18.5 KiB | -18.5 KiB | hard links once -19.5 KiB, symbolic links +1.0 KiB | agrees exactly | 1 unreadable, reason not given | 0.0 s |

GNU du -l also counts each subtree’s own directory and symbolic-link blocks, which fdu
does not: the walk measured 16.0 KiB of them across these 4 directories, and each row
must differ from fdu’s by its own.
Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
4\.

Self-test: 12 readings agree exactly; 0 failed.

### Peer agreement on real trees

#### `node-modules-dense`

fdu read 141.6 MiB allocated and 125.8 MiB apparent, exiting 0; across its 13 readings,
one before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 36 symbolic links, 854 B apparent; 1,146 directories, 4.5 MiB apparent; directories
that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 146.1 MiB | +4.5 MiB | +4.5 MiB | directories +4.5 MiB | agrees exactly | none | 0.0 s |
| GNU du | allocated | 146.1 MiB | +4.5 MiB | +4.5 MiB | directories +4.5 MiB | agrees exactly | none | 0.0 s |
| GNU du -l | apparent | 125.8 MiB | +854 B | +854 B | symbolic links +854 B | agrees exactly | none | 0.0 s |
| GNU du | apparent | 125.8 MiB | +854 B | +854 B | symbolic links +854 B | agrees exactly | none | 0.0 s |
| dust | allocated | 146.1 MiB | +4.5 MiB | +4.5 MiB | directories +4.5 MiB | agrees exactly | none | 0.0 s |
| dust | apparent | 130.3 MiB | +4.5 MiB | +4.5 MiB | symbolic links +854 B, directories +4.5 MiB | agrees exactly | none | 0.0 s |
| pdu | allocated | 146.1 MiB | +4.5 MiB | +4.5 MiB | directories +4.5 MiB | agrees exactly | none | 0.0 s |
| pdu | apparent | 130.3 MiB | +4.5 MiB | +4.5 MiB | symbolic links +854 B, directories +4.5 MiB | agrees exactly | none | 0.0 s |
| dua | allocated | 146.1 MiB | +4.5 MiB | +4.5 MiB | directories +4.5 MiB | agrees exactly | none | 0.0 s |
| dua | apparent | 130.3 MiB | +4.5 MiB | +4.5 MiB | symbolic links +854 B, directories +4.5 MiB | agrees exactly | none | 0.0 s |
| diskus | allocated | 146.1 MiB | +4.5 MiB | +4.5 MiB | directories +4.5 MiB | agrees exactly | none | 0.0 s |
| diskus | apparent | 125.8 MiB | +854 B | +854 B | symbolic links +854 B | agrees exactly | none | 0.0 s |

GNU du -l also counts each subtree’s own directory and symbolic-link blocks, which fdu
does not: the walk measured 4.5 MiB of them across these 1 directories, and each row
must differ from fdu’s by its own.
Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
1\.

#### `linux-v6.12`

fdu read 1.6 GiB allocated and 1.4 GiB apparent, exiting 0; across its 13 readings, one
before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 62 symbolic links, 2.0 KiB apparent; 5,758 directories, 23.1 MiB apparent;
directories that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 1.6 GiB | +23.1 MiB | +23.1 MiB | directories +23.1 MiB | agrees exactly | none | 0.4 s |
| GNU du | allocated | 1.6 GiB | +23.1 MiB | +23.1 MiB | directories +23.1 MiB | agrees exactly | none | 0.4 s |
| GNU du -l | apparent | 1.4 GiB | +2.0 KiB | +2.0 KiB | symbolic links +2.0 KiB | agrees exactly | none | 0.4 s |
| GNU du | apparent | 1.4 GiB | +2.0 KiB | +2.0 KiB | symbolic links +2.0 KiB | agrees exactly | none | 0.4 s |
| dust | allocated | 1.6 GiB | +23.1 MiB | +23.1 MiB | directories +23.1 MiB | agrees exactly | none | 0.2 s |
| dust | apparent | 1.4 GiB | +23.1 MiB | +23.1 MiB | symbolic links +2.0 KiB, directories +23.1 MiB | agrees exactly | none | 0.1 s |
| pdu | allocated | 1.6 GiB | +23.1 MiB | +23.1 MiB | directories +23.1 MiB | agrees exactly | none | 0.1 s |
| pdu | apparent | 1.4 GiB | +23.1 MiB | +23.1 MiB | symbolic links +2.0 KiB, directories +23.1 MiB | agrees exactly | none | 0.1 s |
| dua | allocated | 1.6 GiB | +23.1 MiB | +23.1 MiB | directories +23.1 MiB | agrees exactly | none | 0.3 s |
| dua | apparent | 1.4 GiB | +23.1 MiB | +23.1 MiB | symbolic links +2.0 KiB, directories +23.1 MiB | agrees exactly | none | 0.3 s |
| diskus | allocated | 1.6 GiB | +23.1 MiB | +23.1 MiB | directories +23.1 MiB | agrees exactly | none | 0.1 s |
| diskus | apparent | 1.4 GiB | +2.0 KiB | +2.0 KiB | symbolic links +2.0 KiB | agrees exactly | none | 0.1 s |

GNU du -l also counts each subtree’s own directory and symbolic-link blocks, which fdu
does not: the walk measured 23.1 MiB of them across these 24 directories, and each row
must differ from fdu’s by its own.
Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
24\.

#### `cargo-registry-src`

fdu read 353.6 MiB allocated and 283.4 MiB apparent, exiting 0; across its 13 readings,
one before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 0 symbolic links, 0 B apparent; 4,944 directories, 19.5 MiB apparent; directories
that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 373.1 MiB | +19.5 MiB | +19.5 MiB | directories +19.5 MiB | agrees exactly | none | 0.1 s |
| GNU du | allocated | 373.1 MiB | +19.5 MiB | +19.5 MiB | directories +19.5 MiB | agrees exactly | none | 0.1 s |
| GNU du -l | apparent | 283.4 MiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |
| GNU du | apparent | 283.4 MiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| dust | allocated | 373.1 MiB | +19.5 MiB | +19.5 MiB | directories +19.5 MiB | agrees exactly | none | 0.1 s |
| dust | apparent | 302.9 MiB | +19.5 MiB | +19.5 MiB | directories +19.5 MiB | agrees exactly | none | 0.1 s |
| pdu | allocated | 373.1 MiB | +19.5 MiB | +19.5 MiB | directories +19.5 MiB | agrees exactly | none | 0.0 s |
| pdu | apparent | 302.9 MiB | +19.5 MiB | +19.5 MiB | directories +19.5 MiB | agrees exactly | none | 0.0 s |
| dua | allocated | 373.1 MiB | +19.5 MiB | +19.5 MiB | directories +19.5 MiB | agrees exactly | none | 0.1 s |
| dua | apparent | 302.9 MiB | +19.5 MiB | +19.5 MiB | directories +19.5 MiB | agrees exactly | none | 0.1 s |
| diskus | allocated | 373.1 MiB | +19.5 MiB | +19.5 MiB | directories +19.5 MiB | agrees exactly | none | 0.1 s |
| diskus | apparent | 283.4 MiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |

GNU du -l also counts each subtree’s own directory and symbolic-link blocks, which fdu
does not: the walk measured 19.5 MiB of them across these 1 directories, and each row
must differ from fdu’s by its own.
Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
1\.

Every reading is explained.

### Phase 6 pty probe

```
fdu: <work>/bin/fdu
slow tree: linux-v6.12-x20-hardlinked
small tree: fdu-checkout
analyze tree: linux-v6.12

ok   slow scan exits 0 (1.4 s)
ok   slow scan draws frames (9 frames)
ok   first frame after about half a second (0.52 s)
ok   a frame says Scanning
     phases seen: ['Scanning']
ok   the last frame is erased before the report
ok   the report follows the erase, with no frame left in it
ok   every frame fits 100 columns (at most 99)
ok   no frame contains a newline
ok   file counts climb ([471239] to [1641486])
ok   the counts hold their column while Scanning ([56])
     sample frame: ⠼ linux-v6.12-x20-hardlinked  Scanning        986,291 files ·    72,297 dirs ·   23 GiB · 0.9 s
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
ok   60 columns: 61 frames fit, widest 59 ('⠏ /var/t…v6.12  Summarizing  86,618 files · 1.5 GiB · 6.4 s')
ok   40 columns: 60 frames fit, widest 13 ('⠇ Summarizing')
ok   30 columns: 62 frames fit, widest 13 ('⠙ Summarizing')
ok   20 columns: 61 frames fit, widest 13 ('⠋ Summarizing')
ok   19 columns: 62 frames fit, spinner and phase word only ('⠋ Summarizing')
ok   12 columns: 63 frames fit, spinner and phase word only ('⠋ Analyzing')
ok   narrowing mid-run shrinks every later frame without wrapping
ok   --analyze all shows Analyzing with a percentage ([10] to [95])
ok   the percentage climbs
ok   the analyze indicator is erased before the report

38 of 38 checks passed
```

### Terminal tests

```
...
----------------------------------------------------------------------
Ran 3 tests in 19.995s

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
sort-time                   2      2      1  cold_scan        -            -       withheld
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
ages                        2      2      1  cold_scan        -            -       withheld
since-kind-file             2      2      1  cold_scan        -            -       withheld
time-modified               2      2      1  cold_scan        -            -       withheld
volumes                     2      2      1  cold_scan        -            -       withheld

answer mismatches: 0
mechanism failures (cache did not serve): 0
stale reference instants: 0
cases the snapshot served: 0 of 30
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
sort-time                   0      0      0  cold_scan        cache_only   stale   ok
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
ages                        0      0      0  cold_scan        cache_only   stale   ok
since-kind-file             0      0      0  cold_scan        cache_only   stale   ok
time-modified               0      0      0  cold_scan        cache_only   stale   ok
volumes                     0      0      0  cold_scan        cache_only   stale   ok

answer mismatches: 0
mechanism failures (cache did not serve): 0
stale reference instants: 0
cases the snapshot served: 30 of 30
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
sort-time                   0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
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
ages                        0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
since-kind-file             0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
time-modified               0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
volumes                     0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)

answer mismatches: 0
mechanism failures (cache did not serve): 30
  - default: cache-only exited 1, so nothing was stored
  - summary: cache-only exited 1, so nothing was stored
  - tree: cache-only exited 1, so nothing was stored
  - types: cache-only exited 1, so nothing was stored
  - extensions: cache-only exited 1, so nothing was stored
  - largest: cache-only exited 1, so nothing was stored
  - files: cache-only exited 1, so nothing was stored
  - size-apparent: cache-only exited 1, so nothing was stored
  - size-allocated: cache-only exited 1, so nothing was stored
  - sort-time: cache-only exited 1, so nothing was stored
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
  - ages: cache-only exited 1, so nothing was stored
  - since-kind-file: cache-only exited 1, so nothing was stored
  - time-modified: cache-only exited 1, so nothing was stored
  - volumes: cache-only exited 1, so nothing was stored
stale reference instants: 0
cases the snapshot served: 9 of 30
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
sort-time                   2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
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
ages                        2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
since-kind-file             2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
time-modified               2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
volumes                     2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED

answer mismatches: 0
mechanism failures (cache did not serve): 30
  - default: a partial scan was served from the cache
  - summary: a partial scan was served from the cache
  - tree: a partial scan was served from the cache
  - types: a partial scan was served from the cache
  - extensions: a partial scan was served from the cache
  - largest: a partial scan was served from the cache
  - files: a partial scan was served from the cache
  - size-apparent: a partial scan was served from the cache
  - size-allocated: a partial scan was served from the cache
  - sort-time: a partial scan was served from the cache
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
  - ages: a partial scan was served from the cache
  - since-kind-file: a partial scan was served from the cache
  - time-modified: a partial scan was served from the cache
  - volumes: a partial scan was served from the cache
stale reference instants: 0
cases the snapshot served: 0 of 30
```

## Root Supplement

The pass’s candidate and correctness stages ran again at the same commit as root on the
same host, with `--only candidate,correctness` and `FDU_TEST_ALLOW_NO_PERMISSION_BITS=1`
declared. The refusal pass ran as `nobody`, which mode bits bind on this host.
The candidate was the wheel built from the commit, reporting `fdu 0.6.0-dev+gbf4981b99`.
The refusal tree held 16 kinds and lacked one, `permission-denied-effective`, which root
cannot experience; the served tree held all 16, device nodes included.

| Pass | Result |
| --- | --- |
| Refusal tree, as `nobody` | 30 of 30 cases partial and withheld; 0 answer mismatches, 0 mechanism failures, 0 stale reference instants; exit status 0 |
| Served tree | 30 of 30 served `cache_only` and labelled `stale`; 0 answer mismatches, 0 mechanism failures, 0 stale reference instants; exit status 0 |
| Cross-warm matrix | 72 of 72 pairs matched the cold answer; 0 violations; exit status 0 |
| Break: no snapshot stored | `warm_cold.py` exited 1, 30 of 30 cases caught (21 `NO-SNAPSHOT`, 9 `NOT-WARM`); `cross_warm.py` exited 1, 17 of 17 same-analyzer pairs `NOT-WARM` |
| Break: partial answer stored | `warm_cold.py --refusals-only` exited 1, 30 of 30 cases `PARTIAL-STORED` |

### Correctness as root: refusals

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
sort-time                   2      2      1  cold_scan        -            -       withheld
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
ages                        2      2      1  cold_scan        -            -       withheld
since-kind-file             2      2      1  cold_scan        -            -       withheld
time-modified               2      2      1  cold_scan        -            -       withheld
volumes                     2      2      1  cold_scan        -            -       withheld

answer mismatches: 0
mechanism failures (cache did not serve): 0
stale reference instants: 0
cases the snapshot served: 0 of 30
```

### Correctness as root: served

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
sort-time                   0      0      0  cold_scan        cache_only   stale   ok
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
ages                        0      0      0  cold_scan        cache_only   stale   ok
since-kind-file             0      0      0  cold_scan        cache_only   stale   ok
time-modified               0      0      0  cold_scan        cache_only   stale   ok
volumes                     0      0      0  cold_scan        cache_only   stale   ok

answer mismatches: 0
mechanism failures (cache did not serve): 0
stale reference instants: 0
cases the snapshot served: 30 of 30
```

### Correctness as root: cross-warm

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

### Correctness as root: break, no snapshot, warm against cold

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
sort-time                   0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
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
ages                        0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
since-kind-file             0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
time-modified               0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
volumes                     0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)

answer mismatches: 0
mechanism failures (cache did not serve): 30
  - default: cache-only exited 1, so nothing was stored
  - summary: cache-only exited 1, so nothing was stored
  - tree: cache-only exited 1, so nothing was stored
  - types: cache-only exited 1, so nothing was stored
  - extensions: cache-only exited 1, so nothing was stored
  - largest: cache-only exited 1, so nothing was stored
  - files: cache-only exited 1, so nothing was stored
  - size-apparent: cache-only exited 1, so nothing was stored
  - size-allocated: cache-only exited 1, so nothing was stored
  - sort-time: cache-only exited 1, so nothing was stored
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
  - ages: cache-only exited 1, so nothing was stored
  - since-kind-file: cache-only exited 1, so nothing was stored
  - time-modified: cache-only exited 1, so nothing was stored
  - volumes: cache-only exited 1, so nothing was stored
stale reference instants: 0
cases the snapshot served: 9 of 30
```

### Correctness as root: break, no snapshot, cross-warm

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

### Correctness as root: break, partial answer stored

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
sort-time                   2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
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
ages                        2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
since-kind-file             2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
time-modified               2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
volumes                     2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED

answer mismatches: 0
mechanism failures (cache did not serve): 30
  - default: a partial scan was served from the cache
  - summary: a partial scan was served from the cache
  - tree: a partial scan was served from the cache
  - types: a partial scan was served from the cache
  - extensions: a partial scan was served from the cache
  - largest: a partial scan was served from the cache
  - files: a partial scan was served from the cache
  - size-apparent: a partial scan was served from the cache
  - size-allocated: a partial scan was served from the cache
  - sort-time: a partial scan was served from the cache
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
  - ages: a partial scan was served from the cache
  - since-kind-file: a partial scan was served from the cache
  - time-modified: a partial scan was served from the cache
  - volumes: a partial scan was served from the cache
stale reference instants: 0
cases the snapshot served: 0 of 30
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
