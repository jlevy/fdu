---
type: is
id: is-01m4890qg7xpqq58xkk47gmx0j
title: "PR #177 C5: no test pins the Rust opened-root read refusal wording at crates/fdu-core/src/opened/read.rs"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
created_at: 2026-10-06T09:34:19.142Z
updated_at: 2026-10-06T09:34:19.142Z
---
Review C (https://github.com/jlevy/fdu/pull/177#issuecomment-6013447138), Low. no test pins the Rust opened-root read refusal wording at crates/fdu-core/src/opened/read.rs:295 (the unit test calls validate_opened directly; the Python test is refused earlier in read_opened). Add a test through the opened read path.
