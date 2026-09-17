---
type: is
id: is-01m2rt8s04hfaybe9y5y5mqr69
title: "Windows golden: --watch --one-filesystem names ScopeUnsupported instead of WatchScope"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m2rss53ech1v9fvh9hdd4cgg
created_at: 2026-09-17T23:12:15.876Z
updated_at: 2026-09-17T23:22:18.726Z
closed_at: 2026-09-17T23:22:18.726Z
close_reason: "Addressed on 7390b62b: golden now asserts WatchScope via --scan-depth (honoured on every platform); --one-filesystem watch rule moved to a Unix-gated unit test. Windows Test job is green on that commit."
---
CI Test (windows-latest) failed on tests/golden/cli-surface.tryscript.md session 'Watching Rejects a Narrowed Scan Scope' (--one-filesystem variant). Expected RequestError::WatchScope wording; got 'unsupported scan configuration: --one-filesystem requires platform device identity' (RequestError::ScopeUnsupported). Cause: Cli::request calls Request::validate() (capability first) before validate_delivery(). On Unix, one_filesystem is honourable so WatchScope fires; on Windows, ScopeUnsupported fires first. Session::new already validates delivery first. Unit tests only call validate_delivery in isolation (query_request.rs a_watch_refuses_what_it_cannot_keep_current). Fix (pick one): (a) CLI watch path: compose Delivery then validate_delivery before Request::validate, matching Session::new; (b) golden platform pattern for the one-filesystem session. Proven by run 35284906671 job 105415040241.
