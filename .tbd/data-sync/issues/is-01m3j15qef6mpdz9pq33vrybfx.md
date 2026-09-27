---
type: is
id: is-01m3j15qef6mpdz9pq33vrybfx
title: "Decide: stop writing the metadata snapshot on one-shot metadata runs"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-09-27T18:13:56.815Z
updated_at: 2026-09-27T21:00:24.771Z
closed_at: 2026-09-27T21:00:24.765Z
close_reason: "Adopted as --cache auto|on|off (H160, exp-163): auto persists only where a later request reads what it stores, so a one-shot metadata report neither reads nor writes; content analysis, open, watch, and refresh still write; --cache on writes after every complete scan. --cache only became --stale-ok; refresh and read-only were removed. default-tree -13.81% [-15.99%, -10.65%], quiet, 12 pairs. Python open keeps writing: a session is its own later reader."
---
Decision needed on Option A in docs/project/research/research-2026-09-27-cache-economics-and-default-plans.md. Under --cache auto a one-shot metadata report never reads the snapshot (H108) but writes or re-verifies a full 79 MB image each run: +0.32 s on fdu . and +0.31 s on the default summary at 1M entries on Linux (screen); estimated 0.3-0.5 s on macOS. Readers that lose an implicit seed: --cache only (3.6x faster than a walk on APFS, about 20% on ext4) and Python open. Content analysis reuse comes from the sidecar and works without the metadata snapshot (measured). If adopted, document the explicit way to seed --cache only and pair with Option B (hardware CRC-32C, fdu-n75m part 3 fsync policy).
