---
type: is
id: is-01m4qd8phf7xrm86zsnymre3ft
title: Remove blank startup delay from README demo
kind: bug
status: closed
priority: 2
version: 5
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-12T06:37:11.078Z
updated_at: 2026-10-12T19:09:10.995Z
started_at: 2026-10-12T06:37:40.693Z
closed_at: 2026-10-12T19:09:10.995Z
close_reason: "Completed in PR #200, merged as 924007e477eab2ffc2ee9daaaa73fc0645751e92. Quiet internal-storage capture, 26.4s matching MP4/GIF, immediate typing and no scheduled holds, original palette preserved. Fresh/cached commands 5.534s/0.733s; all 343 cast payloads and later intervals preserved; 313/313 frame changes verified. Two independent reviews, including strong senior Review B, found no issues. All make check components passed across the disk-interrupted run and exact terminal continuation; all 20 PR CI checks passed. Public README verified against the corrected preview. Release preparation and unresolved gates remain in the parent."
resolution: null
duplicate_of: null
---

## Notes

Compared receipts: prior night tree0.385/fresh8.123/cached0.752s, preceding take0.618/23.368/3.048s, current0.425/30.459/4.028s. First typed character prior1.022s/current2.589s. Created immediate-start local MP4/GIF from retained lossless master by removing152 frames at60fps, preserving all later frames; verified577 frame changes against original cast frame positions. Preview artifacts/evidence in immediate-start under durable external release evidence. README replacement pending choice of quiet re-record versus retaining slow take; current take load not independently established, external content-cache placement is a confound. No repo edits or public media replacement yet.

2026-10-12: Replacement26.4s MP4/GIF completed; first typed character frame0. Only4.196s startup removed; all343 payloads and later intervals preserved. Frame verification313/313 and independent review passed; prior/current background/titlebar pixels identical. Public preview https://github.com/jlevy/fdu/pull/196#issuecomment-6129700944 . Public MP4 HTTP200 exact3179202bytes SHA110862d1b95ad23f543dc2f0164818a7a8c70bdeac6d92e44e94f505a41b30a6 verified twice. README/GIF/declaration/scenario updated on codex/demo-immediate-start, pending fullgate/PR. Gate exposed unrelated announcement-test TMPDIR substring falsepositive; assertion corrected to filenames and fdu-demo- fixtureprefix retained,13 tests pass, independentreview zero findings. Fullmakecheck rerunning, failedlog preserved. No tag or publication.
