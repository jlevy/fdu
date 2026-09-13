import assert from "node:assert/strict";
import test from "node:test";

import { auditGolden, scenarioNames } from "./check-opened-root-goldens.mjs";

const valid = [
  "scenario: schema=1 name=sample",
  "action.open: OpenOptions { root: \"$ROOT\" }",
  "result.open: Ok(SessionId(session-1))",
  "action.close: close()",
  "result.close: Ok(())",
  "final: attrs=[TIME] [SYSTEM_TIME] [DIR_SIZE] [ALLOCATED] [INODE] [DEVICE]",
  "",
].join("\n");

test("accepts the closed normalization vocabulary", () => {
  assert.deepEqual(auditGolden("sample", valid), []);
});

test("rejects hidden machine values and incomplete sessions", () => {
  const findings = auditGolden(
    "sample",
    'scenario: schema=1 name=sample\nresult.open: SessionId(42) root="/tmp/private" mtime_ns: 9\n',
  );
  assert.ok(findings.some((finding) => finding.includes("missing action.open")));
  assert.ok(findings.some((finding) => finding.includes("opened-root identity")));
  assert.ok(findings.some((finding) => finding.includes("absolute machine path")));
  assert.ok(findings.some((finding) => finding.includes("platform-assigned attribute")));
});

test("accepts declared durations and rejects measured ones", () => {
  const declared = valid.replace(
    "action.close: close()",
    "action.changes: timeout: 0ns settle: 1ms max_hold: 10ms timeout=1s\naction.close: close()",
  );
  assert.deepEqual(auditGolden("sample", declared), []);
  for (const measured of ["elapsed: 1.234567ms", "took 15.5µs", "Instant { tv_sec: 1, tv_nsec: 2 }"]) {
    const findings = auditGolden("sample", valid.replace("result.close: Ok(())", `result.close: ${measured}`));
    assert.ok(
      findings.some((finding) => finding.includes("wall-clock duration")),
      `${measured}: ${findings}`,
    );
  }
});

test("derives scenario inventory from the Rust declarations", () => {
  assert.deepEqual(
    scenarioNames('SessionTrace::new("first-case", root); SessionTrace::new("second-case", root);'),
    ["first-case", "second-case"],
  );
});
