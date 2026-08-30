import assert from "node:assert/strict";
import test from "node:test";

import { auditGoldenText, formatFindings } from "./check-golden-observability.mjs";

test("accepts direct complete product output and fixture-only scripts", () => {
  const source = [
    "$ fdu --format json fixture",
    '{"schema":"fdu.report/5","complete":true,"reports":[]}',
    "$ node -e \"require('node:fs').writeFileSync('fixture/a.txt','a')\"",
  ].join("\n");

  assert.deepEqual(auditGoldenText(source, "tests/golden/good.tryscript.md"), []);
});

test("rejects wrappers that parse and select fields from fdu output", () => {
  const source = [
    '$ node -e "const {execFileSync}=require(\'node:child_process\'); const value=JSON.parse(execFileSync(process.env.FDU,[\'--format\',\'json\'])); console.log(value.complete)"',
    "$ fdu --format json fixture | jq '.complete'",
  ].join("\n");

  assert.deepEqual(auditGoldenText(source, "tests/golden/bad.tryscript.md"), [
    {
      file: "tests/golden/bad.tryscript.md",
      line: 1,
      reason: "a wrapper parses fdu output instead of recording the complete product response",
    },
    {
      file: "tests/golden/bad.tryscript.md",
      line: 2,
      reason: "a shell filter selects part of fdu output instead of recording the complete product response",
    },
  ]);
});

test("rejects redirection followed by a product-output filter", () => {
  const source = [
    "$ fdu --format json fixture > report.json",
    "$ jq '.complete' report.json",
  ].join("\n");

  assert.deepEqual(auditGoldenText(source, "tests/golden/bad.tryscript.md"), [
    {
      file: "tests/golden/bad.tryscript.md",
      line: 2,
      reason: "a shell filter selects part of fdu output instead of recording the complete product response",
    },
  ]);
});

test("rejects helper scripts that parse output from the configured binary", () => {
  const source = [
    "const result = spawnSync(process.env.FDU, args, { encoding: 'utf8' });",
    "const parsed = JSON.parse(result.stdout);",
  ].join("\n");

  assert.deepEqual(auditGoldenText(source, "tests/golden/bin/reducer.mjs"), [
    {
      file: "tests/golden/bin/reducer.mjs",
      line: 2,
      reason: "a helper parses fdu output instead of forwarding the complete product response",
    },
  ]);
});

test("formats deterministic actionable diagnostics", () => {
  assert.equal(
    formatFindings([
      {
        file: "tests/golden/example.tryscript.md",
        line: 7,
        reason: "a wrapper parses fdu output instead of recording the complete product response",
      },
    ]),
    [
      "golden observability check failed:",
      "- tests/golden/example.tryscript.md:7: a wrapper parses fdu output instead of recording the complete product response",
      "  invoke fdu directly and record its complete stable output; put relational assertions in a focused test",
    ].join("\n"),
  );
});
