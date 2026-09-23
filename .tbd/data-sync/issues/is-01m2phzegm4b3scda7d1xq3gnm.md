---
type: is
id: is-01m2phzegm4b3scda7d1xq3gnm
title: End-to-end verification of the final 0.1.0 release candidate
kind: task
status: open
priority: 0
version: 6
labels:
  - release
  - testing
  - runbook-verified
dependencies:
  - type: blocks
    target: is-01m2phzevyf68fdz9zcs3yzncw
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
  - type: blocks
    target: is-01m2phzkwcvz3fwk2nh150bdmj
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T02:08:52.755Z
updated_at: 2026-09-23T08:35:19.345Z
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

2026-09-23 post-merge QA on main 7e06e5a4 (after the alpha correctness stack #99-#117 and performance stack #94/#97/#105/#109 merged; tree identical to the one that passed an uninterrupted make check and make cross-lint). Correctness runbook (macOS arm64, bare metal, APFS, uid 502, 14/17 kinds; devices need root, APFS refuses non-UTF-8): the runbook scripts were stale for report/7 and could not prove serving on a tree with refusals; fixed in PR #118. With the fix: refusal tree 23/23 partial and withheld, 0 mismatches; complete tree 23/23 served, 0 mismatches; cross-warm 30/30; broken-cache wrapper exits 1 with 23 NO-SNAPSHOT. Integration runbook: sections 1-4 and 7 covered by make check on the identical tree; 5 cache by hand ok (runbook text for the first --analyze source is inaccurate after the metadata steps; content tier scanned then revalidated as expected; doc fix in #118); 6 watch ok (idle 0% CPU, record within 2s, snapshot survives kill -9, cache-only serves stale, scope flags exit 2). Full path-independence matrix dispatched on 7e06e5a4. This bead's post-publication items (artifact download, checksums, installed wheel, real trees) remain open.
