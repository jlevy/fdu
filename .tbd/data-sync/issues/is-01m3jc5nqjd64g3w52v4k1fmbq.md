---
type: is
id: is-01m3jc5nqjd64g3w52v4k1fmbq
title: Clarify gitignored subsets and align bar-first tree and remainder rows
kind: task
status: closed
priority: 2
version: 8
spec_path: docs/project/specs/done/plan-2026-09-26-code-analysis-presentation.md
labels:
  - presentation
dependencies: []
created_at: 2026-09-27T21:26:09.383Z
updated_at: 2026-09-27T22:50:25.615Z
closed_at: 2026-09-27T22:50:25.614Z
close_reason: "Implemented shared human integer grouping, presentation controls, precise ignore/remainder styling and notes, and wall-clock-first perf. All local targets pass across full gate and corrective reruns; 198 goldens, 53 authoritative Linux parity differences, 23 installed manual checks. Installed fdu 0.1.0-dev+g1f5bc81af with matching skill; executable source equals final stack. All 19 checks green on implementation6ac2bb44, docs502f435e, top9077a631. Review and validation: https://github.com/jlevy/fdu/pull/136#issuecomment-5860563364"
resolution: null
duplicate_of: null
---
Presentation follow-up: gitignored amounts included in totals; bar-first human trees with one aggregate remainder; population-aware colored bars, gray below-1% shares, bold sizes >=1 GiB, bright bold cyan names, gray directory suffix except dot paths. Shared inclusion notes; quiet suppresses informational diagnostics/progress but preserves warnings/errors/data. Configurable bar-size (10 default, nonpositive hides, 4096 maximum for human trees). Performance begins with wall-clock duration and parenthesized quantities. Every human integer count and file rate uses one shared grouping policy, including wide integer rates and cache lifecycle messages; structured values and executable flag arguments stay numeric/raw. Update focused and golden coverage, design/usage/skill, authoritative Linux parity, full gate, PR stack and fresh local installation.
