import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { cargoTargetDir, debugFdu } from "./cargo-target.mjs";

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
// A directory that is not `<checkout>/target`, the way AGENTS.md asks worktrees to build.
const ELSEWHERE = join(tmpdir(), "fdu-elsewhere-target");

function dryRun(target, env = {}) {
  const result = spawnSync("make", ["--no-print-directory", "-n", target], {
    cwd: ROOT,
    encoding: "utf8",
    env: { ...process.env, ...env },
  });
  assert.equal(result.status, 0, `${target}: ${result.stderr}`);
  return result.stdout;
}

test("the scripts find the build where CARGO_TARGET_DIR put it", () => {
  const env = { ...process.env, CARGO_TARGET_DIR: ELSEWHERE };
  assert.equal(cargoTargetDir(ROOT, env), ELSEWHERE);
  const exe = process.platform === "win32" ? ".exe" : "";
  assert.equal(debugFdu(ROOT, env), join(ELSEWHERE, "debug", `fdu${exe}`));
});

test("every gate that runs a built binary runs the one cargo built (fdu-bi9a, fdu-dfbu)", () => {
  const env = { CARGO_TARGET_DIR: ELSEWHERE };
  const consumers = {
    "path-independence": `FDU_BIN="${ELSEWHERE}/debug/fdu"`,
    "test-terminal": `FDU_BIN="\${FDU_BIN:-${ELSEWHERE}/debug/fdu}"`,
    "test-performance": `CARGO_TARGET_DIR="${ELSEWHERE}"`,
    "release-rehearse": `"${ELSEWHERE}/package/fdu-core-$version.crate"`,
  };
  for (const [target, expected] of Object.entries(consumers)) {
    const recipe = dryRun(target, env);
    assert(recipe.includes(expected), `${target} does not use ${ELSEWHERE}:\n${recipe}`);
    assert(!recipe.includes(join(ROOT, "target")), `${target} still names ${ROOT}/target`);
  }
});

test("no gate names a literal target/ build path", () => {
  // `$(CARGO_TARGET)/debug` is the answer; `target/debug` is the assumption it replaced.
  const literal = /(?<![\w)$}./-])target\/(?:debug|release|profiling|package)\b/;
  const joined = /join\(\s*root\s*,\s*['"]target['"]/;
  const sources = [
    "Makefile",
    ...readdirSync(join(ROOT, "scripts"))
      // cargo-target.mjs holds the one fallback, for a cargo that cannot answer.
      .filter((name) => name.endsWith(".mjs") && !name.endsWith(".test.mjs"))
      .filter((name) => name !== "cargo-target.mjs")
      .map((name) => join("scripts", name)),
  ];
  for (const source of sources) {
    readFileSync(join(ROOT, source), "utf8")
      .split("\n")
      .forEach((line, index) => {
        if (/^\s*(#|\/\/|\*)/.test(line)) return;
        assert(!literal.test(line) && !joined.test(line), `${source}:${index + 1}: ${line.trim()}`);
      });
  }
});
