import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { fileURLToPath } from "node:url";

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
const MINIMUM = "0.12.1";

function shellQuote(value) {
  return `'${value.replaceAll("'", `'"'"'`)}'`;
}

function runGuard(output, exitCode = 0) {
  const scratch = mkdtempSync(join(tmpdir(), "fdu-uv-version-"));
  const stub = join(scratch, "uv");
  try {
    writeFileSync(
      stub,
      `#!/bin/sh\nprintf '%s\\n' ${shellQuote(output)}\nexit ${exitCode}\n`,
      "utf8",
    );
    chmodSync(stub, 0o755);
    return spawnSync("make", ["--no-print-directory", "uv-version", `UV=${stub}`], {
      cwd: ROOT,
      encoding: "utf8",
    });
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

test("uv guard accepts only successful stable versions at or above the reviewed floor", () => {
  const missing = spawnSync(
    "make",
    ["--no-print-directory", "uv-version", "UV=/definitely/missing/fdu-test-uv"],
    { cwd: ROOT, encoding: "utf8" },
  );
  assert.notEqual(missing.status, 0);
  assert.match(missing.stdout, /uv is not installed/);
  assert.match(missing.stdout, new RegExp(`reviewed ${MINIMUM} release`));

  for (const version of [MINIMUM, "0.12.4", "1.0.0"]) {
    const result = runGuard(`uv ${version}`);
    assert.equal(result.status, 0, `${version}: ${result.stdout}${result.stderr}`);
  }

  for (const version of ["0.11.9", "0.11.27"]) {
    const result = runGuard(`uv ${version}`);
    assert.notEqual(result.status, 0, version);
    assert.match(result.stdout, new RegExp(`needs uv >= ${MINIMUM}`));
    // The pinned installer is the remedy that works regardless of how uv was
    // installed; `uv self update` is offered second because it fails outright when
    // an external manager owns the binary.
    assert.match(result.stdout, new RegExp(`astral\\.sh/uv/${MINIMUM}/install\\.sh`));
    assert.match(result.stdout, new RegExp(`uv self update ${MINIMUM}`));
  }

  for (const output of ["uv 0.12.1rc1", "uv 0.12.1.dev1", "uv malformed", "not-uv 1.0.0"]) {
    const result = runGuard(output);
    assert.notEqual(result.status, 0, output);
    assert.match(result.stdout, /could not determine a stable uv version/);
  }

  const failed = runGuard("uv 1.0.0", 42);
  assert.notEqual(failed.status, 0);
  assert.match(failed.stdout, /could not run uv --version/);
});

test("every directly uv-backed Make target depends on the version guard", () => {
  const makefile = readFileSync(join(ROOT, "Makefile"), "utf8");
  const uvBackedTargets = new Set();
  let currentTargets = [];
  for (const line of makefile.split("\n")) {
    if (!line.startsWith("\t")) {
      const match = line.match(/^([A-Za-z0-9_.-]+(?:\s+[A-Za-z0-9_.-]+)*):/);
      currentTargets = match ? match[1].split(/\s+/) : [];
      continue;
    }
    if (/\buvx?\b|\$\((?:UV|FLOWMARK|PERF_RUN|PERF_UV|SOFTSCHEMA)\)/.test(line)) {
      for (const target of currentTargets) {
        if (target !== "uv-version") uvBackedTargets.add(target);
      }
    }
  }

  const database = spawnSync("make", ["-qp"], { cwd: ROOT, encoding: "utf8" }).stdout;
  assert(uvBackedTargets.size > 0);
  for (const target of uvBackedTargets) {
    const rule = database.split("\n").find((line) => line.startsWith(`${target}:`));
    assert.match(rule ?? "", /(?:^|\s)uv-version(?:\s|$)/, `${target}: ${rule}`);
  }
});

test("every environment a wheel smoke creates names a GIL-enabled interpreter", () => {
  // uv picks a free-threaded CPython when it manages one, and the cp312-abi3 wheel cannot
  // install there, so an environment created without --python fails the gate on such a
  // host (fdu-pd1b). The default is CI's 3.12; UV_PYTHON chooses another.
  const targets = ["python-smoke", "python-sdist-smoke", "parity-venv"];
  const requests = [
    [undefined, "3.12"],
    ["3.13", "3.13"],
    // Splitting uv's full request form on its dashes must not invent a free-threaded suffix.
    ["cpython-3.12.11-macos-aarch64-none", "cpython-3.12.11-macos-aarch64-none"],
  ];
  for (const [override, expected] of requests) {
    const env = { ...process.env };
    delete env.UV_PYTHON;
    if (override) env.UV_PYTHON = override;
    for (const target of targets) {
      const result = spawnSync("make", ["--no-print-directory", "-n", target], {
        cwd: ROOT,
        encoding: "utf8",
        env,
      });
      assert.equal(result.status, 0, result.stderr);
      const creations = result.stdout.split("\n").filter((line) => /(?:^|\s)venv\s/.test(line));
      assert(creations.length > 0, `${target} creates no environment`);
      for (const line of creations) {
        assert.match(line, new RegExp(`\\s--python ${expected.replaceAll(".", "\\.")}\\s`), `${target}: ${line}`);
      }
    }
  }
});

test("a free-threaded interpreter request is refused before any environment is created", () => {
  // Asking for one explicitly would otherwise reach uv, whose failure names wheel tags
  // rather than the request (fdu-pd1b).
  const targets = ["python-smoke", "python-sdist-smoke", "parity-venv"];
  // uv reads a `t` after the version, or `td` for the debug build, as free-threaded in
  // every request form, and so does `+freethreaded`.
  const requests = [
    [{ UV_PYTHON: "3.14t" }, [], "3.14t"],
    [{ UV_PYTHON: "cpython-3.14t-macos-aarch64-none" }, [], "cpython-3\\.14t-macos-aarch64-none"],
    [{}, ["WHEEL_PYTHON=3.14td"], "3\\.14td"],
    [{}, ["WHEEL_PYTHON=cpython-3.14+freethreaded"], "cpython-3.14\\+freethreaded"],
  ];
  for (const [overrides, variables, shown] of requests) {
    const env = { ...process.env, ...overrides };
    if (!overrides.UV_PYTHON) delete env.UV_PYTHON;
    for (const target of targets) {
      const result = spawnSync("make", ["--no-print-directory", "-n", target, ...variables], {
        cwd: ROOT,
        encoding: "utf8",
        env,
      });
      assert.notEqual(result.status, 0, `${target}: ${result.stdout}`);
      assert.match(result.stderr, new RegExp(`WHEEL_PYTHON=${shown} is a free-threaded CPython`), target);
      assert.doesNotMatch(result.stdout, /(?:^|\s)venv\s/, target);
    }
  }
});

test("make check refuses a free-threaded request before anything but the uv floor runs", () => {
  // Reached only through the Python targets, the refusal would follow the whole Rust gate.
  const env = { ...process.env, UV_PYTHON: "3.14t" };
  const make = (target) =>
    spawnSync("make", ["--no-print-directory", "-j1", "-n", target], { cwd: ROOT, encoding: "utf8", env });
  const check = make("check");
  assert.notEqual(check.status, 0, check.stdout);
  assert.match(check.stderr, /WHEEL_PYTHON=3\.14t is a free-threaded CPython/);
  const floor = make("uv-version");
  assert.equal(floor.status, 0, floor.stderr);
  assert.equal(check.stdout, floor.stdout);
});

test("the bootstrap policy enforces one reviewed uv version in Make and CI", () => {
  const policy = JSON.parse(readFileSync(join(ROOT, "supply-chain-policy.json"), "utf8"));
  const uvRelease = policy.bootstrap.githubReleases.find(
    (release) => release.repository === "astral-sh/uv",
  );
  assert(uvRelease);
  assert.deepEqual(
    new Set(uvRelease.files),
    new Set([".github/workflows/ci.yml", ".github/workflows/release.yml", "Makefile"]),
  );

  const makefile = readFileSync(join(ROOT, "Makefile"), "utf8");
  const makeVersion = makefile.match(/^UV_MIN_VERSION := (\S+)$/m)?.[1];
  assert.equal(makeVersion, uvRelease.version);
  assert.equal(uvRelease.tag, uvRelease.version);

  const workflowFiles = uvRelease.files.filter((file) => file.endsWith(".yml"));
  const workflowVersions = workflowFiles.flatMap((file) => {
    const workflow = readFileSync(join(ROOT, file), "utf8");
    const versions = [...workflow.matchAll(
      /uses: astral-sh\/setup-uv@[^\n]+\n\s+with:\n\s+version: "([^"]+)"/g,
    )].map((match) => match[1]);
    assert(versions.length > 0, `${file} inventories uv but has no setup-uv pin`);
    return versions;
  });
  assert.deepEqual(new Set(workflowVersions), new Set([uvRelease.version]));
});
