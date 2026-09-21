---
type: is
id: is-01m32twwnydz2mzfcr2aggmq7p
title: "PR #98 review R3: 64-bit file index is not unique on ReFS (Dev Drive)"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m32h6dpd97fr5f8db831dn3y
hold: null
hold_until: null
created_at: 2026-09-21T20:35:39.326Z
updated_at: 2026-09-21T20:57:26.180Z
started_at: 2026-09-21T20:36:00.249Z
closed_at: 2026-09-21T20:57:26.180Z
close_reason: "Fixed in 650b6b08: identity comes from GetFileInformationByHandleEx(FileIdInfo), folded so an identifier that is the 64-bit index zero-extended (NTFS) keeps the index and a nonzero high half is mixed in asymmetrically; a volume that does not answer FileIdInfo keeps the 64-bit index. Fold covered by a pure unit test; the existing windows_reconcile_detects_path_identity_replacement test passed in Windows CI. Not verified on ReFS itself."
resolution: null
duplicate_of: null
---
Medium from https://github.com/jlevy/fdu/pull/98#issuecomment-5764966314. windows_metadata.rs:61-63 uses nFileIndexHigh/Low; BY_HANDLE_FILE_INFORMATION docs say the 64-bit id is not guaranteed unique on ReFS, which Windows 11 Dev Drive uses. Fix: GetFileInformationByHandleEx(FileIdInfo) into FILE_ID_INFO (128-bit id) and fold or widen.
