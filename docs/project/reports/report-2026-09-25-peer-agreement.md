# Peer Agreement on Real Trees — 2026-09-25

The first run of the installed-CLI playbook’s
[peer-agreement phase](../../../tests/qa/cli-installed-e2e.qa.md#phase-7-peer-agreement-on-real-trees):
fdu’s totals on four real trees, compared with GNU du, dust, pdu, dua, diskus, and the
system du, with every difference measured from the tree.
Replace the tables when revising; keep the playbook’s procedure.

## Run Identity

| Field | Value |
| --- | --- |
| fdu | `fdu 0.1.0-dev+g963d48520` (main after #125), installed from its release wheel |
| Script | `scripts/qa_peer_agreement.py` at c521954a |
| Host | macOS, Apple silicon, APFS; a desktop in use |
| Peers | GNU du 9.9 (`gdu`), dust 1.2.4, pdu 0.24.0, dua 2.41.1, diskus 0.9.0, BSD du |
| Commands | `python3 scripts/qa_peer_agreement.py --self-test`, then `. ~/.rustup /Applications ~/Library --json FILE` |
| Trees | this repository, `~/.rustup`, `/Applications`, `~/Library` |

## Verdict

**Every reading is explained.** Both runs exited 0.

- The self-test tree, built with every case the counting models name, agrees exactly for
  all 13 tool readings.
  On APFS it tests the directory and symbolic-link terms only in apparent size, since
  those occupy no blocks.
- On the three quiet trees, every reading agrees exactly once its measured differences
  are added (39 readings), and every top-level directory’s allocated size equals GNU
  du’s. fdu’s allocated totals equal GNU du `--count-links` to the byte.
- `~/Library` moved 4.1 MiB allocated during its part of the run.
  Each tool falls within the fdu readings taken around it, or outside them by no more
  than fdu moved during or next to them, or short by exactly the folders it reported
  skipping, which were measured afterwards.
  fdu exited 2 (partial) with 153 errors, as many as GNU du reports denied and as many
  as the script’s own walk found unreadable (149 directories and 4 files).

## Findings

1. **Allocated is the right default, and fdu’s agrees with du and dust.** `~/Library`
   occupies 72.1 GiB but is 8.1 TiB apparent: OrbStack’s virtual-machine disk
   (`Group Containers/…dev.orbstack/data/data.img`) is 8 TiB apparent and 39.2 MiB
   allocated, and Docker’s is 59.6 GiB apparent and 313 MiB allocated.
   du, dust, dua, and diskus default to allocated too.
   The interactive progress line counted apparent bytes and showed 8.1 TiB on this tree;
   #125 fixed that, so the line and the performance footer now count what the answer
   counts.
2. **`/Applications` is larger apparent than allocated** (48.8 GiB against 43.9 GiB)
   because APFS compresses application files.
   Every tool agrees on both figures.
3. **Other tools skip folders on a live tree; fdu did not.** GNU du and pdu on macOS
   give up on a directory whose read is interrupted, and diskus on one that fails for a
   moment without saying why; each leaves that subtree out.
   In an earlier run diskus skipped a 743 MiB application container this way.
   dua reports only a count, so its skips cannot be named, and its reading is bounded by
   what the others skipped.
   The script measures each named folder afterwards, and every such tool fell short by
   exactly what it skipped.
   In an earlier, unrecorded run, GNU du `--count-links` came in 1.4 GiB short in
   `~/Library/Containers` while every other tool agreed; that run kept no stderr, but a
   rerun of `Containers` matched fdu to the byte while GNU du reported interrupted reads
   there, so an interrupted large directory is the most likely cause.
   fdu’s fast macOS reader declines on any failure and its portable reader reads the
   directory again; none of its 64 detailed errors was an interrupted read.
4. **The tools count differently in three ways, all measured from the tree:**
   - *Hard links*: fdu, `du -l`, pdu, and dust’s apparent size count a hard-linked file
     once per path; du, dua, diskus, and dust’s allocated size count it once.
     `~/Library` has 119 shared inodes, which per-path counting adds 22.6 MiB to.
   - *Symbolic links*: du, dust, pdu, and diskus count a link’s own size, its target
     text; dua does too, except for links directly inside the root.
     fdu counts regular files only.
     1.2 KiB in this repository, 308.5 KiB in `/Applications`, 1.1 MiB in `~/Library`;
     links occupy no blocks on APFS.
   - *Directory sizes*: dust `-s` and pdu’s apparent size add every directory’s own
     size, and dua every directory’s but the root’s: 1 to 35 MiB here.
5. **A checker for a live tree has to be hard to fool.** Two independent reviews broke
   earlier versions of the script: a tolerance taken as a share of the tree allowed
   gigabytes on `~/Library`’s 8 TiB apparent total, and one bad fdu reading could loosen
   every row. The script now takes its allowances only from fdu’s own movement around
   each tool, fails a lone disagreeing fdu reading on its own row, and fails on each of
   the reviews’ attacks.

The times in the tables are single runs on a machine in use, not a benchmark; see
[the performance loop](../guides/performance-loop.md) for claims about speed.

## Self-Test

A tree with a 10 KB file hard-linked twice (within and across directories); symbolic
links to a file, a directory, and a 1000-character target, directly inside the root and
deeper; an unreadable folder; a 1 GiB sparse file; and a name with spaces.
Every reading must agree exactly.

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

## Tables

### This Repository

fdu read 550.4 MiB allocated and 493.6 MiB apparent, exiting 0; across its 14 readings,
one before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 31 symbolic links, 1.2 KiB apparent; 3,464 directories, 987.0 KiB apparent;
directories that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 550.4 MiB | 0 B | 0 B | — | agrees exactly | none | 0.6 s |
| GNU du | allocated | 550.4 MiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |
| GNU du -l | apparent | 493.6 MiB | +1.2 KiB | +1.2 KiB | symbolic links +1.2 KiB | agrees exactly | none | 0.2 s |
| GNU du | apparent | 493.6 MiB | +1.2 KiB | +1.2 KiB | symbolic links +1.2 KiB | agrees exactly | none | 0.2 s |
| dust | allocated | 550.4 MiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| dust | apparent | 494.5 MiB | +988.2 KiB | +988.2 KiB | symbolic links +1.2 KiB, directories +987.0 KiB | agrees exactly | none | 0.1 s |
| pdu | allocated | 550.4 MiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| pdu | apparent | 494.5 MiB | +988.2 KiB | +988.2 KiB | symbolic links +1.2 KiB, directories +987.0 KiB | agrees exactly | none | 0.1 s |
| dua | allocated | 550.4 MiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| dua | apparent | 494.5 MiB | +987.0 KiB | +987.0 KiB | symbolic links +1.2 KiB, directories +985.8 KiB | agrees exactly | none | 0.1 s |
| diskus | allocated | 550.4 MiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| diskus | apparent | 493.6 MiB | +1.2 KiB | +1.2 KiB | symbolic links +1.2 KiB | agrees exactly | none | 0.1 s |
| BSD du | allocated | 550.4 MiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
15\.

### `~/.rustup`

fdu read 3.2 GiB allocated and 3.0 GiB apparent, exiting 0; across its 14 readings, one
before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 0 symbolic links, 0 B apparent; 3,420 directories, 2.6 MiB apparent; directories
that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 3.2 GiB | 0 B | 0 B | — | agrees exactly | none | 0.8 s |
| GNU du | allocated | 3.2 GiB | 0 B | 0 B | — | agrees exactly | none | 0.3 s |
| GNU du -l | apparent | 3.0 GiB | 0 B | 0 B | — | agrees exactly | none | 0.3 s |
| GNU du | apparent | 3.0 GiB | 0 B | 0 B | — | agrees exactly | none | 0.3 s |
| dust | allocated | 3.2 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |
| dust | apparent | 3.0 GiB | +2.6 MiB | +2.6 MiB | directories +2.6 MiB | agrees exactly | none | 0.1 s |
| pdu | allocated | 3.2 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |
| pdu | apparent | 3.0 GiB | +2.6 MiB | +2.6 MiB | directories +2.6 MiB | agrees exactly | none | 0.2 s |
| dua | allocated | 3.2 GiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| dua | apparent | 3.0 GiB | +2.6 MiB | +2.6 MiB | directories +2.6 MiB | agrees exactly | none | 0.1 s |
| diskus | allocated | 3.2 GiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| diskus | apparent | 3.0 GiB | 0 B | 0 B | — | agrees exactly | none | 0.1 s |
| BSD du | allocated | 3.2 GiB | 0 B | 0 B | — | agrees exactly | none | 0.2 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
4\.

### `/Applications`

fdu read 43.9 GiB allocated and 48.8 GiB apparent, exiting 0; across its 14 readings,
one before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 16,049 symbolic links, 308.5 KiB apparent; 83,169 directories, 23.8 MiB apparent;
directories that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 10.5 s |
| GNU du | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 10.3 s |
| GNU du -l | apparent | 48.8 GiB | +308.5 KiB | +308.5 KiB | symbolic links +308.5 KiB | agrees exactly | none | 10.1 s |
| GNU du | apparent | 48.8 GiB | +308.5 KiB | +308.5 KiB | symbolic links +308.5 KiB | agrees exactly | none | 10.0 s |
| dust | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 3.3 s |
| dust | apparent | 48.8 GiB | +24.1 MiB | +24.1 MiB | symbolic links +308.5 KiB, directories +23.8 MiB | agrees exactly | none | 3.3 s |
| pdu | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 3.3 s |
| pdu | apparent | 48.8 GiB | +24.1 MiB | +24.1 MiB | symbolic links +308.5 KiB, directories +23.8 MiB | agrees exactly | none | 3.5 s |
| dua | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 3.3 s |
| dua | apparent | 48.8 GiB | +24.1 MiB | +24.1 MiB | symbolic links +308.5 KiB, directories +23.8 MiB | agrees exactly | none | 3.3 s |
| diskus | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 3.5 s |
| diskus | apparent | 48.8 GiB | +308.5 KiB | +308.5 KiB | symbolic links +308.5 KiB | agrees exactly | none | 3.3 s |
| BSD du | allocated | 43.9 GiB | 0 B | 0 B | — | agrees exactly | none | 6.1 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of
164\.

### `~/Library`

fdu read 72.1 GiB allocated and 8.1 TiB apparent, exiting 2; across its 14 readings, one
before each tool and one at the end, the tree moved 4.1 MiB allocated and 4.0 MiB
apparent. fdu errors: 64 denied, 89 more, not detailed.

Measured from the tree: 119 hard-linked inodes, which per-path counting adds 22.6 MiB
allocated to; 36,756 symbolic links, 1.1 MiB apparent; 150,102 directories, 35.2 MiB
apparent; directories that could not be listed: 149 denied; entries that could not be
stat’d: 4.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 72.1 GiB | +4.0 KiB | 0 B | — | agrees within the tree’s movement | 153 denied, 2 interrupted | 50.8 s |
| GNU du | allocated | 72.0 GiB | -22.6 MiB | -22.6 MiB | hard links once -22.6 MiB | agrees within fdu’s nearby movement (52.0 KiB) | 153 denied, 3 interrupted | 55.8 s |
| GNU du -l | apparent | 8.1 TiB | +1.2 MiB | +1.1 MiB | symbolic links +1.1 MiB | agrees within the tree’s movement | 153 denied, 4 interrupted | 62.6 s |
| GNU du | apparent | 8.1 TiB | -21.2 MiB | -21.2 MiB | hard links once -22.3 MiB, symbolic links +1.1 MiB | agrees within the tree’s movement | 153 denied, 3 interrupted | 55.7 s |
| dust | allocated | 72.0 GiB | -23.0 MiB | -22.6 MiB | hard links once -22.6 MiB | agrees within the tree’s movement | 149 denied | 18.5 s |
| dust | apparent | 8.1 TiB | +36.3 MiB | +36.3 MiB | symbolic links +1.1 MiB, directories +35.2 MiB | agrees within the tree’s movement | 149 denied | 13.3 s |
| pdu | allocated | 72.1 GiB | -1.6 MiB | 0 B | — | short by what it skipped: 6 directories holding 1.6 MiB | 153 denied, 6 interrupted | 16.7 s |
| pdu | apparent | 8.1 TiB | +37.1 MiB | +36.3 MiB | symbolic links +1.1 MiB, directories +35.2 MiB | agrees within the tree’s movement | 153 denied, 13 interrupted | 16.9 s |
| dua | allocated | 72.0 GiB | -22.6 MiB | -22.6 MiB | hard links once -22.6 MiB | agrees within fdu’s nearby movement (1.6 MiB) | 161 unreadable, reason not given | 13.0 s |
| dua | apparent | 8.1 TiB | +12.5 MiB | +14.0 MiB | hard links once -22.3 MiB, symbolic links +1.1 MiB, directories +35.2 MiB | agrees within fdu’s nearby movement (4.0 MiB) | 159 unreadable, reason not given | 12.9 s |
| diskus | allocated | 72.0 GiB | -30.8 MiB | -22.6 MiB | hard links once -22.6 MiB | short by what it skipped: 53 directories holding 4.2 MiB | 206 unreadable, reason not given | 14.2 s |
| diskus | apparent | 8.1 TiB | -67.6 MiB | -21.2 MiB | hard links once -22.3 MiB, symbolic links +1.1 MiB | short by what it skipped: 49 directories holding 46.4 MiB, within fdu’s nearby movement (4.0 MiB) | 202 unreadable, reason not given | 13.4 s |
| BSD du | allocated | 72.0 GiB | -22.6 MiB | -22.6 MiB | hard links once -22.6 MiB | agrees within the tree’s movement | 153 denied | 31.9 s |

Folders a tool reported it could not read and the walk could list, counted rather than
named because their names identify the apps and accounts on this machine (mostly
temporary folders of the macOS photo conversion service, and application containers):
GNU du -l (allocated): 2, holding 0 B; GNU du (allocated): 3, holding 0 B; GNU du -l
(apparent): 4, holding 0 B; GNU du (apparent): 3, holding 0 B; pdu (allocated): 6,
holding 1.6 MiB; pdu (apparent): 13, holding 489.0 KiB; diskus (allocated): 53, holding
4.2 MiB; diskus (apparent): 49, holding 46.4 MiB.

Top-level directories whose allocated size does not agree with GNU du -l within fdu’s
readings around it, ± 52.0 KiB: 0 of 149.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
