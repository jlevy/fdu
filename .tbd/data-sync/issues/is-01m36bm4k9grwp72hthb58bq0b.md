---
type: is
id: is-01m36bm4k9grwp72hthb58bq0b
title: Measure the composed correctness stack against main before merge
kind: task
status: closed
priority: 1
version: 3
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:41.608Z
updated_at: 2026-09-24T07:38:48.278Z
closed_at: 2026-09-24T07:38:48.276Z
close_reason: "Regression sanity check 2026-09-24 on the alpha candidate (fdu 0.1.0-dev+g235b3c23a, release build) with make perf-compare-tools on the README's corpus (balanced recipe, 1,000,001 entries, digest 4bbd97c0d3d4e2ad), 12 paired trials, 3 warmups, host load 45-88 (uncontrolled): dumac +11.0% [+4.9%, +17.6%] (README +11.3% [+5.8%, +13.5%]); diskus +48.2%; pdu +80.1%; dust +113.2%; dua +169.7%; fdu at main 0059ddd5 +10.9% [-5.0%, +16.5%] (no regression). fdu median 10.71 s (README 5.206 s at load 8-10). 0 invalid samples, 0 semantic or oracle mismatches, no drift. README figures kept per the maintainer (quieter-host numbers). Progress-handle cost: exp-156 (no handle vs main, confirmed) and exp-157 (handle attached, open, needs a quiet rerun)."
resolution: null
duplicate_of: null
---
The alpha correctness plan makes no performance claim, and nobody has measured #117 against main, although #113 adds status/provenance computation and #115 reroutes execution.rs/scan.rs. Earlier stacks used the gate fdu PATH within 10% of main on a control-free and a control-rich tree (2026-09-14 decision). Run make perf-compare, interleaved and paired, on a real tree for fdu PATH, --view summary, a warm open and --format json; record with make perf-record. The README headline (fdu-y5xr, dumac +11.3%) must be re-measured again on the final release candidate after #94/#97/#105 compose.
