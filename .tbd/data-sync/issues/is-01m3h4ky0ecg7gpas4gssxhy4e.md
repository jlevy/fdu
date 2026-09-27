---
type: is
id: is-01m3h4ky0ecg7gpas4gssxhy4e
title: Re-test H157 (direct file fold, owned names) with the product CLI job as primary
kind: task
status: open
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - linux
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-09-27T09:54:53.582Z
updated_at: 2026-09-27T19:18:02.918Z
---
exp-161 rejected H157 on its pre-registered probe job: cold-scan-index wall -2.22% [-4.04%, +0.04%], component -4.12% [-7.40%, +1.25%]. The same change paired on the product CLI job (fdu --cache off --depth 1 --limit 10, tool harness, 12 pairs, quiet) measured -3.71% [-4.71%, -1.97%], and allocations fell 7.03M -> 4.28M. The change: DetachedIndexBuilder::push_directory folds a file child straight into its parent roll-up through InternedRollUp::add_file, and Index::contribution builds a file's contribution by applying add_file to an empty roll-up, so both paths share one definition; WalkEmission::record_entry takes the listing's owned OsString and record_detached_entry moves it into DetachedChild instead of to_os_string. Pre-register the product job as primary, measure on the balanced 1M tree and one reconstructible real subject (linux-v6.12), and keep only under the accept rule. See docs/project/reports/report-2026-09-27-fdu-linux-tool-comparison.md.
