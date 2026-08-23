---
type: is
id: is-01m0pqhs2711jjcz6g9x9wpcen
title: "PR #38 review R3: ledger headline selects the cumulative run by title substring"
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m0pqh0yf7etx8dywann7tnx3
created_at: 2026-08-23T07:14:52.614Z
updated_at: 2026-08-23T07:34:37.635Z
closed_at: 2026-08-23T07:34:37.634Z
close_reason: "Fixed: _campaign_headline selects by method.control containing BASELINE_COMMIT, excluding baseline decisions. Ledger headline now reports exp-032 (-54.5% cold-scan-index) instead of exp-054 (+1.4%). Three tests, including the exp-054 case as a fixture."
---
summary.py _latest_cumulative picks the last artifact whose TITLE contains "cumulative", which is exp-054 (control: main at 26280e4), so the ledger headline reports cold-scan-index +1.4% as "measured against the pre-work baseline". The true pre-work cumulative is exp-032 at -54.5%. Select by method.control matching timeline.py BASELINE_COMMIT (b565882).
