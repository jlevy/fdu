#!/usr/bin/env node
// Keep the golden corpus from resolving fdu anywhere but the build under test.
//
// tryscript's `path:` front matter PREPENDS to the inherited PATH rather than replacing
// it. Every session invokes a bare `fdu`, because a bare command name is the only form
// /bin/sh and cmd.exe both read the same way -- Windows does not expand `$FDU`, it wants
// `%FDU%`. So the directory that supplies the command is the whole of the safety story:
// if it fails to resolve, lookup continues into the inherited PATH and finds whatever
// fdu is installed on the machine, and the suite passes while testing a different
// binary, silently (fdu-9h2w).
//
// Two rules follow, and this check exists because both are worth nothing if the next
// session can quietly drop them and still go green.

import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const goldenDir = join(root, 'tests', 'golden');

const findings = [];

// Rule 1: every session file declares exactly one path entry, and it is the variable
// run-golden.mjs sets after preflighting the binary. A literal like
// `$TRYSCRIPT_GIT_ROOT/target/debug` is what the corpus used to say, and it silently
// pinned the corpus to one surface while looking equally correct.
for (const name of readdirSync(goldenDir).filter((f) => f.endsWith('.tryscript.md'))) {
  const file = join('tests', 'golden', name);
  const lines = readFileSync(join(goldenDir, name), 'utf8').split('\n');
  const start = lines.indexOf('path:');
  if (start === -1) {
    findings.push(`${file}: no \`path:\` entry, so fdu resolves from the inherited PATH`);
    continue;
  }
  const entries = [];
  for (let i = start + 1; i < lines.length && lines[i].startsWith('  - '); i += 1) {
    entries.push(lines[i].slice(4).trim());
  }
  if (entries.length !== 1 || entries[0] !== '$FDU_BIN') {
    findings.push(
      `${file}:${start + 1}: path must be exactly [$FDU_BIN], found [${entries.join(', ')}]`,
    );
  }
}

// Rule 2: the helper scripts sessions delegate to spawn the binary themselves, so they
// are subject to no PATH lookup and must name the exact file. Checking only the session
// files is how watch-repaint-capture.mjs kept a bare `fdu` after the corpus had stopped
// relying on one: it passed locally by resolving the installed build, and failed in CI.
const binDir = join(goldenDir, 'bin');
for (const name of readdirSync(binDir).filter((f) => f.endsWith('.mjs'))) {
  const file = join('tests', 'golden', 'bin', name);
  readFileSync(join(binDir, name), 'utf8')
    .split('\n')
    .forEach((line, index) => {
      // The literal need not sit inside the spawn call to be a PATH lookup: the defect
      // wrote `const binary = platform === 'win32' ? 'fdu.exe' : 'fdu'` several lines
      // above `spawn(binary, ...)`, so matching only the call site missed it entirely.
      if (/(?<![\w/\\.$-])['"]fdu(\.exe)?['"]/.test(line)) {
        findings.push(`${file}:${index + 1}: names \`fdu\` literally; use \`process.env.FDU\``);
      }
    });
}

if (findings.length > 0) {
  console.error('golden corpus must resolve fdu only from the build under test:\n');
  for (const finding of findings) {
    console.error(`  ${finding}`);
  }
  console.error('\nsee scripts/run-golden.mjs for why, and fdu-9h2w for what it broke');
  process.exit(1);
}

console.log('golden invocations ok: every session pins $FDU_BIN, no helper names fdu');
