---
type: is
id: is-01m2phzegm4b3scda7d1xq3gnm
title: End-to-end verification of the final 0.1.0 release candidate
kind: task
status: open
priority: 0
version: 5
labels:
  - release
  - testing
dependencies:
  - type: blocks
    target: is-01m2phzevyf68fdz9zcs3yzncw
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
  - type: blocks
    target: is-01m2phzkwcvz3fwk2nh150bdmj
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T02:08:52.755Z
updated_at: 2026-09-23T04:50:54.649Z
---
After the stabilization fixes merge: main CI green; dispatch the release rehearsal on main; download
every artifact and verify SHA256SUMS; run the end-to-end harness against the CI-built macOS arm64 wheel
(filesystem oracle, every view and format, YAML equals JSON, .gitignore selection, analysis sidecar
warm-versus-cold equality, all five cache policies, partial results and exit codes, usage errors,
ignore budget, watch create/modify/delete and Ctrl-C, Python/CLI parity, uv tool paths with an explicit
GIL interpreter, real trees ~/.rustup, ~/wrk, ~/Library/Application Support).

Baseline on the 5f2d36d rehearsal wheel (2026-09-16): 52 of 60 checks passed; failures were 4 harness
errors and the defects now tracked as fdu-gija, the Ctrl-C bead, the YAML contract bead, and uv's
free-threaded interpreter choice. Harness: scratch e2e_fdu.py (consider committing under scripts/release).

## Notes

2026-09-22 pre-merge candidate evidence: PR117 adc39d24 and current dependent heads 113–116 passed exact-head CI (19/19 each); final full PI run 35819047425 passed 46,926 cases with zero exceptions/empty registry on all three platforms. Packaged rehearsal 35815753312 passed all nine jobs at ff2b07da; subsequent changes affect tests/docs, not production or packaging. This does not satisfy this bead’s post-merge main-branch artifact download, checksum, installed-wheel, watch, real-tree, and release end-to-end checks. Leave open; no merge or publication has occurred.
