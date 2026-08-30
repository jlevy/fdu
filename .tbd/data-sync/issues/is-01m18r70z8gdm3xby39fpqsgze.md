---
type: is
id: is-01m18r70z8gdm3xby39fpqsgze
title: macOS home scans exit 2 and emit one warning per TCC-protected path
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md
labels:
  - macos
  - cli
dependencies: []
created_at: 2026-08-30T07:12:48.615Z
updated_at: 2026-08-30T07:12:48.615Z
---
On main (not PR #48): 'fdu ~' produces a correct roll-up - 214 GiB, 4,114,663 files - but exits 2 and writes 158 'warning: I/O error at ...: Operation not permitted' lines, one per TCC-protected path (~/.Trash, ~/Library/Accounts, ~/Library/Biome, ~/Library/Caches/CloudKit, and so on).

Those denials are unavoidable on a stock Mac without Full Disk Access, so on macOS the single most obvious command reads as a failure. Agents treat exit 2 as 'the scan failed' and fall back to du - which is exactly what happened in the field report that opened this work.

Isolated fixture confirms the semantics are as designed: one unreadable subdirectory gives exit 2, and --allow-partial gives exit 0.

This is deliberate, not an oversight - fdu-yz68 closed on 'validated default exit 2 ... without weakening strict partial-result semantics' and explicitly declined to adopt dust's behaviour implicitly. So do NOT silently flip it to 0. Open bead fdu-jej9 already names the missing piece: exit codes that distinguish 'partial due to unreadable paths' from 'failed'.

Options to weigh: (a) keep exit 2, collapse the 158 warnings into one summary line stating the count and naming --allow-partial; (b) implement fdu-jej9's distinct code for OS-policy denials; (c) treat TCC/EACCES as expected and exit 0 - most agent-friendly, reverses a deliberate decision, can mask real permission faults.

Acceptance: a macOS home scan is unambiguously distinguishable from a failed scan by an unattended caller, without weakening strict partial semantics.
