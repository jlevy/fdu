---
type: is
id: is-01kzy2rjnap5ewjbz8seft57sb
title: "Inode-ordered statting: cold verdict needs bare metal"
kind: chore
status: open
priority: 3
version: 1
labels:
  - perf
  - linux
dependencies: []
created_at: 2026-08-13T17:29:48.969Z
updated_at: 2026-08-13T17:29:48.969Z
---
The frontier research lists d_ino-sorted statting as the highest-ROI cold technique (4-6x literature claim, ext4/btrfs-gated). Scouting rig could not decide it: -2.3% cold [-3.9%, +0.5%] and +6.8% warm [+2.8%, +13.7%] (sort cost) at 450k entries, because guest-cold virtio reads are host-cached and carry no seek penalty. Keep the hypothesis alive but require a bare-metal ext4 SSD/HDD host; pair it with the planned controlled-cold matrix (fdu-tk1b). Sort only above a per-directory entry threshold and only in cold-suspected states (service-time calibration already detects those) so the warm regression cannot ship.
