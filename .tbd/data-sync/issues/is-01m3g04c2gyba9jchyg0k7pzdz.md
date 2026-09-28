---
type: is
id: is-01m3g04c2gyba9jchyg0k7pzdz
title: Review Spotlight and FSEvents incremental revalidation research
kind: task
status: closed
priority: 2
version: 5
delegate: claude-code
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-26T23:17:14.959Z
updated_at: 2026-09-28T16:20:53.455Z
started_at: 2026-09-26T23:17:33.599Z
closed_at: 2026-09-26T23:22:00.254Z
close_reason: Reviewed the repository's Spotlight and FSEvents research, historical spike, current tests and code state; reported the implementation chain and tracked three unresolved design decisions.
resolution: null
duplicate_of: null
---
Review the repository's prior Spotlight and FSEvents research, spikes, tests, open beads, and current engine constraints; report the work's present state and outline a sound support design without implementing it.

## Notes

Review complete. Findings: Spotlight has only a bounded mdfind coverage check and is unsuitable as the accounting authority; the 2026-08-10 FSEvents scratch spike confirmed useful changed-scope mechanics and replay cost but was not committed, omitted exact flags, and cannot support retention/completeness claims; main has Source::JournalScoped scaffolding but snapshot FORMAT_VERSION 5 has no replay cursor, no history_replay module/build feature exists, and no historical replay integration tests exist. Existing live-watch tests prove only resident notify/FSEvents behavior. Current implementation chain is fdu-uwhl probe, fdu-2cdv cursor/gate, fdu-3tun replay, fdu-6ld9 policy, fdu-rvje scoped reconciliation, fdu-hs10 performance. Review identified and tracked fdu-kfp6 (freshness contract), fdu-5l1j (build-feature policy conflict), and fdu-gmw2 (managed-store self-event exclusion). Recommended architecture: FSEvents nominates dirty scopes; existing bulk reconciler observes facts; exact commit path applies them; inventory+cursor publish atomically; full sweep handles every gate failure. Flat O(N) load/save prevents an end-to-end O(changes) claim until bounded persistent access lands.
