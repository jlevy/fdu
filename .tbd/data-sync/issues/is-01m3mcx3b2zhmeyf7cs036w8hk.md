---
type: is
id: is-01m3mcx3b2zhmeyf7cs036w8hk
title: Scrub host names from tbd bead records and stop recording them
kind: task
status: open
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-09-28T16:17:25.857Z
updated_at: 2026-09-28T16:21:20.228Z
---
tbd start records delegate as <agent>@<host>; ~230 records on the public tbd-sync branch carry this machine's host name. User approved removal 2026-09-28. Stop new ones (claim with tbd start --as <name> / configure identity), scrub current records with a forward commit, and decide separately on rewriting tbd-sync history (other replicas would re-push old history).

## Notes

2026-09-28: (1) stopped new host names: this checkout's .tbd/state.yml agent_name is now 'claude-code' (was claude-code@spud10), fdu-perf-linux-20260919 is 'cursor', the three Codex worktrees got 'agent_name: codex'; tbd whoami now resolves to 'claude-code'. Agents should also claim with --as <agent> or TBD_AGENT (AGENTS.md note to follow in the release-process PR). (2) Scrubbed all 243 delegate values containing '@host' via tbd update --delegate (forward commit, synced); 0 remain, and the host name appeared in no other field. (3) NOT done: rewriting tbd-sync history. Old records stay reachable in history; a force-push would be undone by any replica that syncs its old history back (Codex worktrees, the Linux host), and GitHub keeps unreferenced commits reachable by SHA. Needs the user's call after that trade-off.
