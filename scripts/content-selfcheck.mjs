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
    "words",
    "--view",
    "types,families,documents",
    "--format",
    "json",
    "--size",
    "apparent",
    tree,
  ]);
  const report = JSON.parse(output);
  assert.equal(report.schema, "fdu.report/3");
  assert.equal(report.complete, true);
  assert.deepEqual(report.analysis.analyze, ["lines", "words"]);
  assert.deepEqual(report.analysis.analyzers, [
    { id: "content-basic-v1", version: 1 },
    { id: "text-logical-v1", version: 1 },
    { id: "markdown-prose-v1", version: 1 },
  ]);

  const sections = new Map(report.reports.map((section) => [section.view, section]));
  const types = sections.get("types").metrics;
  const families = sections.get("families").metrics;
  const documents = sections.get("documents").metrics;

  const typeRows = new Map(types.rows.map((row) => [row.id, row]));
  for (const id of ["rust", "python", "javascript", "markdown", "toml", "json", "yaml"])
    assert.ok(typeRows.has(id), `missing tracked file type ${id}`);
  for (const row of types.rows) assert.ok(row.files > 0, `empty type row ${row.id}`);

  const familyRows = new Map(families.rows.map((row) => [row.id, row]));
  for (const id of ["code", "prose", "data"])
    assert.ok(familyRows.has(id), `missing tracked family ${id}`);
  assert.equal(documents.total.family, "unknown");
  assert.ok(documents.total.files >= typeRows.get("markdown").files);
  assert.equal(documents.share_metric, "document_words");
  assert.equal(documents.words_per_page, 250);
  assert.equal(documents.total.pages.words, documents.total.metrics.document_words);
  assert.ok(documents.total.metrics.raw_words > 0);
  assert.ok(documents.total.metrics.logical_words > 0);
  assert.ok(documents.total.metrics.visible_words > 0);
  assert.ok(documents.total.metrics.visible_logical_words > 0);
  assert.ok(documents.total.metrics.paragraphs > 0);
  const documentRows = new Map(documents.rows.map((row) => [row.id, row]));
  const markdown = documentRows.get("markdown");
  assert.ok(markdown, "missing self-host Markdown document row");
  assert.ok(markdown.metrics.visible_words > 0);
  assert.equal(markdown.metrics.document_words, markdown.metrics.visible_logical_words);
  assert.equal(markdown.pages.words, markdown.metrics.document_words);

  for (const summary of [types, families, documents]) {
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
      code_blank_lines: 0,
      raw_words: 0,
      logical_words: 0,
      paragraphs: 0,
      visible_words: 0,
      visible_logical_words: 0,
      document_words: 0,
    });
  }

  const codeOutput = run(resolve(root, "target/debug/fdu"), [
    "--cache",
    "off",
    "--allow-partial",
    "--analyze",
    "code",
    "--view",
    "languages",
    "--format",
    "json",
    "--limit",
    "all",
    tree,
  ]);
  const codeReport = JSON.parse(codeOutput);
  assert.equal(codeReport.schema, "fdu.report/3");
  assert.deepEqual(codeReport.analysis.analyze, ["lines", "code"]);
  assert.deepEqual(codeReport.analysis.analyzers, [
    { id: "content-basic-v1", version: 1 },
    { id: "code-sloc-v1", version: 1 },
  ]);
  const code = codeReport.reports[0].metrics;
  assert.equal(code.share_metric, "code_lines");
  assert.equal(code.total.share.numerator, code.total.metrics.code_lines);
  assert.equal(code.total.share.denominator, code.total.metrics.code_lines);
  assert.ok(code.total.metrics.code_lines > 0);
  assert.ok(code.total.coverage.unsupported > 0, "self-host should expose unsupported code");
  const codeRows = new Map(code.rows.map((row) => [row.id, row]));
  for (const id of ["rust", "python", "javascript"]) {
    const row = codeRows.get(id);
    assert.ok(row, `missing self-host SLOC row ${id}`);
    assert.equal(row.coverage.analyzed, row.files);
    assert.equal(
      row.metrics.physical_lines,
      row.metrics.code_lines + row.metrics.comment_lines + row.metrics.code_blank_lines,
      `${id} SLOC partition`,
    );
  }

  console.log(
    `content self-check passed: ${types.total.files} tracked files, ` +
      `${types.total.metrics.physical_lines} text lines, ` +
      `${code.total.metrics.code_lines} standard LOC, ${types.rows.length} types`,
  );
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
