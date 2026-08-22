#!/usr/bin/env node
// Forbid bare `fdu` in the golden corpus.
//
// tryscript's `path:` front matter prepends to the inherited PATH rather than replacing
// it, so a bare `fdu` is resolvable from outside the repository -- typically
// ~/.cargo/bin/fdu on a developer machine. Any golden that invokes the bare name can
// therefore pass while exercising a build nobody selected (fdu-9h2w), and the whole
// premise of parity testing is that the surfaces are different binaries.
//
// The corpus names the executable through $FDU instead. This check keeps it that way:
// the rule is worth nothing if the next session can reintroduce the bare name and still
// go green.

import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const goldenDir = join(root, 'tests', 'golden');

// A shell command line opening with the bare name, and the node child-process helpers
// spelling it as a literal argument. Both resolve through PATH.
const OFFENCES = [
  { pattern: /^\$\s+fdu(\.exe)?\b/, why: 'session invokes bare `fdu`; use `$FDU`' },
  {
    pattern: /(spawnSync|execFileSync|spawn|execFile)\(\s*['"]fdu(\.exe)?['"]/,
    why: 'node helper names `fdu` literally; use `process.env.FDU`',
  },
  {
    pattern: /^path:/,
    why: 'a `path:` entry reintroduces PATH lookup; name the binary through $FDU',
  },
];

const findings = [];
for (const name of readdirSync(goldenDir).filter((f) => f.endsWith('.tryscript.md'))) {
  const file = join(goldenDir, name);
  readFileSync(file, 'utf8')
    .split('\n')
    .forEach((line, index) => {
      for (const { pattern, why } of OFFENCES) {
        if (pattern.test(line)) {
          findings.push(`${join('tests', 'golden', name)}:${index + 1}: ${why}`);
        }
      }
    });
}

if (findings.length > 0) {
  console.error('golden corpus must never resolve fdu through PATH:\n');
  for (const finding of findings) {
    console.error(`  ${finding}`);
  }
  console.error('\nsee scripts/run-golden.mjs for why, and fdu-9h2w for what it broke');
  process.exit(1);
}

console.log(`golden invocations ok: no PATH-resolved fdu in ${goldenDir}`);
