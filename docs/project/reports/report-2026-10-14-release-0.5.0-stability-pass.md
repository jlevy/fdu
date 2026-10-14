# 0.5.0 Stability Pass — 2026-10-14

This is the full record of the 0.5.0 release checklist’s stability pass (step 2 of the
[release process](../guides/release-process.md#the-steps)), written by
`make release-stability`. The summaries belong beside the procedures, in the
[QA playbook’s Current Status](../../../tests/qa/cli-installed-e2e.qa.md) and the
[correctness runbook’s Last Recorded Run](../guides/correctness-runbook.md#last-recorded-run);
this report keeps every table they summarize.

## Verdict

The pass tested commit `4a513b00f7d7a508344725a5ebd48e84299f4727`, tree
`bd82aedb40b7e50c1e128c3be791335408abda32`. A release commit with that tree inherits
every result here by tree identity; check it with `git rev-parse <commit>^{tree}` before
tagging. Nothing failed, and nothing was skipped.

| Gate or phase | Verdict |
| --- | --- |
| `make check` | Passed: `make check` exited 0 |
| `make cross-lint` | Passed: `make cross-lint` exited 0: linted x86_64-apple-darwin, x86_64-pc-windows-msvc, i686-unknown-linux-gnu, x86_64-unknown-linux-musl, aarch64-unknown-linux-gnu |
| `make semver-check` | Passed: `make semver-check` exited 0: fdu-core 0.5.0: starts a new compatibility series; nothing to check; fdu 0.5.0: starts a new compatibility series; nothing to check |
| `make release-rehearse` | Passed: `make release-rehearse` exited 0 |
| Candidate | Passed: `fdu --version` printed `fdu 0.5.0-dev+g4a513b00f`, from `fdu-0.5.0-cp312-abi3-manylinux_2_34_x86_64.whl` |
| QA harness, phases 1–5 and 8 | Passed: 49 checks: 49 ok, 0 warn, 0 fail |
| QA phase 6, terminal progress | Waived by the maintainer for 0.5.0: a person has not watched a real window, and Windows progress QA has not run. Terminal tests passed: 3 tests ran against the candidate; exit status 0; pty probe passed: 38 of 38 checks passed, rerun alone on larger progress trees after its first attempt refused the default ones as too fast to judge |
| QA phase 7, peer agreement self-test | Passed: 12 readings agree exactly; 0 failed; exit status 0 |
| QA phase 7, peer agreement on real trees | Passed: 36 of 36 readings on 3 trees agree exactly, 0 `UNEXPLAINED`; Every reading is explained; exit status 0 |
| Correctness, refusal tree as `nobody` | Passed: 26 of 26 cases partial and withheld; 0 answer mismatches, 0 mechanism failures, 0 stale reference instants; exit status 0 |
| Correctness, served tree | Passed: 26 of 26 served `cache_only` and labelled `stale`; 0 answer mismatches, 0 mechanism failures, 0 stale reference instants; exit status 0 |
| Correctness, cross-warm matrix | Passed: 72 of 72 pairs matched the cold answer; 0 violations; exit status 0 |
| Correctness, break: no snapshot stored | Passed: `warm_cold.py` exited 1, 26 of 26 cases caught (17 `NO-SNAPSHOT`, 9 `NOT-WARM`); `cross_warm.py` exited 1, 17 of 17 same-analyzer pairs `NOT-WARM` |
| Correctness, break: partial answer stored | Passed: `warm_cold.py --refusals-only` exited 1, 26 of 26 cases `PARTIAL-STORED` |

## Regime

- **Host.** Linux x86_64, a 4-vCPU Firecracker virtual machine (Intel Xeon at 2.8 GHz),
  kernel 6.18.44-fc-v114, the work directory on ext4, running as root.
  Nothing else ran locally during the gates and the QA and correctness steps, apart from
  light `make release-candidate` downloads and `make release-body`; the GitHub rehearsal
  ran remotely meanwhile.
  The pass reports no timing claims.
- **Declared preconditions.** `FDU_TEST_ALLOW_NO_PERMISSION_BITS=1`,
  `FDU_TEST_ALLOW_NO_NATIVE_WATCH=1`, `UV_PYTHON=3.12`.
- **Candidate.** The `fdu-0.5.0-cp312-abi3-manylinux_2_34_x86_64.whl` wheel built from
  the commit, installed with `uv tool install` into an isolated tool directory;
  `fdu --version` printed `fdu 0.5.0-dev+g4a513b00f`.
- **Peers.** du (GNU coreutils) 9.4, Dust 1.2.5, pdu 0.24.0, dua 2.45.0, diskus 0.9.0.
- **Trees.** small: `fdu-checkout`; medium: `linux-v6.12`; medium analyze:
  `linux-v6.12/Documentation`; large: `linux-balanced-1m`; progress: `subjects`;
  progress analyze: `linux-v6.12`; peer 1: `node-modules-dense`; peer 2: `linux-v6.12`;
  peer 3: `cargo-registry-src`.
- **Correctness trees.** tree: 16 kinds present, 1 absent (permission-denied-effective);
  served: 16 kinds present, 0 absent.
- **Tooling.** The QA and correctness scripts at `7fc71fe7a`.

## Reproduce

```shell
export COMMIT=4a513b00f7d7a508344725a5ebd48e84299f4727
export RELEASE=<a directory outside any checkout>
export FDU_QA_SMALL=fdu-checkout
export FDU_QA_MEDIUM=linux-v6.12
export FDU_QA_MEDIUM_ANALYZE='linux-v6.12/Documentation'
export FDU_QA_LARGE=linux-balanced-1m
export FDU_QA_PROGRESS_TREE=subjects
export FDU_QA_PROGRESS_ANALYZE=linux-v6.12
export FDU_QA_PEER_TREES=node-modules-dense:linux-v6.12:cargo-registry-src
make release-stability
```

## Full Tables

Paths are replaced by the labels above.

### Installed-CLI harness (fdu 0.5.0-dev+g4a513b00f)

| Phase | Name | Verdict | Exit | Real s | RSS MiB | Note |
| --- | --- | --- | ---: | ---: | ---: | --- |
| sanity | help | ok | 0 | 0.030 | 12.4 |  |
| sanity | version | ok | 0 | 0.030 | 12.4 |  |
| sanity | docs | ok | 0 | 0.030 | 12.3 |  |
| sanity | skill | ok | 0 | 0.020 | 12.5 |  |
| views | tree-cold | ok | 0 | 0.040 | 15.1 | cold scan; fresh=0; cached=0 |
| views | tree-warm | ok | 0 | 0.040 | 15.1 | cold scan; fresh=0; cached=0 |
| views | summary | ok | 0 | 0.030 | 14.8 | cold scan; fresh=0; cached=0 |
| views | languages | ok | 0 | 0.050 | 15.4 | cold scan; fresh=0; cached=0 |
| views | families | ok | 0 | 0.040 | 15.5 | cold scan; fresh=0; cached=0 |
| views | types | ok | 0 | 0.070 | 15.5 | cold scan; fresh=0; cached=0 |
| views | extensions | ok | 0 | 0.050 | 15.4 | cold scan; fresh=0; cached=0 |
| views | documents | ok | 0 | 0.280 | 21.1 | cold scan; fresh=1255; cached=0 |
| views | recent | ok | 0 | 0.030 | 15.8 | cold scan; fresh=0; cached=0 |
| views | largest | ok | 0 | 0.030 | 15.8 | cold scan; fresh=0; cached=0 |
| views | files | ok | 0 | 0.040 | 16.5 | cold scan; fresh=0; cached=0 |
| views | full | ok | 0 | 0.040 | 16.5 | cold scan; fresh=0; cached=0 |
| views | combo-kinds | ok | 0 | 0.040 | 15.7 | cold scan; fresh=0; cached=0 |
| views | exclude-ignored-summary | ok | 0 | 0.070 | 15.9 | cold scan; fresh=0; cached=0 |
| views | depth-limit-tree | ok | 0 | 0.030 | 15.2 | cold scan; fresh=0; cached=0 |
| views | scan-depth-1-summary | ok | 0 | 0.030 | 14.5 | cold scan; fresh=0; cached=0 |
| views | json-summary | ok | 0 | 0.040 | 14.8 |  |
| views | yaml-summary | ok | 0 | 0.030 | 15.0 |  |
| cache-analyze | off-code-1 | ok | 0 | 0.190 | 17.1 | cold scan; fresh=1255; cached=0 |
| cache-analyze | off-code-2 | ok | 0 | 0.190 | 16.9 | cold scan; fresh=1255; cached=0 |
| cache-analyze | off-lines-1 | ok | 0 | 0.160 | 17.0 | cold scan; fresh=1255; cached=0 |
| cache-analyze | off-lines-2 | ok | 0 | 0.150 | 16.8 | cold scan; fresh=1255; cached=0 |
| cache-analyze | off-cache-status | ok | 0 | 0.030 | 13.6 |  |
| cache-analyze | on-code-1 | ok | 0 | 0.190 | 17.4 | cold scan; fresh=1255; cached=0 |
| cache-analyze | on-code-2 | ok | 0 | 0.040 | 17.1 | warm revalidation; fresh=0; cached=1255 |
| cache-analyze | on-lines-1 | ok | 0 | 0.160 | 17.6 | warm revalidation; fresh=1255; cached=0 |
| cache-analyze | on-lines-2 | ok | 0 | 0.050 | 17.0 | warm revalidation; fresh=0; cached=1255 |
| cache-analyze | on-cache-status | ok | 0 | 0.030 | 13.9 |  |
| analyze-extra | analyze-words | ok | 0 | 0.240 | 21.1 | warm revalidation; fresh=1255; cached=0 |
| analyze-extra | analyze-all | ok | 0 | 0.240 | 21.2 | warm revalidation; fresh=1255; cached=0 |
| analyze-extra | json-analyze-code | ok | 0 | 0.190 | 17.4 | physical_lines=212632 |
| analyze-extra | yaml-analyze-lines | ok | 0 | 0.140 | 17.2 |  |
| analyze-extra | text-analyze-code-summary | ok | 0 | 0.160 | 17.1 | warm revalidation; fresh=1255; cached=0 |
| watch | watch-sigint | ok | -2 | 3.036 |  | SIGINT after file create; elapsed=3.04s |
| medium | med-tree-cold | ok | 0 | 0.360 | 18.2 | cold scan; fresh=0; cached=0 |
| medium | med-tree-warm | ok | 0 | 0.180 | 19.6 | cold scan; fresh=0; cached=0 |
| medium | med-summary | ok | 0 | 0.220 | 20.7 | cold scan; fresh=0; cached=0 |
| medium | med-languages | ok | 0 | 0.260 | 58.1 | cold scan; fresh=0; cached=0 |
| medium | med-combo-kinds | ok | 0 | 0.330 | 59.4 | cold scan; fresh=0; cached=0 |
| medium | med-recent | ok | 0 | 0.340 | 67.1 | cold scan; fresh=0; cached=0 |
| medium | med-json-summary | ok | 0 | 0.130 | 20.3 |  |
| medium | med-analyze-code-subdir | ok | 0 | 3.250 | 29.3 | cold scan; fresh=10121; cached=0 |
| medium | med-analyze-code-subdir-warm | ok | 0 | 0.150 | 29.8 | warm revalidation; fresh=0; cached=10121 |
| large | large-summary-depth1 | ok | 0 | 0.030 | 14.4 | cold scan; fresh=0; cached=0 |
| large | large-summary-depth2 | ok | 0 | 0.030 | 14.9 | cold scan; fresh=0; cached=0 |

### Peer agreement self-test

#### `<scratch>/probe root`

fdu read 44.0 KiB allocated and 1.0 GiB apparent, exiting 0; across its 13 readings, one
before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 1 hard-linked inodes, which per-path counting adds 24.0 KiB
allocated to; 4 symbolic links, 1.0 KiB apparent; 5 directories, 20.0 KiB apparent;
directories that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 68.0 KiB | +24.0 KiB | +24.0 KiB | symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| GNU du | allocated | 44.0 KiB | 0 B | 0 B | hard links once -24.0 KiB, symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| GNU du -l | apparent | 1.0 GiB | +1.0 KiB | +1.0 KiB | symbolic links +1.0 KiB | agrees exactly | none | 0.0 s |
| GNU du | apparent | 1.0 GiB | -18.5 KiB | -18.5 KiB | hard links once -19.5 KiB, symbolic links +1.0 KiB | agrees exactly | none | 0.0 s |
| dust | allocated | 44.0 KiB | 0 B | 0 B | hard links once -24.0 KiB, symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| dust | apparent | 1.0 GiB | +21.0 KiB | +21.0 KiB | symbolic links +1.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| pdu | allocated | 68.0 KiB | +24.0 KiB | +24.0 KiB | symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| pdu | apparent | 1.0 GiB | +21.0 KiB | +21.0 KiB | symbolic links +1.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| dua | allocated | 36.0 KiB | -8.0 KiB | -8.0 KiB | hard links once -24.0 KiB, directories +16.0 KiB | agrees exactly | none | 0.0 s |
| dua | apparent | 1.0 GiB | -3.5 KiB | -3.5 KiB | hard links once -19.5 KiB, symbolic links +37 B, directories +16.0 KiB | agrees exactly | none | 0.0 s |
| diskus | allocated | 44.0 KiB | 0 B | 0 B | hard links once -24.0 KiB, symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| diskus | apparent | 1.0 GiB | -18.5 KiB | -18.5 KiB | hard links once -19.5 KiB, symbolic links +1.0 KiB | agrees exactly | none | 0.0 s |

GNU du -l also counts each subtree’s own directory and symbolic-link blocks, which fdu
does not: the walk measured 16.0 KiB of them across these 4 directories, and each row
must differ from fdu’s by its own.
Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
4\.

Self-test: 12 readings agree exactly; 0 failed.

### Peer agreement on real trees

#### `node-modules-dense`

fdu read 722.8 MiB allocated and 514.0 MiB apparent, exiting 0; across its 13 readings,
one before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 107 symbolic links, 2.6 KiB apparent; 9,439 directories, 37.5 MiB apparent;
directories that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 760.3 MiB | +37.6 MiB | +37.6 MiB | directories +37.6 MiB | agrees exactly | none | 0.3 s |
| GNU du | allocated | 760.3 MiB | +37.6 MiB | +37.6 MiB | directories +37.6 MiB | agrees exactly | none | 0.3 s |
| GNU du -l | apparent | 514.0 MiB | +2.6 KiB | +2.6 KiB | symbolic links +2.6 KiB | agrees exactly | none | 0.3 s |
| GNU du | apparent | 514.0 MiB | +2.6 KiB | +2.6 KiB | symbolic links +2.6 KiB | agrees exactly | none | 0.3 s |
| dust | allocated | 760.3 MiB | +37.6 MiB | +37.6 MiB | directories +37.6 MiB | agrees exactly | none | 0.1 s |
| dust | apparent | 551.6 MiB | +37.5 MiB | +37.5 MiB | symbolic links +2.6 KiB, directories +37.5 MiB | agrees exactly | none | 0.1 s |
| pdu | allocated | 760.3 MiB | +37.6 MiB | +37.6 MiB | directories +37.6 MiB | agrees exactly | none | 0.1 s |
| pdu | apparent | 551.6 MiB | +37.5 MiB | +37.5 MiB | symbolic links +2.6 KiB, directories +37.5 MiB | agrees exactly | none | 0.1 s |
| dua | allocated | 760.3 MiB | +37.6 MiB | +37.6 MiB | directories +37.6 MiB | agrees exactly | none | 0.5 s |
| dua | apparent | 551.6 MiB | +37.5 MiB | +37.5 MiB | symbolic links +2.6 KiB, directories +37.5 MiB | agrees exactly | none | 0.5 s |
| diskus | allocated | 760.3 MiB | +37.6 MiB | +37.6 MiB | directories +37.6 MiB | agrees exactly | none | 0.1 s |
| diskus | apparent | 514.0 MiB | +2.6 KiB | +2.6 KiB | symbolic links +2.6 KiB | agrees exactly | none | 0.1 s |

GNU du -l also counts each subtree’s own directory and symbolic-link blocks, which fdu
does not: the walk measured 37.6 MiB of them across these 1 directories, and each row
must differ from fdu’s by its own.
Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
1\.

#### `linux-v6.12`

fdu read 1.8 GiB allocated and 1.6 GiB apparent, exiting 0; across its 13 readings, one
before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 62 symbolic links, 2.0 KiB apparent; 5,769 directories, 23.2 MiB apparent;
directories that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 1.9 GiB | +23.2 MiB | +23.2 MiB | directories +23.2 MiB | agrees exactly | none | 0.3 s |
| GNU du | allocated | 1.9 GiB | +23.2 MiB | +23.2 MiB | directories +23.2 MiB | agrees exactly | none | 0.3 s |
| GNU du -l | apparent | 1.6 GiB | +2.0 KiB | +2.0 KiB | symbolic links +2.0 KiB | agrees exactly | none | 0.3 s |
| GNU du | apparent | 1.6 GiB | +2.0 KiB | +2.0 KiB | symbolic links +2.0 KiB | agrees exactly | none | 0.3 s |
| dust | allocated | 1.9 GiB | +23.2 MiB | +23.2 MiB | directories +23.2 MiB | agrees exactly | none | 0.2 s |
| dust | apparent | 1.7 GiB | +23.2 MiB | +23.2 MiB | symbolic links +2.0 KiB, directories +23.2 MiB | agrees exactly | none | 0.1 s |
| pdu | allocated | 1.9 GiB | +23.2 MiB | +23.2 MiB | directories +23.2 MiB | agrees exactly | none | 0.1 s |
| pdu | apparent | 1.7 GiB | +23.2 MiB | +23.2 MiB | symbolic links +2.0 KiB, directories +23.2 MiB | agrees exactly | none | 0.1 s |
| dua | allocated | 1.9 GiB | +23.2 MiB | +23.2 MiB | directories +23.2 MiB | agrees exactly | none | 0.5 s |
| dua | apparent | 1.7 GiB | +23.2 MiB | +23.2 MiB | symbolic links +2.0 KiB, directories +23.2 MiB | agrees exactly | none | 0.5 s |
| diskus | allocated | 1.9 GiB | +23.2 MiB | +23.2 MiB | directories +23.2 MiB | agrees exactly | none | 0.1 s |
| diskus | apparent | 1.6 GiB | +2.0 KiB | +2.0 KiB | symbolic links +2.0 KiB | agrees exactly | none | 0.1 s |

GNU du -l also counts each subtree’s own directory and symbolic-link blocks, which fdu
does not: the walk measured 23.2 MiB of them across these 25 directories, and each row
must differ from fdu’s by its own.
Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
25\.

#### `cargo-registry-src`

fdu read 277.6 MiB allocated and 242.7 MiB apparent, exiting 0; across its 13 readings,
one before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 0 symbolic links, 0 B apparent; 2,912 directories, 11.4 MiB apparent; directories
that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 289.0 MiB | +11.4 MiB | +11.4 MiB | directories +11.4 MiB | agrees exactly | none | 0.1 s |
| GNU du | allocated | 289.0 MiB | +11.4 MiB | +11.4 MiB | directories +11.4 MiB | agrees exactly | none | 0.1 s |
| GNU du -l | apparent | 242.7 MiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| GNU du | apparent | 242.7 MiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| dust | allocated | 289.0 MiB | +11.4 MiB | +11.4 MiB | directories +11.4 MiB | agrees exactly | none | 0.0 s |
| dust | apparent | 254.1 MiB | +11.4 MiB | +11.4 MiB | directories +11.4 MiB | agrees exactly | none | 0.0 s |
| pdu | allocated | 289.0 MiB | +11.4 MiB | +11.4 MiB | directories +11.4 MiB | agrees exactly | none | 0.0 s |
| pdu | apparent | 254.1 MiB | +11.4 MiB | +11.4 MiB | directories +11.4 MiB | agrees exactly | none | 0.0 s |
| dua | allocated | 289.0 MiB | +11.4 MiB | +11.4 MiB | directories +11.4 MiB | agrees exactly | none | 0.1 s |
| dua | apparent | 254.1 MiB | +11.4 MiB | +11.4 MiB | directories +11.4 MiB | agrees exactly | none | 0.1 s |
| diskus | allocated | 289.0 MiB | +11.4 MiB | +11.4 MiB | directories +11.4 MiB | agrees exactly | none | 0.0 s |
| diskus | apparent | 242.7 MiB | 0 B | 0 B | — | agrees exactly | none | 0.0 s |

GNU du -l also counts each subtree’s own directory and symbolic-link blocks, which fdu
does not: the walk measured 11.4 MiB of them across these 272 directories, and each row
must differ from fdu’s by its own.
Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
272\.

Every reading is explained.

### Phase 6 pty probe

```
fdu: <work>/bin/fdu
slow tree: subjects
small tree: fdu-checkout
analyze tree: linux-v6.12

ok   slow scan exits 0 (1.1 s)
ok   slow scan draws frames (6 frames)
ok   first frame after about half a second (0.52 s)
ok   a frame says Scanning
     phases seen: ['Scanning']
ok   the last frame is erased before the report
ok   the report follows the erase, with no frame left in it
ok   every frame fits 100 columns (at most 99)
ok   no frame contains a newline
ok   file counts climb ([473874] to [1007713])
ok   the counts hold their column while Scanning ([38])
     sample frame: ⠸ ~/subjects  Scanning        776,738 files ·   104,672 dirs ·  4.7 GiB · 0.8 s
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
ok   60 columns: 76 frames fit, widest 59 ('⠼ ~/subj…v6.12  Summarizing  86,643 files · 1.8 GiB · 7.9 s')
ok   40 columns: 76 frames fit, widest 13 ('⠼ Summarizing')
ok   30 columns: 76 frames fit, widest 13 ('⠼ Summarizing')
ok   20 columns: 75 frames fit, widest 13 ('⠸ Summarizing')
ok   19 columns: 77 frames fit, spinner and phase word only ('⠦ Summarizing')
ok   12 columns: 74 frames fit, spinner and phase word only ('⠋ Analyzing')
ok   narrowing mid-run shrinks every later frame without wrapping
ok   --analyze all shows Analyzing with a percentage ([10] to [94])
ok   the percentage climbs
ok   the analyze indicator is erased before the report

38 of 38 checks passed
```

### Terminal tests

```
...
----------------------------------------------------------------------
Ran 3 tests in 22.987s

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

## Maintainer Decisions

The maintainer made three decisions during the release session on 2026-10-13. They are
recorded on the release bead `fdu-f243`.

- **Phase 6.** Human real-window and Windows Terminal progress QA are waived for 0.5.0.
  The automated pty probe and terminal tests passed.
- **macOS peer agreement (`fdu-wo7c`).** The live-Library residual from the 2026-10-12
  macOS pass is accepted as live-tree churn and not a tag blocker.
  The bead stays open as a post-release investigation.
- **The Linux summary (`fdu-4arl`).** The dense summary is level with the age column’s
  base, −0.22% [−3.50%, +3.89%] (H198, exp-222). That is inconclusive at the registered
  +3% margin, so by the registered rule the tag proceeds and the bead stays open.

## Release Record

| Item | Record |
| --- | --- |
| Release commit | `4a513b00f7d7a508344725a5ebd48e84299f4727`, the merge of [#202](https://github.com/jlevy/fdu/pull/202), with the same tree as its gated head `7fc71fe7` |
| Tag | `v0.5.0`: annotated tag object `a98e9868`, message `fdu 0.5.0`, unsigned; `make release-verify-tag` passed every line |
| Preflight | Every line `ok`, including the demo declaration, the protected `release` environment, and no registry secrets |
| Rehearsal | [Run 38464512947](https://github.com/jlevy/fdu/actions/runs/38464512947), all jobs successful on attempt 2, publishing and announcement skipped. Attempt 1 failed one release-script test that runs the runner’s `rustc` through the pinned toolchain, which fetches on demand; nothing it reads changed since the previous green rehearsal, the test passed locally, and the one rerun passed. The eight files were downloaded and checked |
| Release body | `make release-body` passed; GitHub’s rendering has no `<br>` inside a paragraph |
| Demo video | `make release-demo` attached the declared `fdu-demo.mp4` (3,179,202 bytes) to the draft; GitHub reported the declared size and digest |
| Publishing run | [Run 38493673234](https://github.com/jlevy/fdu/actions/runs/38493673234), dispatched on `v0.5.0` at the release commit. Every job succeeded on the first attempt, including `Confirm the release environment is protected`, `Publish to crates.io and PyPI`, and `Announce on GitHub`. The `release` environment was approved on the maintainer’s explicit go-ahead for this release |
| GitHub release | [fdu 0.5.0](https://github.com/jlevy/fdu/releases/tag/v0.5.0): published 2026-10-14T01:20:17Z, immutable, the latest release, with twelve assets: the eleven release files and the declared demo video |
| `make release-published` | crates.io `fdu-core` and `fdu` and PyPI `fdu` 0.5.0 each identical to the run’s files |
| `make release-announced --cargo` | Every line `ok`: the release title, body, and twelve assets; docs.rs built for both crates; `uvx fdu@0.5.0` and `fdu@latest`, and a `cargo install` build, each print `fdu 0.5.0` |
| Use after publication | The PyPI wheel through `uvx` rendered a tree with ages; the crates.io build gave a summary and a two-root report with its total row; the PyPI package’s `fdu.report` returned both roots |
| Cleanup | `release/v0.5.0` deleted from origin; the repository description now says “Fast du replacement”, matching the tagline |

As the guide asks: `Announce on GitHub` succeeded, all twelve assets were verified, and
`make release-demo` attached the declared video.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
