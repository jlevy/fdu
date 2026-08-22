#!/usr/bin/env node
// Refuse committed test data that only reproduces on the machine that wrote it.
//
// Two failures in this repository came from exactly that, and neither was catchable by
// the local gate:
//
//   * `tryscript run --update` rewrites expectations with the ACTUAL output, which
//     expands named patterns into literals. Regenerating a golden after an intentional
//     change silently replaced [OS_ERROR] and [SCAN_PATH] with a real errno and a real
//     sandbox path, so the golden then passed only on the machine that recorded it.
//   * The parity artifact carried an absolute checkout path until it was masked.
//
// Both look correct in review -- a plausible path is harder to notice than a wrong one --
// and both pass locally by construction, because the recording machine is the verifying
// machine. This check is the part that does not depend on where it runs.

import { readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));

// A path rooted in somebody's filesystem, a sandbox directory, or a Windows drive.
const MACHINE_SPECIFIC = [
  { pattern: /\/(?:Users|home)\/[A-Za-z0-9._-]+\//, why: "a home directory from the recording machine" },
  { pattern: /\/(?:private\/)?(?:var|tmp)\/[A-Za-z0-9._\/-]*(?:tryscript|fdu)-[A-Za-z0-9]{6,}/, why: 'a sandbox directory that will not exist again' },
  { pattern: /\b[A-Z]:\\(?:Users|a)\\/, why: 'a Windows path from the recording machine' },
];

const targets = [];
const goldenDir = join(root, 'tests', 'golden');
for (const name of readdirSync(goldenDir).filter((f) => f.endsWith('.tryscript.md'))) {
  targets.push([join('tests', 'golden', name), join(goldenDir, name)]);
}
const artifact = join(root, 'tests', 'parity', 'deviations-python.diff');
try {
  if (statSync(artifact).isFile()) {
    targets.push([join('tests', 'parity', 'deviations-python.diff'), artifact]);
  }
} catch {
  // Not recorded yet; the parity target will say so when it runs.
}

const findings = [];
for (const [name, file] of targets) {
  readFileSync(file, 'utf8')
    .split('\n')
    .forEach((line, index) => {
      for (const { pattern, why } of MACHINE_SPECIFIC) {
        if (pattern.test(line)) {
          findings.push(`${name}:${index + 1}: ${why}\n      ${line.trim().slice(0, 120)}`);
        }
      }
    });
}

if (findings.length > 0) {
  console.error('committed test data must not name the machine that recorded it:\n');
  for (const finding of findings) {
    console.error(`  ${finding}`);
  }
  console.error('\nMask it with a named pattern. If `tryscript run --update` expanded one,');
  console.error('put the pattern back by hand: --update writes what it saw, not what varies.');
  process.exit(1);
}

console.log(`portability ok: no machine-specific paths in ${targets.length} files`);
