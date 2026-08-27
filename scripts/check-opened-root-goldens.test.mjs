import assert from "node:assert/strict";
import test from "node:test";

import { auditGolden, scenarioNames } from "./check-opened-root-goldens.mjs";

const valid = [
  "scenario: schema=1 name=sample",
  "action.open: OpenOptions { root: \"$ROOT\" }",
  "result.open: Ok(SessionId(session-1))",
  "action.close: close()",
  "result.close: Ok(())",
  "final: attrs=[TIME] [DIR_SIZE] [ALLOCATED] [INODE] [DEVICE]",
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

test("derives scenario inventory from the Rust declarations", () => {
  assert.deepEqual(
    scenarioNames('SessionTrace::new("first-case", root); SessionTrace::new("second-case", root);'),
    ["first-case", "second-case"],
  );
});
