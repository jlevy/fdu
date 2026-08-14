---
type: is
id: is-01kzz3neeeh5tw28gybm38hy7s
title: Worker-count knee is tier-dependent, not only platform-dependent
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T03:04:49.357Z
updated_at: 2026-08-14T03:04:49.357Z
---
A warm 450k indexed sweep on 4-core Linux found two workers statistically tied with four (+0.08 percent, interval -1.1 to +2.19) and six already worse (+3.27 percent), so the shipped six-worker cap sits past the knee for the indexed tier rather than under it. That contradicts H76's premise that Linux is under-parallelized, but H76's evidence was diskus, which competes with the aggregate tier and retains no index. The aggregate tier has no index consumer to saturate and may well want more workers, which would make the right worker count a function of the retained-state tier as well as the platform. Settling it needs the transient-summary probe job from fdu-tyjx; until then only the indexed half is measurable. Numbers include constant probe oracle overhead, so magnitudes are diluted while ordering holds.
