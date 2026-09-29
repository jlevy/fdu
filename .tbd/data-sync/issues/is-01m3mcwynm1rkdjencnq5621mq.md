---
type: is
id: is-01m3mcwynm1rkdjencnq5621mq
title: "v0.2.1 patch release: Linux performance verdicts and case-variant .gitignore"
kind: epic
status: closed
priority: 1
version: 25
delegate: claude-code@spud10
labels: []
dependencies: []
child_order_hints:
  - is-01m3j15py91vjyse2zzvhgxaqp
  - is-01m3h4knhxm6kwm8ykhpb7br6n
  - is-01m3h4ky0ecg7gpas4gssxhy4e
  - is-01m3m586j10yfkzrmmy65fkv4j
  - is-01m3mcx2jb4d9k7kwdpjbycx4v
  - is-01m3mcx2yq517yvn7jbcxaw98m
  - is-01m3mg15aem39d0chrsv1swtyb
  - is-01m3mg15qh1hbb3g7nqbnc1k4n
  - is-01m3mg16529x3x0fn8qw190670
  - is-01m3mg16k7j7mtx4sn4wzqfqgt
  - is-01m3msv53jeze9d48vt0kanz87
  - is-01m3mfpqz3cd8w5f9289t6yejb
  - is-01m3mfps518xc4rmbrc5zsh8es
  - is-01m3mfpqhjhwv9redfnv8e87ya
  - is-01m3mfprbtj5qnhxjj7vwvttav
  - is-01m3mfprr9bh9nrfg8ehwq5ez6
  - is-01m3n1x8fyewf1036yvajrzm7n
  - is-01m3n3k089mcfa2yws9sw9gved
  - is-01m3n3k0q15zxng3mqb5en5zc0
  - is-01m3n401mxvy9ftm72ctbm2wpm
  - is-01m3nrm3e34k0x7acfjys49ek0
  - is-01m3nvbmwzqv2rt1dvygg312zd
hold: null
hold_until: null
created_at: 2026-09-28T16:17:21.075Z
updated_at: 2026-09-29T05:51:46.867Z
started_at: 2026-09-29T05:51:39.212Z
closed_at: 2026-09-29T05:51:46.867Z
close_reason: "v0.2.1 completed at c16445757ce82501a2d4eea1ff59b638eef42bd1. CI 36522644331, rehearsal 36522734549, and publishing 36524303462 succeeded. Tag verified; make release-body and release-published passed: both crates and all six PyPI files match the publishing manifest. Created https://github.com/jlevy/fdu/releases/tag/v0.2.1 with all eleven assets; release-announced passed, including both docs.rs builds and fresh pinned/latest wheel installs. Public crate and PyPI READMEs render; PyPI lists one sdist and five wheels; installed skill smoke passed in isolated scratch. Candidate branch deleted and absence verified. Existing QA limitations and remaining follow-up beads are unchanged; no source changes or repeat full stability run in this completion pass."
resolution: null
duplicate_of: null
---
User decision 2026-09-28: v0.2.1 carries the Linux performance checks (H161 confirmation for merged #149, H159 decision for #150, H157 after it) and the case-variant .gitignore change (fdu-0w1b). Worked by the next agent on Linux from the handoff; released with the streamlined process.
