#!/usr/bin/env node
// Parse fdu's YAML output with a real YAML parser.
//
// A byte-stable golden proves the output has not *changed*; it never proves the output is
// *valid*. A consistently malformed document passes forever, and this serializer is
// hand-written -- the project avoids serde -- so nothing else would notice. Until this
// script, no YAML parser had ever read fdu's YAML.
//
// The parser is `yaml`, already in the locked dev tree and integrity-pinned, so this
// costs no new dependency.

import { execFileSync } from 'node:child_process';
import { mkdtempSync, writeFileSync, mkdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
import { parse } from 'yaml';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const fdu = join(root, 'target', 'debug', 'fdu');

// A tree with enough shape that every view has rows to render.
const tree = mkdtempSync(join(tmpdir(), 'fdu-yaml-'));
mkdirSync(join(tree, 'src'));
mkdirSync(join(tree, 'docs'));
writeFileSync(join(tree, 'src', 'main.rs'), 'fn main() {\n    // hi\n}\n');
writeFileSync(join(tree, 'src', 'app.py'), '# c\nprint(1)\n');
writeFileSync(join(tree, 'docs', 'guide.md'), '# T\n\nsome prose words here\n');
writeFileSync(join(tree, 'data.json'), '{"k": 1}\n');

const views = [
  'tree', 'types', 'extensions', 'families', 'languages',
  'files', 'largest', 'recent', 'summary',
];

let checked = 0;
for (const view of views) {
  const args = ['--cache', 'off', '--format', 'yaml', '--view', view];
  // `documents` needs content, and analysing it also exercises the analysis block.
  if (view === 'documents') args.push('--analyze', 'words');
  const out = execFileSync(fdu, [...args, tree], { encoding: 'utf8' });

  const parsed = parse(out);
  assert.ok(parsed, `${view}: parsed to nothing`);
  assert.ok(typeof parsed.schema === 'string' && parsed.schema.startsWith('fdu.report/'),
    `${view}: no schema in parsed YAML`);
  assert.equal(parsed.complete, true, `${view}: complete did not survive the round trip`);
  assert.ok(Array.isArray(parsed.reports) && parsed.reports.length === 1,
    `${view}: expected exactly one report section`);
  assert.equal(parsed.reports[0].view, view, `${view}: section names a different view`);
  checked += 1;
}

// A bounded view must carry its bound through the round trip, since the whole point of
// the field is that a consumer can read it.
const bounded = parse(execFileSync(
  fdu, ['--cache', 'off', '--format', 'yaml', '--view', 'largest', '-n', '1', tree],
  { encoding: 'utf8' },
));
assert.equal(bounded.reports[0].bound.shown, 1, 'bound.shown lost in YAML');
assert.ok(bounded.reports[0].bound.total > 1, 'bound.total lost in YAML');

// And an unbounded one says null rather than omitting the key, so a consumer can branch
// on the value instead of on presence.
const unbounded = parse(execFileSync(
  fdu, ['--cache', 'off', '--format', 'yaml', '--view', 'files', tree],
  { encoding: 'utf8' },
));
assert.ok('bound' in unbounded.reports[0], 'bound key missing when nothing was dropped');
assert.equal(unbounded.reports[0].bound, null, 'an unbounded view must say null');

console.log(`yaml self-check passed: ${checked} views parsed, bound round-trips both ways`);
