#!/usr/bin/env node
// Audit the deterministic opened-root session corpus without interpreting its behavior.

import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
const SOURCE = join(ROOT, "crates/fdu-core/src/opened/golden_tests.rs");
const DIRECTORY = join(ROOT, "crates/fdu-core/tests/golden/opened-root");
const MAX_SCENARIO_LINES = 400;
const MAX_CORPUS_LINES = 2_000;
const MAX_SCENARIO_BYTES = 256 * 1024;
const MAX_CORPUS_BYTES = 768 * 1024;
const ALLOWED_TOKENS = new Set([
  "[ALLOCATED]",
  "[DEVICE]",
  "[DIR_SIZE]",
  "[INODE]",
  "[SYSTEM_TIME]",
  "[TIME]",
]);

export function scenarioNames(source) {
  return [...source.matchAll(/SessionTrace::new\("([a-z0-9-]+)"/g)].map((match) => match[1]);
}

export function auditGolden(name, source) {
  const findings = [];
  const lines = source.endsWith("\n") ? source.slice(0, -1).split("\n") : source.split("\n");
  if (lines.length > MAX_SCENARIO_LINES) {
    findings.push(`${name}: ${lines.length} lines exceeds ${MAX_SCENARIO_LINES}`);
  }
  if (Buffer.byteLength(source) > MAX_SCENARIO_BYTES) {
    findings.push(`${name}: artifact exceeds ${MAX_SCENARIO_BYTES} bytes`);
  }
  if (lines[0] !== `scenario: schema=1 name=${name}`) {
    findings.push(`${name}: first record does not declare its schema and exact scenario name`);
  }
  for (const required of ["action.open:", "result.open:", "action.close:", "final:"]) {
    if (!lines.some((line) => line.startsWith(required))) {
      findings.push(`${name}: missing ${required.slice(0, -1)} record`);
    }
  }
  if (/SessionId\(\d/.test(source)) {
    findings.push(`${name}: contains an unnormalized opened-root identity`);
  }
  if (/"\/(?!\$)/.test(source) || /"[A-Za-z]:\\/.test(source)) {
    findings.push(`${name}: contains an absolute machine path`);
  }
  if (/\b(?:mtime_ns|ctime_ns|inode|dev|allocated): -?\d/.test(source)) {
    findings.push(`${name}: contains an unnormalized platform-assigned attribute`);
  }
  if (/newest_mtime_ns: Some\(-?\d/.test(source)) {
    findings.push(`${name}: contains an unnormalized aggregate timestamp`);
  }
  for (const token of source.match(/\[[A-Z_]+\]/g) ?? []) {
    if (!ALLOWED_TOKENS.has(token)) {
      findings.push(`${name}: uses unknown normalization token ${token}`);
    }
  }
  if (!source.endsWith("\n")) {
    findings.push(`${name}: missing final newline`);
  }
  return findings;
}

export function auditCorpus() {
  const expected = scenarioNames(readFileSync(SOURCE, "utf8"));
  const duplicates = expected.filter((name, index) => expected.indexOf(name) !== index);
  const findings = duplicates.map((name) => `scenario source declares ${name} more than once`);
  const actual = readdirSync(DIRECTORY)
    .filter((name) => name.endsWith(".golden"))
    .map((name) => name.slice(0, -".golden".length))
    .sort();
  const expectedSorted = [...expected].sort();
  for (const missing of expectedSorted.filter((name) => !actual.includes(name))) {
    findings.push(`missing golden for scenario ${missing}`);
  }
  for (const orphan of actual.filter((name) => !expectedSorted.includes(name))) {
    findings.push(`orphan golden without a scenario ${orphan}`);
  }

  let totalLines = 0;
  let totalBytes = 0;
  const recordedSources = new Map();
  for (const name of actual) {
    const path = join(DIRECTORY, `${name}.golden`);
    const source = readFileSync(path, "utf8");
    totalLines += source.endsWith("\n") ? source.slice(0, -1).split("\n").length : source.split("\n").length;
    totalBytes += statSync(path).size;
    findings.push(...auditGolden(name, source));
    if (recordedSources.has(source)) {
      findings.push(`${name}: duplicates ${recordedSources.get(source)} byte for byte`);
    } else {
      recordedSources.set(source, name);
    }
  }
  if (totalLines > MAX_CORPUS_LINES) {
    findings.push(`corpus: ${totalLines} lines exceeds ${MAX_CORPUS_LINES}`);
  }
  if (totalBytes > MAX_CORPUS_BYTES) {
    findings.push(`corpus: ${totalBytes} bytes exceeds ${MAX_CORPUS_BYTES}`);
  }
  return { findings, expected: expectedSorted, totalLines, totalBytes };
}

function main() {
  const scenarioIndex = process.argv.indexOf("--scenario");
  if (scenarioIndex >= 0) {
    const requested = process.argv[scenarioIndex + 1];
    const known = scenarioNames(readFileSync(SOURCE, "utf8"));
    if (!requested || !known.includes(requested)) {
      console.error(`unknown opened-root golden scenario ${JSON.stringify(requested)}; known: ${known.join(", ")}`);
      process.exitCode = 2;
    }
    return;
  }

  const { findings, totalBytes, totalLines } = auditCorpus();
  if (findings.length > 0) {
    console.error("opened-root golden audit failed:");
    for (const finding of findings) console.error(`- ${finding}`);
    process.exitCode = 1;
    return;
  }
  console.log(`opened-root goldens ok: 5 sessions, ${totalLines} records, ${totalBytes} bytes`);
}

if (process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url) main();
