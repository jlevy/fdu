# 0.4.0 Stability Pass and Release Record — 2026-10-09

This is the full record of the 0.4.0 release checklist’s stability pass (step 2 of the
[release process](../guides/release-process.md#the-steps)), written by
`make release-stability`, and the [record of the release](#release-record) it cleared,
which step 12 asks for.
The summaries belong beside the procedures, in the
[QA playbook’s Current Status](../../../tests/qa/cli-installed-e2e.qa.md) and the
[correctness runbook’s Last Recorded Run](../guides/correctness-runbook.md#last-recorded-run);
this report keeps every table they summarize.
fdu 0.4.0 was published on 2026-10-09 at 07:26:50 UTC.

## Verdict

The pass tested commit `f405067db77e36498ea7a85d4186de563cb4b4c5`, tree
`2c728b23c19f3419093e8b4ee1fd6ba83005987f`. A release commit with that tree inherits
every result here by tree identity; check it with `git rev-parse <commit>^{tree}` before
tagging. `f405067d` is the head of #189’s branch, and the release commit, `c041ed1c`, is
the merge of #189 into `main`; `git rev-parse` gives both the tree `2c728b23`, so the
release commit inherits these results.
Nothing failed, and nothing was skipped.
One optional check did not run, and the harness does not count it as a skip: Phase 4’s
bounded analyze of a medium-tree subdirectory and its reuse.
`FDU_QA_MEDIUM_ANALYZE` was unset, and the Linux tree keeps its documentation in
`Documentation/`, not the `docs/` the harness defaults to.
Content analysis did run on that tree, under the pty probe’s `--analyze all`, but not as
a harness check.

| Gate or phase | Verdict |
| --- | --- |
| `make check` | Passed: `make check` exited 0 |
| `make cross-lint` | Passed: `make cross-lint` exited 0: linted x86_64-apple-darwin, x86_64-pc-windows-msvc, i686-unknown-linux-gnu, x86_64-unknown-linux-musl, aarch64-unknown-linux-gnu |
| `make semver-check` | Passed: `make semver-check` exited 0: fdu-core 0.4.0: starts a new compatibility series; nothing to check; fdu 0.4.0: starts a new compatibility series; nothing to check |
| `make release-rehearse` | Passed: `make release-rehearse` exited 0 |
| Candidate | Passed: `fdu --version` printed `fdu 0.4.0-dev+gf405067db`, from `fdu-0.4.0-cp312-abi3-macosx_11_0_arm64.whl` |
| QA harness, phases 1–5 and 8 | Passed: 47 checks: 47 ok, 0 warn, 0 fail |
| QA phase 6, terminal progress | Pending: a person has not watched a real window, and Windows has not run. Terminal tests passed: 3 tests ran against the candidate; exit status 0; pty probe passed: 38 of 38 checks passed |
| QA phase 7, peer agreement self-test | Passed: 13 readings agree exactly; 0 failed; exit status 0 |
| QA phase 7, peer agreement on real trees | Passed: 26 of 26 readings on 2 trees agree exactly, 0 `UNEXPLAINED`; Every reading is explained; exit status 0 |
| Correctness, refusal tree as an unprivileged user | Passed: 26 of 26 cases partial and withheld; 0 answer mismatches, 0 mechanism failures, 0 stale reference instants; exit status 0 |
| Correctness, served tree | Passed: 26 of 26 served `cache_only` and labelled `stale`; 0 answer mismatches, 0 mechanism failures, 0 stale reference instants; exit status 0 |
| Correctness, cross-warm matrix | Passed: 72 of 72 pairs matched the cold answer; 0 violations; exit status 0 |
| Correctness, break: no snapshot stored | Passed: `warm_cold.py` exited 1, 26 of 26 cases caught (17 `NO-SNAPSHOT`, 9 `NOT-WARM`); `cross_warm.py` exited 1, 17 of 17 same-analyzer pairs `NOT-WARM` |
| Correctness, break: partial answer stored | Passed: `warm_cold.py --refusals-only` exited 1, 26 of 26 cases `PARTIAL-STORED` |

## Regime

- **Host.** Bare metal: an Apple M1 Pro with 10 CPUs and 32 GiB of memory, macOS 26.5.2
  (Darwin arm64, kernel 25.5.0), on its internal APFS SSD, running as an unprivileged
  user. It was not quiet: other agents’ jobs ran throughout the pass, with a load average
  of roughly 10 to 18, swap nearly full, and 1 to 4 GiB of disk free.
  The timings in this report are single runs on that loaded host, not claims.
- **Declared preconditions.** None.
- **Candidate.** The `fdu-0.4.0-cp312-abi3-macosx_11_0_arm64.whl` wheel built from the
  commit, installed with `uv tool install` into an isolated tool directory;
  `fdu --version` printed `fdu 0.4.0-dev+gf405067db`.
- **Peers.** du (GNU coreutils) 9.9, Dust 1.2.4, pdu 0.24.0, dua 2.41.1, diskus 0.9.0.
- **Trees.** small: `~/wrk/github/fdu`; medium: `linux-v7.3-rc6`; large:
  `~/Library/Application Support`; progress: `~/wrk/github`; progress analyze:
  `linux-v7.3-rc6`; peer 1: `~/.rustup`; peer 2: `linux-v7.3-rc6`. `linux-v7.3-rc6`
  labels a Linux v7.3-rc6 source tree in a scratch directory.
- **Correctness trees.** tree: 14 kinds present, 3 absent (chardev, blockdev,
  non-utf8-name); served: 13 kinds present, 3 absent (chardev, blockdev, non-utf8-name).
  An unprivileged user cannot make device nodes, and APFS refuses a name that is not
  valid UTF-8. This volume is case-insensitive, so the builder’s two case-colliding
  names became one file.
- **Tooling.** The QA and correctness scripts at `f405067db`.

## Reproduce

```shell
export COMMIT=f405067db77e36498ea7a85d4186de563cb4b4c5
export RELEASE=<a directory outside any checkout>
export FDU_QA_SMALL='~/wrk/github/fdu'
export FDU_QA_MEDIUM=linux-v7.3-rc6
export FDU_QA_LARGE='~/Library/Application Support'
export FDU_QA_PROGRESS_TREE='~/wrk/github'
export FDU_QA_PROGRESS_ANALYZE=linux-v7.3-rc6
export FDU_QA_PEER_TREES='~/.rustup:linux-v7.3-rc6'
make release-stability
```

`linux-v7.3-rc6` stands for the path of that tree.
This pass ran the driver with
`--target-dir <a target directory of its own> --min-free-gb 2 --label <that path>=linux-v7.3-rc6`,
which `make release-stability` passes through `ARGS`; the lower free-space floor was for
this host’s disk.

## Release Record

The release checklist’s [step 12](../guides/release-process.md#the-steps) asks for this
record. Every step ran on 2026-10-09; times are UTC.

| What | Record |
| --- | --- |
| Release commit | `c041ed1c8ac5c588ac4fe964ab95bc14f1e5b0ac`, the merge of #189, tree `2c728b23`, the tree this pass tested |
| Tag | `v0.4.0`, annotated and unsigned: tag object `81eb6a96515dfc1b91ff6eb663b746c7a637d2f6`, message `fdu 0.4.0`, naming the release commit, created 07:07:42. GitHub reports its verification as `unsigned` |
| Rehearsal | [Run 37893372604](https://github.com/jlevy/fdu/actions/runs/37893372604) on `release/v0.4.0` at the release commit, every job a success but the three on the publishing path (the environment check, `Publish to crates.io and PyPI`, and `Announce on GitHub`), which a rehearsal skips. `make release-candidate` downloaded and verified its eight files, and the local `notes.md` from `make release-body` was byte-identical to the rehearsal’s announcement notes |
| Demo video | `make release-demo` created the draft release and attached `fdu-demo.mp4`, 3,784,222 bytes, SHA-256 `31e720b35f96fdc060fa5a33bfa7f62856b3bd8e2160bf39b48724b256a604c5`, as `docs/media/fdu-demo.json` declares. Its first attempt failed because GitHub’s release listing did not yet show the draft it had just created (`fdu-x81v`); a rerun reused that draft |
| Publishing run | [Run 37897343884](https://github.com/jlevy/fdu/actions/runs/37897343884), dispatched on `v0.4.0` at the release commit, 07:08 to 07:26, first attempt: every job a success, `Publish to crates.io and PyPI` and `Announce on GitHub` among them. The agent approved the `release` environment on the maintainer’s explicit go-ahead for this release |
| GitHub release | [v0.4.0](https://github.com/jlevy/fdu/releases/tag/v0.4.0), titled `fdu 0.4.0`, published 07:26:50, immutable, with 12 assets: the two crates, the source distribution, five wheels, `SHA256SUMS`, `release-manifest.json`, `registry-state.json`, and `fdu-demo.mp4` |
| `make release-published` | crates.io’s `fdu-core` and `fdu` and PyPI’s `fdu` each hold exactly the publishing run’s files |
| `make release-announced ARGS=--cargo` | The release body matches `notes.md`; all 12 assets match, the eleven files and the declared demo; docs.rs built `fdu-core` and `fdu`; `uvx fdu@0.4.0` and `uvx fdu@latest` print `fdu 0.4.0`; `cargo install --locked fdu` builds a binary that prints `fdu 0.4.0` |
| Homepage | The README’s GIF is served from `raw/main` as `image/gif`, 1,451,807 bytes, SHA-256 `39a54b05f61af4ea1792c7879d65f0646f49620f00a83a5f4fea9df18a98de8b`, and renders and animates; `releases/latest/download/fdu-demo.mp4`, which it links, returns the declared MP4 byte for byte |
| Clean up | `make release-cleanup` deleted `release/v0.4.0` from origin |

0.4.0 is the first release to declare the demo video, so as step 12 asks:
`Announce on GitHub` succeeded, `make release-demo` attached the declared demo, and
`make release-announced` verified all twelve assets.

Not recorded here: the three checks
[After Publishing](../guides/release-process.md#after-publishing) leaves to the eye, the
crates.io and PyPI pages and `fdu --install-skill`; and Phase 6’s watched window, which
stays pending as it did for 0.3.0.

Follow-ups live in two places: the 0.4.1 epic, `fdu-chcx`, which holds the fixes for
totals that do not match, the 0.4.0 review residue, and the report path’s recorded
non-regression against 0.3.0 (`fdu-vl8a`); and `fdu-ep8f`, which fits human reports to
narrower terminals. `fdu-x81v` covers the listing lag that both `Announce on GitHub` and
`make release-demo` have now hit.

## Full Tables

Paths are replaced by the labels above.

### Installed-CLI harness (fdu 0.4.0-dev+gf405067db)

| Phase | Name | Verdict | Exit | Real s | RSS MiB | Note |
| --- | --- | --- | ---: | ---: | ---: | --- |
| sanity | help | ok | 0 | 0.020 | 18.2 |  |
| sanity | version | ok | 0 | 0.020 | 18.1 |  |
| sanity | docs | ok | 0 | 0.020 | 18.0 |  |
| sanity | skill | ok | 0 | 0.020 | 18.2 |  |
| views | tree-cold | ok | 0 | 0.320 | 26.0 | cold scan; fresh=0; cached=0 |
| views | tree-warm | ok | 0 | 0.220 | 24.5 | cold scan; fresh=0; cached=0 |
| views | summary | ok | 0 | 0.190 | 25.0 | cold scan; fresh=0; cached=0 |
| views | languages | ok | 0 | 0.200 | 47.1 | cold scan; fresh=0; cached=0 |
| views | families | ok | 0 | 0.220 | 46.7 | cold scan; fresh=0; cached=0 |
| views | types | ok | 0 | 0.230 | 46.6 | cold scan; fresh=0; cached=0 |
| views | extensions | ok | 0 | 0.200 | 34.8 | cold scan; fresh=0; cached=0 |
| views | documents | ok | 0 | 2.950 | 111.1 | cold scan; fresh=39853; cached=0 |
| views | recent | ok | 0 | 0.210 | 50.2 | cold scan; fresh=0; cached=0 |
| views | largest | ok | 0 | 0.260 | 50.4 | cold scan; fresh=0; cached=0 |
| views | files | ok | 0 | 0.340 | 86.6 | cold scan; fresh=0; cached=0 |
| views | full | ok | 0 | 0.310 | 70.2 | cold scan; fresh=0; cached=0 |
| views | combo-kinds | ok | 0 | 0.170 | 48.1 | cold scan; fresh=0; cached=0 |
| views | exclude-ignored-summary | ok | 0 | 0.140 | 34.3 | cold scan; fresh=0; cached=0 |
| views | depth-limit-tree | ok | 0 | 0.120 | 24.7 | cold scan; fresh=0; cached=0 |
| views | scan-depth-1-summary | ok | 0 | 0.020 | 19.0 | cold scan; fresh=0; cached=0 |
| views | json-summary | ok | 0 | 0.120 | 24.9 |  |
| views | yaml-summary | ok | 0 | 0.120 | 24.5 |  |
| cache-analyze | off-code-1 | ok | 0 | 1.520 | 81.8 | cold scan; fresh=39853; cached=0 |
| cache-analyze | off-code-2 | ok | 0 | 1.600 | 84.2 | cold scan; fresh=39853; cached=0 |
| cache-analyze | off-lines-1 | ok | 0 | 1.510 | 82.5 | cold scan; fresh=39853; cached=0 |
| cache-analyze | off-lines-2 | ok | 0 | 1.270 | 83.9 | cold scan; fresh=39853; cached=0 |
| cache-analyze | off-cache-status | ok | 0 | 0.020 | 18.2 |  |
| cache-analyze | on-code-1 | ok | 0 | 1.590 | 107.0 | cold scan; fresh=39853; cached=0 |
| cache-analyze | on-code-2 | ok | 0 | 0.380 | 107.9 | warm revalidation; fresh=0; cached=39853 |
| cache-analyze | on-lines-1 | ok | 0 | 1.620 | 99.5 | warm revalidation; fresh=39853; cached=0 |
| cache-analyze | on-lines-2 | ok | 0 | 0.360 | 106.0 | warm revalidation; fresh=0; cached=39853 |
| cache-analyze | on-cache-status | ok | 0 | 0.020 | 18.3 |  |
| analyze-extra | analyze-words | ok | 0 | 1.520 | 106.3 | warm revalidation; fresh=39853; cached=0 |
| analyze-extra | analyze-all | ok | 0 | 1.630 | 117.5 | warm revalidation; fresh=39853; cached=0 |
| analyze-extra | json-analyze-code | ok | 0 | 1.650 | 106.9 | physical_lines=3908779 |
| analyze-extra | yaml-analyze-lines | ok | 0 | 1.620 | 88.7 |  |
| analyze-extra | text-analyze-code-summary | ok | 0 | 1.450 | 89.2 | warm revalidation; fresh=39853; cached=0 |
| watch | watch-sigint | ok | -2 | 3.024 |  | SIGINT after file create; elapsed=3.02s |
| medium | med-tree-cold | ok | 0 | 0.430 | 24.2 | cold scan; fresh=0; cached=0 |
| medium | med-tree-warm | ok | 0 | 0.250 | 24.5 | cold scan; fresh=0; cached=0 |
| medium | med-summary | ok | 0 | 0.260 | 23.3 | cold scan; fresh=0; cached=0 |
| medium | med-languages | ok | 0 | 0.320 | 67.9 | cold scan; fresh=0; cached=0 |
| medium | med-combo-kinds | ok | 0 | 0.350 | 68.5 | cold scan; fresh=0; cached=0 |
| medium | med-recent | ok | 0 | 0.310 | 75.2 | cold scan; fresh=0; cached=0 |
| medium | med-json-summary | ok | 0 | 0.260 | 23.5 |  |
| large | large-summary-depth1 | ok | 0 | 0.070 | 19.1 | cold scan; fresh=0; cached=0 |
| large | large-summary-depth2 | ok | 2 | 0.140 | 20.0 | cold scan; fresh=0; cached=0 |

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

#### `~/.rustup`

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
| dua | allocated | 3.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| dua | apparent | 3.5 GiB | +2.6 MiB | +2.6 MiB | directories +2.6 MiB | agrees exactly | none | 0.1 s |
| diskus | allocated | 3.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| diskus | apparent | 3.5 GiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| BSD du | allocated | 3.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
4\.

#### `linux-v7.3-rc6`

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
| dua | allocated | 1.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.4 s |
| dua | apparent | 1.5 GiB | +3.5 MiB | +3.5 MiB | symbolic links +3.1 KiB, directories +3.5 MiB | agrees exactly | none | 0.2 s |
| diskus | allocated | 1.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |
| diskus | apparent | 1.5 GiB | +3.1 KiB | +3.1 KiB | symbolic links +3.1 KiB | agrees exactly | none | 0.1 s |
| BSD du | allocated | 1.7 GiB | 0 B | 0 B | — | agrees exactly | none | 0.3 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
24\.

Every reading is explained.

### Phase 6 pty probe

```
fdu: <work>/bin/fdu
slow tree: ~/wrk/github
small tree: ~/wrk/github/fdu
analyze tree: linux-v7.3-rc6

ok   slow scan exits 0 (3.4 s)
ok   slow scan draws frames (29 frames)
ok   first frame after about half a second (0.52 s)
ok   a frame says Scanning
     phases seen: ['Indexing', 'Scanning']
ok   the last frame is erased before the report
ok   the report follows the erase, with no frame left in it
ok   every frame fits 100 columns (at most 99)
ok   no frame contains a newline
ok   file counts climb ([74260] to [701105])
ok   the counts hold their column while Scanning ([40])
     sample frame: ⠼ ~/wrk/github  Scanning        394,457 files ·    49,467 dirs ·   12 GiB · 2.0 s
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
ok   60 columns: 81 frames fit, widest 59 ('⠋ /priva…linux  Summarizing  95,938 files · 1.7 GiB · 9.1 s')
ok   40 columns: 78 frames fit, widest 13 ('⠧ Summarizing')
ok   30 columns: 72 frames fit, widest 13 ('⠙ Summarizing')
ok   20 columns: 73 frames fit, widest 13 ('⠙ Summarizing')
ok   19 columns: 73 frames fit, spinner and phase word only ('⠹ Summarizing')
ok   12 columns: 73 frames fit, spinner and phase word only ('⠋ Analyzing')
ok   narrowing mid-run shrinks every later frame without wrapping
ok   --analyze all shows Analyzing with a percentage ([6] to [98])
ok   the percentage climbs
ok   the analyze indicator is erased before the report

38 of 38 checks passed
```

### Terminal tests

```
...
----------------------------------------------------------------------
Ran 3 tests in 23.903s

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
