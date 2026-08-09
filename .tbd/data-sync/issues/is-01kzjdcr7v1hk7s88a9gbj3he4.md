---
type: is
id: is-01kzjdcr7v1hk7s88a9gbj3he4
title: Guard conditional observations against structural and ABA races
kind: bug
status: closed
priority: 0
version: 4
labels:
  - correctness
  - concurrency
  - architecture
dependencies: []
parent_id: is-01kzjceqsx74x63350s7hjb8q0
created_at: 2026-08-09T04:44:42.619Z
updated_at: 2026-08-09T05:10:37.448Z
closed_at: 2026-08-09T05:10:37.448Z
close_reason: "Implemented in commit 5014b13 with focused regression tests; local make check and all PR #1 checks pass on Linux, macOS, Windows, MSRV 1.85, dependency policy, docs, and Python wheel smoke."
---
A PathState-only conditional can accept delayed work after a parent replacement or after state changes away and back. Capture generation plus direct and child-map revisions: present targets reject ABA, absent targets guard the nearest existing ancestor, and destructive directory operations guard child structure. Validate every conditional op at one batch boundary, while allowing independent subtrees and directory-metadata-only changes to proceed.

## Notes

Implemented focused arbitration tests for parent replacement, present/absent ABA, destructive subtree races, non-directory ancestors, independent subtrees, and multi-op batch boundaries.
