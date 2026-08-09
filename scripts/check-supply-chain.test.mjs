import assert from "node:assert/strict";
import test from "node:test";

import {
  assertAged,
  assertEqualProvenance,
  fetchWithRetry,
  parseActionUses,
  parseCargoLock,
  parseUvLock,
  selectGithubToken,
  validateDownloadScripts,
  validateExceptions,
  validateWorkflowSecurity,
} from "./check-supply-chain.mjs";

const NOW = new Date("2026-08-09T12:00:00Z");

test("fresh and undated releases fail closed", () => {
  assert.throws(
    () =>
      assertAged(
        { ecosystem: "cargo", name: "example", version: "1.0.0", publishedAt: "2026-08-01T00:00:00Z" },
        NOW,
        14,
        [],
      ),
    /has not cleared the 14-day cool-off/,
  );
  assert.throws(
    () =>
      assertAged(
        { ecosystem: "npm", name: "example", version: "1.0.0", publishedAt: undefined },
        NOW,
        14,
        [],
      ),
    /missing a valid publication time/,
  );
});

test("only a complete, matching, unexpired exception bypasses release age", () => {
  const exception = {
    id: "first-party-example",
    ecosystem: "npm",
    name: "example",
    version: "1.0.0",
    reason: "First-party tool pinned to the reviewed release.",
    approvedBy: "Repository maintainers in AGENTS.md",
    approvedAt: "2026-08-01T00:00:00Z",
    expiresAt: "2026-08-15T00:00:00Z",
    followUp: "Remove the exception after the release clears the normal cool-off.",
  };

  validateExceptions([exception]);
  assert.doesNotThrow(() =>
    assertAged(
      { ecosystem: "npm", name: "example", version: "1.0.0", publishedAt: "2026-08-01T00:00:00Z" },
      NOW,
      14,
      [exception],
    ),
  );
  assert.throws(() => validateExceptions([{ ...exception, followUp: "" }]), /followUp/);
  assert.throws(
    () =>
      assertAged(
        { ecosystem: "npm", name: "example", version: "2.0.0", publishedAt: "2026-08-01T00:00:00Z" },
        NOW,
        14,
        [exception],
      ),
    /has not cleared/,
  );
});

test("provenance mismatch is never an age exception", () => {
  assert.throws(
    () => assertEqualProvenance("crate checksum", "sha256:expected", "sha256:actual"),
    /crate checksum mismatch/,
  );
});

test("provenance requests retry only transient failures", async () => {
  const statuses = [503, 429, 200];
  const attempts = [];
  const response = await fetchWithRetry(
    "https://example.invalid/provenance",
    {},
    async () => {
      const status = statuses.shift();
      attempts.push(status);
      return { ok: status === 200, status };
    },
    async () => {},
  );
  assert.equal(response.status, 200);
  assert.deepEqual(attempts, [503, 429, 200]);

  await assert.rejects(
    () =>
      fetchWithRetry(
        "https://example.invalid/missing",
        {},
        async () => ({ ok: false, status: 404 }),
        async () => {},
      ),
    /failed \(404\)/,
  );
});

test("GitHub provenance prefers explicit tokens and supports local gh credentials", () => {
  assert.equal(
    selectGithubToken({ GITHUB_TOKEN: " workflow-token ", GH_TOKEN: "gh-token" }, "local-token"),
    "workflow-token",
  );
  assert.equal(selectGithubToken({ GH_TOKEN: " gh-token " }, "local-token"), "gh-token");
  assert.equal(selectGithubToken({}, " local-token\n"), "local-token");
  assert.equal(selectGithubToken({}, ""), undefined);
});

test("every shell downloader is explicitly inventoried", () => {
  const files = [
    { path: "scripts/reviewed.sh", text: "curl https://example.invalid/tool" },
    { path: "scripts/local.sh", text: "cargo test" },
  ];
  assert.doesNotThrow(() => validateDownloadScripts(files, new Set(["scripts/reviewed.sh"])));
  assert.throws(() => validateDownloadScripts(files, new Set()), /absent from the bootstrap inventory/);
});

test("GitHub actions must use an exact commit", () => {
  assert.throws(
    () => parseActionUses([{ path: ".github/workflows/ci.yml", text: "- uses: actions/checkout@v6\n" }]),
    /must pin an exact 40-character commit/,
  );
  assert.deepEqual(
    parseActionUses([
      {
        path: ".github/workflows/ci.yml",
        text: "- uses: actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803\n",
      },
    ]),
    [
      {
        path: ".github/workflows/ci.yml",
        repository: "actions/checkout",
        revision: "d23441a48e516b6c34aea4fa41551a30e30af803",
      },
    ],
  );
});

test("pull-request workflows use least privilege and gate all executable jobs", () => {
  const revision = "d23441a48e516b6c34aea4fa41551a30e30af803";
  const valid = `name: CI
on:
  pull_request:
permissions:
  contents: read
jobs:
  supply-chain:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@${revision}
        with:
          persist-credentials: false
      - run: node --test scripts/check-supply-chain.test.mjs
      - run: node scripts/check-supply-chain.mjs
        env:
          GITHUB_TOKEN: \${{ github.token }}
  test:
    needs: supply-chain
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@${revision}
        with:
          persist-credentials: false
      - run: npm ci --ignore-scripts
`;
  assert.doesNotThrow(() =>
    validateWorkflowSecurity([{ path: ".github/workflows/ci.yml", text: valid }]),
  );
  assert.throws(
    () =>
      validateWorkflowSecurity([
        { path: ".github/workflows/ci.yml", text: valid.replace("contents: read", "contents: write") },
      ]),
    /grant only contents: read/,
  );
  assert.throws(
    () =>
      validateWorkflowSecurity([
        {
          path: ".github/workflows/ci.yml",
          text: valid.replace("persist-credentials: false", "persist-credentials: true"),
        },
      ]),
    /persist-credentials: false/,
  );
  assert.throws(
    () =>
      validateWorkflowSecurity([
        { path: ".github/workflows/ci.yml", text: valid.replace("npm ci --ignore-scripts", "npm ci\n        cache: npm") },
      ]),
    /cache writes are forbidden/,
  );
  assert.throws(
    () =>
      validateWorkflowSecurity([
        {
          path: ".github/workflows/ci.yml",
          text: valid.replace("          GITHUB_TOKEN: \${{ github.token }}\n", ""),
        },
      ]),
    /must authenticate with the least-privilege workflow token/,
  );
});

test("Cargo registry entries require checksums", () => {
  const lock = `version = 4

[[package]]
name = "example"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
`;
  assert.throws(() => parseCargoLock(lock), /example@1.0.0 is missing a checksum/);
});

test("uv registry artifacts require hashes and upload times", () => {
  const lock = `version = 1
revision = 3

[[package]]
name = "example"
version = "1.0.0"
source = { registry = "https://pypi.org/simple" }
wheels = [
    { url = "https://files.pythonhosted.org/example.whl", hash = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" },
]
`;
  assert.throws(() => parseUvLock(lock), /example@1.0.0 artifact is missing an upload time/);
});
