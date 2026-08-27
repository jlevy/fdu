#!/usr/bin/env node
// Keep transparent-box goldens transparent.
//
// A golden is useful because a reviewer can see the complete stable behavior that the
// product exposed. A wrapper that parses a broad response and prints selected fields
// turns that artifact into a disguised assertion: adjacent errors, state, totals, and
// warnings can change without changing the golden. Relational or cost assertions still
// belong in focused tests; fixture-setup scripts remain valid because they do not hide
// product output.

import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, relative, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const SCRIPT_DIRECTORY = dirname(fileURLToPath(import.meta.url));
const ROOT = dirname(SCRIPT_DIRECTORY);
const GOLDEN_DIRECTORY = join(ROOT, "tests", "golden");

const SHELL_FILTER = /(?:^|[|;&][ \t]*)\b(?:grep|jq|head|tail|sed|awk)\b/;
const SHELL_REDIRECT = /(?:^|[ \t])1?>[ \t]*(?:"([^"]+)"|'([^']+)'|([^\s;&|]+))/g;
const WRAPPER_INVOCATION = /(?:execFileSync|spawnSync)\(process\.env\.FDU\b/;
const OUTPUT_PARSER = /(?:JSON\.parse|yaml\.parse|parseYaml|\.stdout\s*\.(?:split|match)|\.stdout\s*\[)/;

export function auditGoldenText(source, file) {
  const findings = [];
  const helperRunsFdu = file.includes("/bin/") && /process\.env\.FDU\b/.test(source);
  const redirectedFduOutputs = new Set();

  source.split("\n").forEach((line, index) => {
    const command = line.replace(/^\s*\$\s+/, "");
    const invokesFdu = /(?:\bfdu\b|\$FDU|process\.env\.FDU)/.test(command);
    if (invokesFdu) {
      for (const match of command.matchAll(SHELL_REDIRECT)) {
        redirectedFduOutputs.add(match[1] ?? match[2] ?? match[3]);
      }
    }
    const filtersRedirectedOutput = [...redirectedFduOutputs].some((path) =>
      command.includes(path),
    );

    if (SHELL_FILTER.test(command) && (invokesFdu || filtersRedirectedOutput)) {
      findings.push({
        file,
        line: index + 1,
        reason:
          "a shell filter selects part of fdu output instead of recording the complete product response",
      });
      return;
    }

    if (WRAPPER_INVOCATION.test(command) && OUTPUT_PARSER.test(command)) {
      findings.push({
        file,
        line: index + 1,
        reason:
          "a wrapper parses fdu output instead of recording the complete product response",
      });
      return;
    }

    if (helperRunsFdu && OUTPUT_PARSER.test(command)) {
      findings.push({
        file,
        line: index + 1,
        reason: "a helper parses fdu output instead of forwarding the complete product response",
      });
    }
  });

  return findings;
}

export function formatFindings(findings) {
  return [
    "golden observability check failed:",
    ...findings.map(({ file, line, reason }) => `- ${file}:${line}: ${reason}`),
    "  invoke fdu directly and record its complete stable output; put relational assertions in a focused test",
  ].join("\n");
}

function collectGoldenFiles() {
  const sessions = readdirSync(GOLDEN_DIRECTORY)
    .filter((name) => name.endsWith(".tryscript.md"))
    .map((name) => join(GOLDEN_DIRECTORY, name));
  const helpers = readdirSync(join(GOLDEN_DIRECTORY, "bin"))
    .filter((name) => name.endsWith(".mjs"))
    .map((name) => join(GOLDEN_DIRECTORY, "bin", name));
  return [...sessions, ...helpers];
}

export function auditGoldenCorpus() {
  return collectGoldenFiles().flatMap((path) => {
    const file = relative(ROOT, path).split(sep).join("/");
    return auditGoldenText(readFileSync(path, "utf8"), file);
  });
}

function main() {
  const findings = auditGoldenCorpus();
  if (findings.length > 0) {
    console.error(formatFindings(findings));
    process.exitCode = 1;
    return;
  }
  console.log("golden observability ok: product sessions record complete stable responses");
}

if (process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url) {
  main();
}
