import assert from "node:assert/strict";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
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

function ownerRun(target) {
  return spawnSync("make", ["--no-print-directory", "target-owner", `CARGO_TARGET=${target}`], {
    cwd: ROOT,
    encoding: "utf8",
  });
}

test("a target directory another checkout built is rebuilt from this one (fdu-8whh)", () => {
  const scratch = mkdtempSync(join(tmpdir(), "fdu-target-owner-"));
  const target = join(scratch, "target");
  const stamp = join(target, ".fdu-checkout");
  const workspace = [
    join(target, "debug", ".fingerprint", "fdu-core-0123456789abcdef"),
    join(target, "debug", ".fingerprint", "fdu-fedcba9876543210"),
    join(target, "release", ".fingerprint", "fdu-py-0123456789abcdef"),
    join(target, "x86_64-pc-windows-msvc", "debug", ".fingerprint", "fdu-core-0123456789abcdef"),
  ];
  const dependency = join(target, "debug", ".fingerprint", "serde-0123456789abcdef");
  const plant = () => {
    for (const directory of [...workspace, dependency]) {
      mkdirSync(directory, { recursive: true });
      writeFileSync(join(directory, "lib-x"), "fingerprint");
    }
  };
  try {
    // Built before the check existed: nobody is recorded, so nothing can be trusted.
    plant();
    let result = ownerRun(target);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /an unrecorded checkout/);
    for (const directory of workspace) assert(!existsSync(directory), directory);
    assert(existsSync(dependency), "a dependency's fingerprint is kept");
    assert.equal(readFileSync(stamp, "utf8").trim(), ROOT);

    // This checkout's own build is left alone.
    plant();
    result = ownerRun(target);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, "");
    for (const directory of workspace) assert(existsSync(directory), directory);

    // Another checkout built here since: its outputs are not this checkout's.
    writeFileSync(stamp, "/elsewhere/other-worktree\n");
    result = ownerRun(target);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /last built from \/elsewhere\/other-worktree/);
    for (const directory of workspace) assert(!existsSync(directory), directory);
    assert(existsSync(dependency));
    assert.equal(readFileSync(stamp, "utf8").trim(), ROOT);

    // A directory that does not exist yet is simply claimed.
    const fresh = join(scratch, "fresh");
    result = ownerRun(fresh);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, "");
    assert.equal(readFileSync(join(fresh, ".fdu-checkout"), "utf8").trim(), ROOT);
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
});

test("every target that compiles a workspace crate first checks who owns the target directory", () => {
  const makefile = readFileSync(join(ROOT, "Makefile"), "utf8");
  const compiles =
    /\$\(CARGO\)(?:\s+\+\$\(MSRV\))?\s+(?:build|test|clippy|doc|check|package|run)\b|maturin build|--group dev pytest|run_concurrency\.py/;
  const builders = new Set();
  let currentTargets = [];
  for (const line of makefile.split("\n")) {
    if (!line.startsWith("\t")) {
      const match = line.match(/^([A-Za-z0-9_.-]+(?:\s+[A-Za-z0-9_.-]+)*):(?!=)/);
      if (match) currentTargets = match[1].split(/\s+/);
      else if (line.trim() !== "" && !line.startsWith("#")) currentTargets = [];
      continue;
    }
    if (compiles.test(line)) for (const target of currentTargets) builders.add(target);
  }
  assert(builders.has("build") && builders.has("python-check"), [...builders].join(" "));
  const database = spawnSync("make", ["-qp"], { cwd: ROOT, encoding: "utf8" }).stdout;
  for (const target of builders) {
    const rule = database.split("\n").find((line) => line.startsWith(`${target}:`));
    assert.match(rule ?? "", /(?:^|\s)target-owner(?:\s|$)/, `${target}: ${rule}`);
  }
});

test("the sdist smoke builds the sdist in a target directory of its own", () => {
  const recipe = dryRun("python-sdist-smoke", { CARGO_TARGET_DIR: ELSEWHERE });
  const install = recipe.split("\n").find((line) => /\bpip install\b/.test(line));
  assert.match(install ?? "", /env -u CARGO_TARGET_DIR \S*uv pip install/, recipe);
});
