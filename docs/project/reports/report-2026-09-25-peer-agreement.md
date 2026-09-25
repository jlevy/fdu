# Peer Agreement on Real Trees — 2026-09-25

The first run of the installed-CLI playbook’s
[peer-agreement phase](../../../tests/qa/cli-installed-e2e.qa.md#phase-7-peer-agreement-on-real-trees):
fdu’s totals on four real trees, compared with GNU du, dust, pdu, dua, diskus, and the
system du, with every difference given a cause.
Replace the tables when revising; keep the playbook’s procedure.

## Run Identity

| Field | Value |
| --- | --- |
| fdu | `fdu 0.1.0-dev+g963d48520` (main after #125), installed from its release wheel |
| Host | macOS, Apple silicon, APFS; a loaded desktop (load average 30–150 during the runs) |
| Peers | GNU du 9.9 (`gdu`), dust 1.2.4, pdu 0.24.0, dua 2.41.1, diskus 0.9.0, BSD du |
| Command | `python3 scripts/qa_peer_agreement.py . ~/.rustup /Applications ~/Library` |
| Trees | this repository, `~/.rustup`, `/Applications`, `~/Library` |

## Verdict

**fdu agrees with every tool on every tree, and every difference has a named cause.**

- On the three quiet trees, fdu’s allocated and apparent totals equal GNU du
  `--count-links`, pdu, and (after hard links) du, dust, dua, and diskus **to the
  byte**, as does every top-level directory.
  The only differences are the three causes below.
- On `~/Library`, which moved by 212 MiB during the 21-minute comparison, every tool
  falls within the range of the two fdu readings taken around it, after the same causes.
  fdu could not list 153 directories (macOS privacy protection), exactly as many as GNU
  du reports denied.

## Findings

1. **Allocated is the right default, and fdu’s agrees with du and dust.** `~/Library`
   holds 72.3 GiB on disk but 8.1 TiB apparent: OrbStack’s virtual-machine disk
   (`Group Containers/…dev.orbstack/data/data.img`) is 8 TiB apparent and 39 MiB
   allocated, and Docker’s is 64 GB and 313 MiB. du, dust, dua, and diskus default to
   allocated too, and agree.
   The interactive progress line counted apparent bytes and showed 8.1 TiB on this tree;
   #125 fixed that, so the line and the performance footer now count what the answer
   counts.
2. **`/Applications` is larger apparent than allocated** (48.8 GiB against 43.9 GiB)
   because APFS compresses application files.
   Every tool agrees on both figures.
3. **GNU du and pdu skip directories on macOS when a read is interrupted.** Both report
   `Interrupted system call` and leave that directory’s subtree out.
   In the first run this cost GNU du `--count-links` 1.4 GiB in `~/Library/Containers`
   while every other tool agreed; a rerun of `Containers` matched fdu to the byte.
   fdu’s fast macOS reader declines on any failure and its portable reader reads the
   directory again; it reported no such failure in any run.
   The script now counts each tool’s errors and marks a reading that falls short after
   interrupted reads.
4. **Three named differences, each measured from the tree itself:**
   - *Hard links*: fdu, `du -l`, and pdu count a hard-linked file once per path; du,
     dust, dua, and diskus once per inode.
     `~/Library` has 119 shared inodes, which per-path counting adds 22.6 MiB to.
   - *Symbolic links*: du and diskus count a link’s target text as apparent size; fdu
     counts regular files only.
     1.2 KiB in this repository, 308.5 KiB in `/Applications`, 1.1 MiB in `~/Library`;
     links occupy no blocks on APFS.
   - *Directory sizes*: dust `-s`, pdu apparent, and dua `-A` add each directory’s own
     size, 1–36 MiB here.
5. **A live tree needs bracketing.** `~/Library` shrank by about 400 MiB partway through
   one run and partly recovered, so a comparison with one early fdu reading blamed the
   tools measured in the dip.
   fdu now runs before each tool and once at the end, and each tool is judged against
   the two readings around it.

The times in the tables are single runs on a loaded machine, not a benchmark; see
[the performance loop](../guides/performance-loop.md) for claims about speed.

## Tables

### This Repository

fdu: allocated 550.1 MiB, apparent 493.4 MiB; across its 14 readings, one before each
tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.
Hard links: 0 shared inodes, which counting per path adds 0 B allocated
to. Symbolic links: 31, 1.2 KiB apparent, 0 B allocated.
Directories that could not be listed: 0.

| Tool | Metric | Counts | Total | Δ vs fdu | Expected Δ | fdu moved | Verdict | Errors | Time |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- | --- | ---: |
| GNU du -l | allocated | per path | 550.1 MiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.5 s |
| GNU du | allocated | per inode | 550.1 MiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.2 s |
| GNU du -l | apparent | per path | 493.4 MiB | 1.2 KiB | 1.2 KiB | 0 B | agrees exactly after symbolic links | none | 0.2 s |
| GNU du | apparent | per inode | 493.4 MiB | 1.2 KiB | 1.2 KiB | 0 B | agrees exactly after symbolic links | none | 0.2 s |
| dust | allocated | per inode | 550.1 MiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.1 s |
| dust | apparent | per path + directory sizes | 494.3 MiB | 986.8 KiB | 0 B | 0 B | adds directory sizes | none | 0.1 s |
| pdu | allocated | per path | 550.1 MiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.1 s |
| pdu | apparent | per path + directory sizes | 494.3 MiB | 986.8 KiB | 0 B | 0 B | adds directory sizes | none | 0.1 s |
| dua | allocated | per inode | 550.1 MiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.1 s |
| dua | apparent | per inode + directory sizes | 494.3 MiB | 985.6 KiB | 0 B | 0 B | adds directory sizes | none | 0.1 s |
| diskus | allocated | per inode | 550.1 MiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.1 s |
| diskus | apparent | per inode | 493.4 MiB | 1.2 KiB | 1.2 KiB | 0 B | agrees exactly after symbolic links | none | 0.1 s |
| BSD du | allocated | per inode | 550.1 MiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.2 s |

Top-level directories whose allocated size differs from GNU du -l by more than 0.1% and
1 MiB: 0.

### `~/.rustup`

fdu: allocated 3.2 GiB, apparent 3.0 GiB; across its 14 readings, one before each tool
and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.
Hard links: 0 shared inodes, which counting per path adds 0 B allocated
to. Symbolic links: 0, 0 B apparent, 0 B allocated.
Directories that could not be listed: 0.

| Tool | Metric | Counts | Total | Δ vs fdu | Expected Δ | fdu moved | Verdict | Errors | Time |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- | --- | ---: |
| GNU du -l | allocated | per path | 3.2 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.8 s |
| GNU du | allocated | per inode | 3.2 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.3 s |
| GNU du -l | apparent | per path | 3.0 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.3 s |
| GNU du | apparent | per inode | 3.0 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.4 s |
| dust | allocated | per inode | 3.2 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.2 s |
| dust | apparent | per path + directory sizes | 3.0 GiB | 2.6 MiB | 0 B | 0 B | adds directory sizes | none | 0.1 s |
| pdu | allocated | per path | 3.2 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.2 s |
| pdu | apparent | per path + directory sizes | 3.0 GiB | 2.6 MiB | 0 B | 0 B | adds directory sizes | none | 0.2 s |
| dua | allocated | per inode | 3.2 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.2 s |
| dua | apparent | per inode + directory sizes | 3.0 GiB | 2.6 MiB | 0 B | 0 B | adds directory sizes | none | 0.2 s |
| diskus | allocated | per inode | 3.2 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.1 s |
| diskus | apparent | per inode | 3.0 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.1 s |
| BSD du | allocated | per inode | 3.2 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 0.2 s |

Top-level directories whose allocated size differs from GNU du -l by more than 0.1% and
1 MiB: 0.

### `/Applications`

fdu: allocated 43.9 GiB, apparent 48.8 GiB; across its 14 readings, one before each tool
and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.
Hard links: 0 shared inodes, which counting per path adds 0 B allocated
to. Symbolic links: 16,049, 308.5 KiB apparent, 0 B allocated.
Directories that could not be listed: 0.

| Tool | Metric | Counts | Total | Δ vs fdu | Expected Δ | fdu moved | Verdict | Errors | Time |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- | --- | ---: |
| GNU du -l | allocated | per path | 43.9 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 12.3 s |
| GNU du | allocated | per inode | 43.9 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 12.4 s |
| GNU du -l | apparent | per path | 48.8 GiB | 308.5 KiB | 308.5 KiB | 0 B | agrees exactly after symbolic links | none | 11.7 s |
| GNU du | apparent | per inode | 48.8 GiB | 308.5 KiB | 308.5 KiB | 0 B | agrees exactly after symbolic links | none | 12.2 s |
| dust | allocated | per inode | 43.9 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 3.4 s |
| dust | apparent | per path + directory sizes | 48.8 GiB | 24.1 MiB | 0 B | 0 B | adds directory sizes | none | 3.4 s |
| pdu | allocated | per path | 43.9 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 3.7 s |
| pdu | apparent | per path + directory sizes | 48.8 GiB | 24.1 MiB | 0 B | 0 B | adds directory sizes | none | 3.2 s |
| dua | allocated | per inode | 43.9 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 3.5 s |
| dua | apparent | per inode + directory sizes | 48.8 GiB | 24.1 MiB | 0 B | 0 B | adds directory sizes | none | 3.5 s |
| diskus | allocated | per inode | 43.9 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 3.2 s |
| diskus | apparent | per inode | 48.8 GiB | 308.5 KiB | 308.5 KiB | 0 B | agrees exactly after symbolic links | none | 3.4 s |
| BSD du | allocated | per inode | 43.9 GiB | 0 B | 0 B | 0 B | agrees exactly | none | 6.3 s |

Top-level directories whose allocated size differs from GNU du -l by more than 0.1% and
1 MiB: 0.

### `~/Library`

fdu: allocated 72.3 GiB, apparent 8.1 TiB; across its 14 readings, one before each tool
and one at the end, the tree moved 212.2 MiB allocated and 211.9 MiB apparent.
fdu errors: 64 denied, 89 more, not detailed.
Hard links: 119 shared inodes, which counting per path adds 22.6 MiB allocated to.
Symbolic links: 36,755, 1.1 MiB apparent, 0 B allocated.
Directories that could not be listed: 153.

| Tool | Metric | Counts | Total | Δ vs fdu | Expected Δ | fdu moved | Verdict | Errors | Time |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- | --- | ---: |
| GNU du -l | allocated | per path | 72.3 GiB | 60.0 KiB | 0 B | 192.0 KiB | agrees within the tree’s movement | 153 denied, 12 interrupted | 107.4 s |
| GNU du | allocated | per inode | 72.3 GiB | 42.5 MiB | -22.6 MiB | 65.9 MiB | agrees within the tree’s movement after hard links | 153 denied, 5 interrupted | 66.9 s |
| GNU du -l | apparent | per path | 8.1 TiB | 1.2 MiB | 1.1 MiB | 127.2 KiB | agrees within the tree’s movement after symbolic links | 153 denied, 9 interrupted | 85.0 s |
| GNU du | apparent | per inode | 8.1 TiB | -234.3 MiB | -21.2 MiB | -211.9 MiB | agrees within the tree’s movement after hard links and symbolic links | 153 denied, 18 interrupted | 271.6 s |
| dust | allocated | per inode | 72.1 GiB | -22.6 MiB | -22.6 MiB | 0 B | agrees within the tree’s movement after hard links | 1 other | 26.5 s |
| dust | apparent | per path + directory sizes | 8.1 TiB | 36.4 MiB | 0 B | 45.0 KiB | adds directory sizes | 1 other | 26.4 s |
| pdu | allocated | per path | 72.1 GiB | -4.1 MiB | 0 B | 2.2 MiB | agrees within the tree’s movement | 153 denied, 37 interrupted | 32.2 s |
| pdu | apparent | per path + directory sizes | 8.1 TiB | 36.1 MiB | 0 B | 258.3 KiB | adds directory sizes | 153 denied, 28 interrupted | 28.5 s |
| dua | allocated | per inode | 72.1 GiB | -22.8 MiB | -22.6 MiB | -4.0 KiB | agrees within the tree’s movement after hard links | none | 19.1 s |
| dua | apparent | per inode + directory sizes | 8.1 TiB | 13.8 MiB | -22.3 MiB | -177.8 KiB | adds directory sizes | none | 20.0 s |
| diskus | allocated | per inode | 72.1 GiB | -31.1 MiB | -22.6 MiB | 252.0 KiB | agrees within the tree’s movement after hard links | 1 other | 16.8 s |
| diskus | apparent | per inode | 8.1 TiB | -22.4 MiB | -21.2 MiB | -93.7 KiB | agrees within the tree’s movement after hard links and symbolic links | 1 other | 20.1 s |
| BSD du | allocated | per inode | 72.1 GiB | -22.6 MiB | -22.6 MiB | 420.0 KiB | agrees within the tree’s movement after hard links | 153 denied | 69.3 s |

Top-level directories whose allocated size differs from GNU du -l by more than 0.1% and
1 MiB: 0.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
