---
type: is
id: is-01m499cpky6h46gy2tee98mvng
title: "PR #182 A2: 'round' means a paired trial and a campaign on the perf page"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m499cf1q15te09v5zkpvdxd2
hold: null
hold_until: null
created_at: 2026-10-06T19:00:05.885Z
updated_at: 2026-10-06T19:44:31.776Z
started_at: 2026-10-06T19:00:11.300Z
closed_at: 2026-10-06T19:44:31.775Z
close_reason: "Fixed in 51d77172: round defined in The Loop as one paired trial of each arm (the harness's --trials), headline links to it, figcaption says campaign, evidence report definition matches. Test: test_a_round_is_defined_where_it_is_first_stated_and_names_no_campaign. Dispositions: https://github.com/jlevy/fdu/pull/182#issuecomment-6024139573"
resolution: null
duplicate_of: null
---
Low. report_html.py:2551-2552, :2214, :1219, :2275, :342; evidence report :11-12. Coordinator choice: keep 'round', define at first use as one paired trial of each arm (the harness's --trials), match the report's definition, reword :342 so a campaign is a campaign. PR #182, review https://github.com/jlevy/fdu/pull/182#issuecomment-6022824692
