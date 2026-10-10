---
type: is
id: is-01m4kjcqgpkc6nf1qtkkj0mmzy
title: Decide whether a switch should turn off work fdu's peers do not do
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-10T18:49:48.310Z
updated_at: 2026-10-10T18:49:48.310Z
---
Maintainer question (2026-10-10): for fair benchmarking, consider a flag that disables work beyond what competing tools do, or at least document that work. Documented in docs/performance-measurements.md (What fdu Does That Its Peers Do Not), from strace counts: fdu 0.5.0 makes the same statx per entry as pdu and diskus and skips their fstat per directory; the one extra I/O is .gitignore, which --no-gitignore already turns off; the rest is user-space (classification, per-directory roll-ups with ages and ignored shares, the rendered tree). A broader switch (no ages, no roll-ups) would be a one-off flag, which the design principles' six-axes rule rules out; a columns axis is the plan's open question. Decide: document only, or design a columns axis.
