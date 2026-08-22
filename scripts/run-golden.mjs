#!/usr/bin/env node
// Run the golden corpus against one named surface, and prove which one ran.
//
// The corpus used to invoke a bare `fdu` and rely on `path:` front matter to put
// target/debug ahead of everything else. That is a prepend, not a replacement: if the
// entry ever failed to resolve -- variable unset, target/ cleaned, a cross-compiled
// layout -- lookup continued into the inherited PATH and found whatever fdu happened to
// be installed there, typically ~/.cargo/bin/fdu. The suite then passed while testing a
// different binary, silently (fdu-9h2w).
//
// So the corpus now invokes `$FDU`, which names the executable outright. There is no
// PATH lookup left to fall through, an unset variable is an immediate `command not
// found` rather than a wrong-binary pass, and pointing the same sessions at a second
// implementation is a matter of naming a different file.

import { spawnSync } from 'node:child_process';
import { accessSync, constants, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const exe = process.platform === 'win32' ? '.exe' : '';

// Each surface is a full path, never a name to be searched for.
const SURFACES = {
  rust: join(root, 'target', 'debug', `fdu${exe}`),
  python: join(root, 'tests', 'parity', 'py', 'fdu'),
};

const surface = process.env.FDU_SURFACE ?? 'rust';
const binary = SURFACES[surface];
if (!binary) {
  const known = Object.keys(SURFACES).join(', ');
  console.error(`run-golden: unknown FDU_SURFACE "${surface}" (known: ${known})`);
  process.exit(2);
}

// Preflight. A missing or non-executable surface has to be a diagnosed failure before
// the first session, not a hundred confusing diffs after it.
try {
  if (!statSync(binary).isFile()) {
    throw new Error('not a regular file');
  }
  if (process.platform !== 'win32') {
    accessSync(binary, constants.X_OK);
  }
} catch (error) {
  console.error(`run-golden: the ${surface} surface is not runnable at ${binary}`);
  console.error(`run-golden: ${error.message}`);
  console.error(
    surface === 'rust'
      ? 'run-golden: build it with `make build`'
      : 'run-golden: the parity shim ships in the repository; check it is executable',
  );
  process.exit(2);
}

// State the instrument, the way every fdu report states its own source and freshness.
console.log(`run-golden: ${surface} surface -> ${binary}`);

// Resolved from the locked tree rather than from PATH, for the same reason the surface
// is: the harness should not be able to run a tryscript nobody pinned.
const tryscript = join(root, 'node_modules', '.bin', `tryscript${process.platform === 'win32' ? '.cmd' : ''}`);
// FDU_CORPUS lets the parity run replay a filtered copy of the same sessions; it
// defaults to the corpus itself, which is what every ordinary run wants.
const corpus = process.env.FDU_CORPUS ?? join('tests', 'golden');
const args = ['run', ...process.argv.slice(2), join(corpus, '*.tryscript.md')];
const result = spawnSync(tryscript, args, {
  cwd: root,
  stdio: 'inherit',
  shell: process.platform === 'win32',
  env: { ...process.env, FDU: binary, FDU_SURFACE: surface },
});

process.exit(result.status ?? 1);
