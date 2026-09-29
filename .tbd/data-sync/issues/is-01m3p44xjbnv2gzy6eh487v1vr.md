---
type: is
id: is-01m3p44xjbnv2gzy6eh487v1vr
title: "H175: derive each listing's control chain from its parent's instead of ControlTable::chain_for per listing"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T08:22:53.771Z
updated_at: 2026-09-29T10:10:22.742Z
closed_at: 2026-09-29T10:10:22.742Z
close_reason: "Accepted and merged (2379233a): exp-178 (H171, default-tree -29.62%, summary -25.45%) and exp-179 (H175, default-tree -3.31%) on linux-v6.12, quiet, 20 pairs; placebos at zero; answers identical to the base and to git."
resolution: null
duplicate_of: null
---
From the Fable plan review (F6c). chain_for probes a BTreeMap<PathBuf> once per ancestor with component-wise Ord: ~104M consumer instructions on linux-v6.12. Store the chain beside each directory's EntryId in DetachedIndexBuilder.directory_ids; a listing's chain = parent's + its own source. Implemented as a separate commit in the H171 worktree and measured stacked on H171 in the Q2 cell. Prediction: consumer Ir -100M, wall -2-4% alone.
