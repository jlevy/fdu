---
type: is
id: is-01m3n9n0rq6wm4w9ywwbnhcazp
title: "Differential and quiet speed run: fdu --analyze code vs tokei 15.0.0 and scc 4.1.0 on linux-v6.12"
kind: task
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
delegate: claude-code@vm
labels: []
dependencies:
  - type: blocks
    target: is-01m3n77fy60sdcdzzk9qmbzg8e
hold: null
hold_until: null
created_at: 2026-09-29T00:39:49.782Z
updated_at: 2026-09-29T19:48:28.885Z
started_at: 2026-09-29T18:57:44.265Z
closed_at: 2026-09-29T19:48:28.885Z
close_reason: "Differential and quiet speed run done on linux-v6.12 (both arms plus the cached row); harness contracts, brief, evidence, and README columns landed on #162 at 0fa5e017."
resolution: null
duplicate_of: null
---
Needed for the README matrix scc/tokei columns (fdu-dbn9) and to re-score accuracy after analyzer v3 (the 14/30 fixture score predates v3; tokei 15 and scc 4.1 never measured). Protocol from the SLOC survey (fdu-61ez; saved with the matrix draft): Arm A ignore rules off on an identical copied tree, Arm B each tool's .gitignore handling on; text output to /dev/null; one untimed JSON validation run per tool per arm; add fdu-code/scc/tokei contracts to explorations/benchmarks/realtree/compare_tools.py (3 warm-ups, 12 alternating pairs, quiet gate, wait4 RSS). Settle per-file disagreements with minimal reproductions. Install tokei via cargo --locked --version 15.0.0; scc from the release tarball verified against checksums.txt (host Go is too old).

## Notes

# Results, 2026-09-29 (done on jlevy/fdu#162, branch claude/readme-comparison-matrix)

Commits: 5e5b5198 (harness contracts), 06f3efa7 (brief and evidence), 0fa5e017 (README matrix); head 0fa5e017, pushed.

Setup: fdu fdu-final-ebc06c78 (engine of #161/#162, SHA-256 32b4724a4ca43317...); scc 4.1.0 from scc_Linux_x86_64.tar.gz, checksums.txt verified, binary SHA-256 4763d7437218d2d642e2f2a4f7f078e044aa67e0e15b90bab6c70d5fc0ab4022; tokei 15.0.0 via `cargo +1.97.1 install --locked tokei --version 15.0.0 --root /home/user/peers`, SHA-256 0b168d485c082d7e8bf37e934ba424b94c2ed9134440f9918c55b55af72c271c. Subject linux-v6.12 (adc21867); arm A on a tar copy without .git (verified identical in every path, type, size, mode, link target, and byte; deleted afterwards), arm B on the clone. 4-vCPU Firecracker ext4, quiet, 3 warm-ups + 12 adjacent pairs, 0 invalid timed samples in all three cells.

Harness: compare_tools.py contracts fdu-code-no-ignore, fdu-code-cached-no-ignore, fdu-code-gitignore, scc-no-ignore, scc-gitignore, tokei-no-ignore, tokei-gitignore; `measures` field refuses mixed comparisons; the text table's total row is parsed per sample (no stderr, zero exit, one answer per tool); cross-tool agreement is the separate untimed JSON validation. Tests in test_compare_tools.py (LineCountContractTests); make perf-test 380 OK.

Speed (median wall; paired change vs adjacent fdu [95%]):
- Arm A, ignore rules off: fdu 7.92 s (29.9 CPU-s, 126 MiB); scc 1.24 s -84% [-84.5, -83.8] (4.8 CPU-s, 230 MiB); tokei 1.87 s -76% [-76.8, -76.1] (7.2 CPU-s, 150 MiB).
- Arm B, own ignore handling: fdu 9.22 s (31.0 CPU-s, 157 MiB); scc 1.29 s -86% [-86.3, -85.6]; tokei 1.98 s -78% [-78.6, -78.3].
- Optional row, fdu repeated under the default cache policy (arm A tree): fdu 0.55 s (0.72 CPU-s); scc +127% [+123, +130]; tokei +237% [+227, +242].
- Not measured: scc with complexity on (the other optional row).

Accuracy (untimed JSON, arm A, 59,953 .c/.h files): all three agree on 59,766; fdu=scc 59,931; fdu=tokei 59,788; scc=tokei 59,766. Causes, each reproduced in a 1-5 line file: scc counts a form-feed-only line as code (22 files, +91 code); tokei opens a string at '"' (42 files, +4,672); tokei counts code between two block comments as comment (39 files, -1,426); tokei counts code after a multi-line comment close as comment (26 files, -30); a macro line splice after a close (55 files, -151) is a convention. 3 files (+9) unattributed. C-family totals (C+C++ sources and headers): fdu 26,104,345 / scc 26,104,489 / tokei 26,107,591 code. fdu labels 168 .h as C++ (fdu-0lo2; 165 match only 'namespace '). Whole tree: fdu 6 languages 26,312,547 code; scc 47 languages 29,725,820; tokei 50 languages 29,615,689.

Protocol corrections: fdu treats .git as ignored, so --ignored=exclude never walks it (the --exclude is redundant); 3 tracked files under tools/testing/selftests/arm64/tags/ match the kernel's `tags` ignore pattern and drop out of arm B in all three tools; scc 4.1.0 has 366 languages (368 is main).

New beads: fdu-xpwv (cold --analyze=code is 6.2x scc's CPU; exclude mode adds 16%), fdu-d0gc (modeline alias prefix match: 'mode: conf-colon' -> C).

Evidence: docs/project/research/evidence/sloc-tool-comparison-2026-09-29-{no-ignore,gitignore,cached}.json.gz and sloc-validation-2026-09-29.json.gz; brief docs/project/research/research-2026-09-29-sloc-tools-survey.md.
