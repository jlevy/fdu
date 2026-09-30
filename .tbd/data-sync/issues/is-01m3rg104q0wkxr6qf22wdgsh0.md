---
type: is
id: is-01m3rg104q0wkxr6qf22wdgsh0
title: "Atomic writes: state the rule in the design principles and enforce it with a check in make check and CI"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m3rfz36eh8m7xp52aq5c0kmz
created_at: 2026-09-30T06:28:57.111Z
updated_at: 2026-09-30T07:13:13.818Z
closed_at: 2026-09-30T07:13:13.818Z
close_reason: "Rule 'Write Every File Whole' added under Data Structures in docs/project/architecture/fdu-design-principles.md. scripts/check-atomic-writes.mjs (+ .test.mjs, 14 tests) flags Rust non-test fs::write/File::create/fs::copy/write-mode OpenOptions (skipping cfg(test) items and test-only modules), Python open-family write modes and computed modes, write_text/write_bytes, json.dump to a file, shutil copies, os.open for writing, and Node writeFile(Sync)/appendFile(Sync)/createWriteStream/copyFile(Sync)/cpSync/open with write flags, ignoring comments and strings. Allowlists: HELPERS (snapshot.rs, skill_install.rs, scripts/atomic_write.py, scripts/atomic-write.mjs), MIRRORS (benchmarks copy must be identical), test code by name/dir, INPUT_WRITERS, and site EXCEPTIONS with reasons (stale ones fail); test inputs exempt with fdu-tq70 named. Wired as make atomic-writes in make check next to admission-sites, npm run check:atomic-writes in CI's test matrix next to admission, Node helper tests run there, Python helper tests in make release-test. Passes: 159 files, 37 listed raw writes. Commits bb1404a2..acb2882b."
resolution: null
duplicate_of: null
---
The rule in fdu-design-principles.md, and a check (Rust, Python and Node) that fails on any write outside the atomic helpers and the staged-rename path, wired into make check and CI, with tests.
