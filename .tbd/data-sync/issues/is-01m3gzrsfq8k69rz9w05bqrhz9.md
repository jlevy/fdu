---
type: is
id: is-01m3gzrsfq8k69rz9w05bqrhz9
title: Reconcile tbd managed-skill drift checks with required Markdown formatting
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-09-27-cli-and-skill-followups.md
labels: []
dependencies: []
created_at: 2026-09-27T08:30:09.910Z
updated_at: 2026-09-30T10:10:01.111Z
closed_at: 2026-09-30T10:10:01.111Z
close_reason: |
  Reproduced 2026-09-30 with tbd 0.9.0; the defect is in tbd, not this repository. `tbd doctor` reports .agents/skills/tbd/SKILL.md and .claude/skills/tbd/SKILL.md as "stale managed file". In a scratch copy, `tbd setup --auto --surfaces=portable,claude` regenerates both (doctor then says current), and the only difference from the committed files is five guideline-group notes under "Available Guidelines" (General engineering, TypeScript, Python, Rust, Convex), which tbd emits as single unwrapped lines: `lines.push(`*${group.note}*`)` in generateShortcutDirectory (src/file/doc-cache.ts). The rest of tbd's output is already in flowmark normal form: flowmark --auto of tbd's output is byte-identical to the committed file, and tbd's raw output fails flowmark --auto --check (make docs-format-check). Doctor's inspectManagedArtifact (src/cli/lib/managed-artifact.ts) compares `managedContent === expectedContent` byte for byte. The YAML-presentation half of the original report no longer reproduces: the frontmatter is identical. Belongs upstream in github.com/jlevy/tbd: wrap group notes (or all generated prose) as flowmark does, or normalize Markdown whitespace in the doctor comparison. No repository change: excluding the generated skills from docs-format-check would take them out of the documentation check, which this bead ruled out. Plan updated in docs/project/specs/active/plan-2026-09-27-cli-and-skill-followups.md.
resolution: canceled
duplicate_of: null
---
tbd 0.9.0 setup produces current portable/Claude skills, then required make docs-format changes YAML presentation and prose wrapping. tbd doctor reports both as stale although parsed frontmatter and whitespace-normalized body match tbd skill. Decide formatter ownership or upstream semantic drift normalization; keep actual content drift detectable. Reproduction and normalized comparison recorded in the 2026-09-27 tracking review. Do not disable the repository documentation check.
