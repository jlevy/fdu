---
type: is
id: is-01m0129t2wsdsv20mt3bq7s0zh
title: Document and rehearse the fdu 0.1.0 release
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels: []
dependencies: []
parent_id: is-01m01293x5gaacv3vxjdtrg146
created_at: 2026-08-14T21:19:28.347Z
updated_at: 2026-08-14T23:36:36.924Z
closed_at: 2026-08-14T23:36:36.923Z
close_reason: Published the release/security/support policy and same-account, project-specific PyPI/crates.io authentication runbook; rehearsed host artifacts and all missing/identical/conflict decision paths; and read-only rechecked both names. External publisher configuration and actual publication remain in fdu-9cf0.
---
Document the supported platform, Python version, MSRV change, compatibility, deprecation, security, rerun, partial-publication, and incident-recovery policies. Map the same-owner, project-specific PyPI and crates.io authentication setup used by Flowmark. Rehearse the non-uploading artifact path and missing, identical, and conflicting registry classifications. External publisher configuration, token use, tagging, uploads, and post-publication verification remain in fdu-9cf0.

## Notes

The release runbook maps the exact PyPI pending publisher and crates.io bootstrap/steady-state authentication model, supported matrix, least-privilege boundaries, and partial-failure recovery. Local host rehearsal and all classification tests pass. Registry names were read-only rechecked through authoritative endpoints on 2026-08-14; both returned not found. No accounts or registries were mutated.
