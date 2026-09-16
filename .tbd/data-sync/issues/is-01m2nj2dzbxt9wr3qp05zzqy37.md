---
type: is
id: is-01m2nj2dzbxt9wr3qp05zzqy37
title: Move the three completed CLI plans to specs/done and rewrite their inbound links
kind: task
status: open
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-16T16:51:16.074Z
updated_at: 2026-09-16T16:51:30.165Z
---
Three plans are marked Completed but still filed in docs/project/specs/active/, because moving them while the doc-drift PRs are open would break links across several of them at once. Move them with `git mv` in one change once those PRs have merged:

- plan-2026-08-09-fdu-cli-ux-and-agent-skill.md (PR #2, epic fdu-6c8n closed)
- plan-2026-08-10-fdu-composable-cli-surface.md (PRs #5 and #37; follow-up epics fdu-pxeb and fdu-ktyl stay open and are not a reason to keep the plan in active/)
- plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md (PR #39; epic fdu-yov0 stays open only for fdu-c2ml's JSONL parser check. Move it anyway, or close fdu-c2ml first. Decide which.)

Do this after the doc-drift PRs for audit packages 1, 3/4 and 5, and README PR #68, have merged, because the inbound links live in their files.

Inbound links to rewrite, as of origin/main 16efcd0 plus the package 3/4 branch (line numbers will move):

- README.md:594 (composable) — PR #68 owns this file
- TODO.md:26 and :47 (view vocabulary), :27 and :31 (composable)
- TODO.archive.md:30 (CLI UX), :31 (composable)
- docs/project/architecture/fdu-design-principles.md:283 (composable)
- docs/project/research/research-2026-08-10-performance-frontier.md:85 (composable). This is a dated research file, so change only the link.
- docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md:383 (CLI UX)
- docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md:494 (composable)
- docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md:833 (composable)
- docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md:1002 (composable, `../active/` becomes a same-directory link)
- docs/project/specs/done/plan-2026-08-21-fdu-python-cli-parity.md:491 and :493 (composable and view vocabulary). These same-directory links are broken today and start resolving once the files move. Check them rather than edit them.

Outbound links in the moved plans that name a plan still in active/ need a `../active/` prefix:

- composable: plan-2026-08-08-fdu-phase-1.md, plan-2026-08-10-fdu-fsevents-scoped-revalidation.md (three links), plan-2026-08-11-fdu-progressive-results.md, plan-2026-08-14-fdu-release-packaging-python-api-polish.md
- CLI UX: plan-2026-08-08-fdu-phase-1.md, plan-2026-08-09-fdu-rust-engineering-quality.md. Its `../done/plan-2026-08-09-fdu-cli-golden-tests.md` becomes a same-directory link.
- Links among the three moved plans stay valid if all three move together.

Bead spec_path values also name these files: 54 beads for the composable plan (9 open), 9 for the view vocabulary plan (2 open), and 9 for the CLI UX plan (1 open), counted with `tbd list --all --spec <file>` on 2026-09-16. Update each spec_path, or confirm that tbd resolves a moved spec by filename suffix.

Acceptance: the three files are in specs/done/, a relative-link check over every *.md reports no new broken link, `make docs-format-check` passes, and `tbd list --spec` still finds the linked beads.
