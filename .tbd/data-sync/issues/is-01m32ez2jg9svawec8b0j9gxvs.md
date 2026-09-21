---
type: is
id: is-01m32ez2jg9svawec8b0j9gxvs
title: "Manual correctness runbook: warm-versus-cold on a large real tree with every file kind"
kind: feature
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T17:07:07.984Z
updated_at: 2026-09-21T17:07:07.984Z
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
