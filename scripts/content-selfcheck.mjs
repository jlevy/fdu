#!/usr/bin/env node

/**
 * Analyze exactly the tracked files at HEAD and assert cross-layer content invariants.
 *
 * A git archive excludes target/, caches, virtual environments, and working-tree edits.
 * That makes this a repeatable repository sanity check instead of a benchmark sample.
 */

import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
const scratch = mkdtempSync(join(tmpdir(), "fdu-content-selfcheck-"));
const archive = join(scratch, "tracked.tar");
const tree = join(scratch, "tree");

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    ...options,
  });
  if (result.status !== 0) {
    throw new Error(
      `${command} ${args.join(" ")} failed (${result.status})\n${result.stderr || result.stdout}`,
    );
  }
  return result.stdout;
}

try {
  run("git", ["archive", "--format=tar", `--output=${archive}`, "HEAD"]);
  run("mkdir", [tree]);
  run("tar", ["-xf", archive, "-C", tree]);

  const output = run(resolve(root, "target/debug/fdu"), [
    "--cache",
    "off",
    "--analyze",
    "basic",
    "--view",
    "types,families,languages,documents",
    "--format",
    "json",
    "--size",
    "apparent",
    tree,
  ]);
  const report = JSON.parse(output);
  assert.equal(report.schema, "fdu.report/2");
  assert.equal(report.complete, true);
  assert.equal(report.analysis.profile, "basic");
  assert.deepEqual(report.analysis.analyzers, [
    { id: "content-basic-v1", version: 1 },
  ]);

  const sections = new Map(report.reports.map((section) => [section.view, section]));
  const types = sections.get("types").metrics;
  const families = sections.get("families").metrics;
  const languages = sections.get("languages").metrics;
  const documents = sections.get("documents").metrics;

  const typeRows = new Map(types.rows.map((row) => [row.id, row]));
  for (const id of ["rust", "python", "javascript", "markdown", "toml", "json", "yaml"])
    assert.ok(typeRows.has(id), `missing tracked file type ${id}`);
  for (const row of types.rows) assert.ok(row.files > 0, `empty type row ${row.id}`);

  const familyRows = new Map(families.rows.map((row) => [row.id, row]));
  for (const id of ["code", "prose", "data"])
    assert.ok(familyRows.has(id), `missing tracked family ${id}`);
  assert.equal(languages.total.family, "unknown");
  assert.equal(documents.total.family, "unknown");
  assert.ok(languages.total.files >= typeRows.get("rust").files);
  assert.ok(documents.total.files >= typeRows.get("markdown").files);

  for (const summary of [types, families, languages, documents]) {
    assert.equal(
      summary.total.metrics.physical_lines,
      summary.total.metrics.blank_lines + summary.total.metrics.nonblank_lines,
      `${summary.group} line partition`,
    );
    const covered = Object.values(summary.total.coverage).reduce((sum, count) => sum + count, 0);
    assert.equal(covered, summary.total.files, `${summary.group} coverage partition`);
  }

  const binary = familyRows.get("binary");
  if (binary) {
    assert.equal(binary.analyzed_files, 0);
    assert.equal(binary.coverage.binary, binary.files);
    assert.deepEqual(binary.metrics, {
      physical_lines: 0,
      blank_lines: 0,
      nonblank_lines: 0,
      code_lines: 0,
      comment_lines: 0,
      raw_words: 0,
      logical_words: 0,
      paragraphs: 0,
      visible_words: 0,
      visible_logical_words: 0,
    });
  }

  console.log(
    `content self-check passed: ${types.total.files} tracked files, ` +
      `${types.total.metrics.physical_lines} text lines, ${types.rows.length} types`,
  );
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
