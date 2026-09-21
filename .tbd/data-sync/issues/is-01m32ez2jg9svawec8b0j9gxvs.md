---
type: is
id: is-01m32ez2jg9svawec8b0j9gxvs
title: "Manual correctness runbook: warm-versus-cold on a large real tree with every file kind (platform-neutral)"
kind: feature
status: closed
priority: 1
version: 6
labels:
  - macos
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
child_order_hints:
  - is-01m32f50wdffzswdhb754scda6
created_at: 2026-09-21T17:07:07.984Z
updated_at: 2026-09-21T17:22:35.161Z
---
A standing manual process: install a large real tree (many directories, deep nesting, real package layouts), run every request cold and warm, and confirm the answers match. Complements the automated path-independence harness rather than duplicating it, because the two fail differently.

## Why it covers something the harness cannot

The harness builds a synthetic fixture (`tests/path_independence/fixture.py`). Verified 2026-09-21, it contains symlinks, a permission-denied directory, hidden and empty entries, a binary, prose and a code file — and ZERO hard links (`grep -c 'os.link'` returns 0). No FIFOs, sockets, device nodes, sparse files, setuid bits, non-UTF8 names, case-colliding names, very long paths, or files above a few KiB.

A real tree after a full install has scale, depth, odd names and file kinds nobody enumerated when writing a fixture. That is the point: the fixture tests what its author thought of.

Scale is already a known failure mode the fixture cannot reach: fdu-6o5o (a ~/Library scan SIGKILLed at 137 through growth the control cap does not govern) and fdu-if7o (default --view summary retaining the full index, peak RSS 13 MiB to 128 MiB).

## The property that makes it worth running, and without which it is theatre

It must record that the cache WAS USED, not only that answers match.

Verified in review on 2026-09-21: with snapshot serving hard-wired to refuse, the harness subset ran 884 cases with ZERO failures — a completely dead cache is indistinguishable from a correct one when you only compare warm against cold, because a miss just scans cold and matches. A with-and-without-caching comparison has exactly that blind spot by construction: turn caching off on both sides and every case passes.

So each case records the answer AND the mechanism: `source` (cold_scan / warm / cache_only), hit counts, and files opened (`FDU_COUNTERS=1` costs a variable rather than a rebuild). A run where cache-only refuses more than the recording fails, even if every answer matched.

## Hard links need a decision before they need a runbook

fdu-579b (hardlink attribution policy that survives incremental updates) is still open, and the engine has no hardlink handling today. Until the policy is chosen — count once per inode, once per link, or per device — "the same before and after" is not yet the right oracle for them, because both answers could be consistently wrong. Pin the decided policy in the runbook, then test against it.

## Shape

Extend the existing installed-CLI QA work rather than starting fresh: `scripts/run_installed_cli_qa.py` and `docs/project/reports/report-2026-09-18-cli-installed-qa.md`.

Tree builder should cover, per platform and skipping what the platform refuses: regular files across sizes including zero and >4 GiB sparse; hard links, including two names in different directories and one crossing a filter boundary; symlinks — relative, absolute, to directories, dangling, and a cycle; FIFOs, sockets, device nodes; setuid/setgid/sticky; permission-denied directories and files; non-UTF8 and case-colliding names; paths near the platform limit; a bind mount or second filesystem for --one-file-system.

Run each request cold, then warm, then cache-only, then after each kind of mutation, on both the command line and Python, and diff. Record the regime — platform, bare metal or virtualized, filesystem, cache state — because that decides what the result is evidence about.

## Notes

## macOS section: ~/Library under TCC

Added 2026-09-21 at the user's direction. This half cannot be run from the Linux container and is genuinely manual, on a real Mac. It is also where an open bug already lives: fdu-6o5o records a SIGKILL (137) on `fdu ~/Library -d 2 -n 30 --sort size --min-size 300M`, and its last note says the unbounded full-Library scan has still not been retested. The 2026-09-18 installed-CLI QA got bounded scans through cleanly (0.08s / 23 MiB at --scan-depth=1; 0.23s / 27 MiB at --scan-depth=2, exit 2 with 26 TCC warnings).

### Run it in both permission states, because they are different tests

Terminal WITHOUT Full Disk Access and WITH it. Under TCC the restricted subtrees (Mail, Messages, Safari, Cookies, Application Support/com.apple.TCC, and the ~1012 sandbox dirs under Containers) return EPERM, which is not EACCES and not ENOENT. "Handled appropriately" means: a clean refusal, exit 2, warnings on stderr, `complete: false` with the errors named in machine formats — never a crash, never a silently smaller total.

### The correctness property, and the trap in checking it

The principle is that a partial answer contains exactly the part of the tree the run verified, equals a cold answer over that part, and names what is missing — and that retained facts under an unverified subtree are never served. TCC denials are how you get unverified subtrees on a real machine, so ~/Library is the natural place to exercise this.

Trap: the `unverified-subtree` class still has 128 registered violations on Linux and macOS, pending fdu-szll. So warm-versus-cold on ~/Library is EXPECTED to differ today in registered ways. Compare against the registry, not against naive equality, or the run reports known-open issues as new findings.

### macOS-specific hazards the Linux fixture cannot reach

- Do not chase Containers as an fdu performance bug. fdu-6o5o established that `du -sh ~/Library/Containers` times out too: that subtree is hostile to every tool because of per-container TCC checks.
- iCloud Drive dataless files under ~/Library/Mobile Documents. Reading one can trigger a download or return EIO. A metadata walk must not materialize them, and `--analyze` over that path must be checked deliberately: a disk-usage tool that silently downloads gigabytes is a correctness and a safety problem.
- APFS: case-insensitive but case-preserving by default, so case-colliding names collapse where they stay distinct on Linux; firmlinks into /System/Volumes/Data; snapshots; cloned files, which make `allocated` diverge from `apparent` in ways ext4 will not show; resource forks and extended attributes; .DS_Store.
- The macOS walk is separate code. `crates/fdu-core/src/scan.rs`, `lib.rs`, `platform_tuning.rs` and `counters/process.rs` all carry `cfg(target_os = "macos")` paths, so a Linux run validates none of it, and `make cross-lint` only checks that it compiles.

### Memory, which is the open question

fdu-6o5o wants an RSS slope, not an anecdote: peak RSS at three fixture sizes on a quiet host, both binaries, to establish whether growth is unbounded with entry count on ~/Library-shaped trees (deep, wide, many small files). Record disk pressure — the original reporter's host was at 95-99% full, which is a confound. Keep TCC-induced slowness out of the attribution.
