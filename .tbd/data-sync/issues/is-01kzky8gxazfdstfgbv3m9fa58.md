---
type: is
id: is-01kzky8gxazfdstfgbv3m9fa58
title: Preserve filesystem-native identity through classification and Python
kind: bug
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - filesystem
  - python
  - api
dependencies:
  - type: blocks
    target: is-01kzg4bf862ajh8g2tmv5bznng
  - type: blocks
    target: is-01kzg4bfj2cqzcksgpmfce89w6
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T18:58:44.265Z
updated_at: 2026-08-13T08:13:48.313Z
closed_at: 2026-08-13T08:13:48.305Z
close_reason: Classification now operates on native OsStr units, so ASCII extensions survive non-Unicode Unix stems and invalid Windows wide stems. Python accepts os.PathLike/bytes paths, returns reversible surrogateescaped identities for roots, children, reports, watch events, caches, and deltas, and maps filesystem failures to OSError with errno/message/filename. Core, Clippy, rustdoc, all-feature tests, and installed cp312-abi3 wheel smoke pass.
---
Index contribution currently calls OsStr::to_str before extension classification, so a non-Unicode name with an ASCII extension disappears from by-extension totals. The Python boundary accepts string-only paths and renders child and delta paths lossily. Make classification operate on native path units and prove ASCII extension behavior for Unix bytes and Windows wide strings without lossy conversion. Accept normal Python os.PathLike inputs losslessly, expose reversible identity for root, children, and since records, and map I/O errors to useful OSError fields including the path and OS cause. Expand installed-wheel tests to the declared minimum and current Python versions, cover representative failures, and clean up only the exact temporary roots they create. Keep the binding bulk-oriented and independent of the watch feature.

## Notes

The CLI UX follow-up now extracts wheel console sys.argv as Vec<OsString>, so uvx/native CLI arguments preserve surrogateescaped Unix bytes and Windows wide-string identity. This bead remains open for its distinct scope: extension classification and the public Python open/scan/children/since path APIs still narrow or emit lossy strings.
