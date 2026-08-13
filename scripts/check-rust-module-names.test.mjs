import assert from "node:assert/strict";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import test from "node:test";

import {
  auditRustModuleNames,
  collectRustFiles,
  formatViolations,
} from "./check-rust-module-names.mjs";

async function withRustTree(paths, run) {
  const root = await mkdtemp(join(tmpdir(), "fdu-rust-module-names-"));
  try {
    for (const path of paths) {
      const absolute = join(root, path);
      await mkdir(dirname(absolute), { recursive: true });
      await writeFile(absolute, "// test fixture\n", "utf8");
    }
    await run(root);
  } finally {
    await rm(root, { force: true, recursive: true });
  }
}

test("collects crate Rust surfaces but excludes fixture source trees", async () => {
  await withRustTree(
    [
      "crates/alpha/build.rs",
      "crates/alpha/src/lib.rs",
      "crates/alpha/src/content/content_cache.rs",
      "crates/alpha/tests/query_report_integration.rs",
      "crates/alpha/examples/perf_probe.rs",
      "crates/alpha/tests/fixtures/sample/src/cache.rs",
      "tests/golden/fixtures/sample/src/main.rs",
    ],
    async (root) => {
      const files = await collectRustFiles(root);
      assert.deepEqual(
        files.map((path) => relative(root, path)),
        [
          "crates/alpha/build.rs",
          "crates/alpha/examples/perf_probe.rs",
          "crates/alpha/src/content/content_cache.rs",
          "crates/alpha/src/lib.rs",
          "crates/alpha/tests/query_report_integration.rs",
        ],
      );
    },
  );
});

test("accepts clear names and repeated standard crate roots", async () => {
  await withRustTree(
    [
      "crates/alpha/build.rs",
      "crates/alpha/src/lib.rs",
      "crates/alpha/src/main.rs",
      "crates/alpha/src/content/content_cache.rs",
      "crates/beta/build.rs",
      "crates/beta/src/lib.rs",
      "crates/beta/src/query/query_report.rs",
    ],
    async (root) => {
      const files = await collectRustFiles(root);
      assert.deepEqual(auditRustModuleNames(files, root), []);
    },
  );
});

test("rejects legacy module roots and context-dependent catch-all names", async () => {
  await withRustTree(
    [
      "crates/alpha/src/content/mod.rs",
      "crates/alpha/src/types.rs",
      "crates/alpha/src/query/parse.rs",
    ],
    async (root) => {
      const files = await collectRustFiles(root);
      assert.deepEqual(
        auditRustModuleNames(files, root).map(({ kind, basename }) => ({ kind, basename })),
        [
          { kind: "legacy-module-root", basename: "mod.rs" },
          { kind: "forbidden-basename", basename: "parse.rs" },
          { kind: "forbidden-basename", basename: "types.rs" },
        ],
      );
    },
  );
});

test("rejects duplicate non-root basenames with every conflicting path", async () => {
  await withRustTree(
    ["crates/alpha/src/cache.rs", "crates/alpha/src/content/cache.rs"],
    async (root) => {
      const files = await collectRustFiles(root);
      assert.deepEqual(auditRustModuleNames(files, root), [
        {
          kind: "duplicate-basename",
          basename: "cache.rs",
          paths: ["crates/alpha/src/cache.rs", "crates/alpha/src/content/cache.rs"],
        },
      ]);
    },
  );
});

test("rejects crate-root names outside their standard locations", async () => {
  await withRustTree(["crates/alpha/src/query/lib.rs"], async (root) => {
    const files = await collectRustFiles(root);
    assert.deepEqual(auditRustModuleNames(files, root), [
      {
        kind: "misplaced-crate-root",
        basename: "lib.rs",
        paths: ["crates/alpha/src/query/lib.rs"],
      },
    ]);
  });
});

test("formats deterministic actionable diagnostics", () => {
  assert.equal(
    formatViolations([
      {
        kind: "duplicate-basename",
        basename: "cache.rs",
        paths: ["crates/alpha/src/cache.rs", "crates/alpha/src/content/cache.rs"],
      },
    ]),
    [
      "Rust module filename check failed:",
      '- duplicate basename "cache.rs"; use subsystem-specific responsibility names:',
      "  - crates/alpha/src/cache.rs",
      "  - crates/alpha/src/content/cache.rs",
    ].join("\n"),
  );
});
