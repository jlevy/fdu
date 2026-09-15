---
type: is
id: is-01m2k28bahqt6zrm0geh2vy38j
title: "PR #61 review PR61-GUIDE-4: the 0.1.1 recovery path reruns Tag step 3, which requires 404 for names 0.1.0 now occupies"
kind: bug
status: closed
priority: 3
version: 3
labels:
  - release
dependencies: []
parent_id: is-01m2k27z25tt9ygs4c1nchhzez
created_at: 2026-09-15T17:36:23.889Z
updated_at: 2026-09-15T17:41:55.028Z
closed_at: 2026-09-15T17:41:55.027Z
close_reason: "2f78939: recovery step 5 keeps Tag step 3's name check for names 0.1.0 never reached, and checks the 0.1.1 version URL (must print 404) for names it did reach; Tag step 3 points there."
resolution: null
duplicate_of: null
---
PR #61, delta review 5213560245. docs/project/guides/release-process.md:203-212, 398-404 @ 4db083b. Recovery step 5 repeats the by-hand section with 0.1.1, which reruns Tag step 3; its name checks must print 404, but a name 0.1.0 reached prints 200. Give the 0.1.1 path a version-level availability check.
