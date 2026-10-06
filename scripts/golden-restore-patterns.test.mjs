import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

import { lineMatcher, main, restorePatterns } from './golden-restore-patterns.mjs';

const script = fileURLToPath(new URL('./golden-restore-patterns.mjs', import.meta.url));
const repoRoot = dirname(dirname(script));

const committed = `---
patterns:
  PERF_TIME: '[\\d.]+ (ns|µs|ms|s)'
---
\`\`\`console
$ fdu x
! note: old wording
! perf: took [PERF_TIME] to walk 2 files at [..]
? 0
\`\`\`
`;

test('an expanded pattern line is restored and a reworded line is kept', () => {
  const updated = committed
    .replace('! note: old wording', '! note: new wording')
    .replace('took [PERF_TIME] to walk 2 files at [..]', 'took 1.2 ms to walk 2 files at 9 files/s');
  const { text, restored } = restorePatterns(committed, updated);
  assert.equal(restored, 1);
  assert.match(text, /! note: new wording/);
  assert.match(text, /took \[PERF_TIME\] to walk 2 files at \[\.\.\]/);
});

test('a line whose literal text changed is not restored', () => {
  const updated = committed.replace(
    'took [PERF_TIME] to walk 2 files at [..]',
    'took 1.2 ms to walk 3 files at 9 files/s',
  );
  const { text, restored } = restorePatterns(committed, updated);
  assert.equal(restored, 0);
  assert.match(text, /walk 3 files/);
});

test('front-matter patterns are honored, and elision lines are never paired', () => {
  const matcher = lineMatcher('took [PERF_TIME] total', { PERF_TIME: '[\\d.]+ (ns|µs|ms|s)' });
  assert.ok(matcher.test('took 1.5 ms total'));
  assert.ok(!matcher.test('took soon total'));
  assert.equal(lineMatcher('...', {}), null);
  assert.equal(lineMatcher('no patterns here', {}), null);
});

// Each of these would otherwise restore nothing and exit 0, which reads as success.
for (const [name, args, message] of [
  ['an unknown base', ['--base', 'no-such-revision'], /not a revision: no-such-revision/],
  ['a bare --base', ['--base'], /not a revision: \(none\)/],
  ['a missing file', ['tests/golden/no-such.tryscript.md'], /no such file: tests\/golden\/no-such/],
]) {
  test(`the script exits 2 for ${name}`, () => {
    const run = spawnSync(process.execPath, [script, ...args], { cwd: repoRoot, encoding: 'utf8' });
    assert.equal(run.status, 2, run.stderr);
    assert.match(run.stderr, message);
    assert.doesNotMatch(run.stdout, /lines restored/);
  });
}

test('only a file the base lacks is skipped, and a changed golden is restored', (t) => {
  const repo = mkdtempSync(join(tmpdir(), 'golden-restore-'));
  const outside = mkdtempSync(join(tmpdir(), 'golden-restore-outside-'));
  t.after(() => {
    rmSync(repo, { recursive: true, force: true });
    rmSync(outside, { recursive: true, force: true });
  });
  const git = (...args) =>
    execFileSync(
      'git',
      ['-c', 'user.name=test', '-c', 'user.email=test@example.com', '-c', 'commit.gpgsign=false', ...args],
      { cwd: repo, stdio: 'pipe' },
    );
  const golden = join(repo, 'tests', 'golden', 'kept.tryscript.md');
  mkdirSync(dirname(golden), { recursive: true });
  writeFileSync(golden, committed);
  git('init', '--quiet');
  git('add', '.');
  git('commit', '--quiet', '--no-verify', '-m', 'base');
  const logged = t.mock.method(console, 'log', () => {});
  const errors = t.mock.method(console, 'error', () => {});

  const added = join(repo, 'tests', 'golden', 'added.tryscript.md');
  const expanded = committed.replace('[PERF_TIME]', '1.2 ms');
  writeFileSync(added, expanded);
  assert.equal(main([added], repo), 0, 'a file new in this change is not an error');
  assert.equal(readFileSync(added, 'utf8'), expanded, 'and it is left as written');

  writeFileSync(golden, expanded);
  assert.equal(main([golden], repo), 0);
  assert.equal(readFileSync(golden, 'utf8'), committed, 'the expanded pattern is restored');
  assert.match(logged.mock.calls.at(-1).arguments[0], /1 lines restored against HEAD/);

  const stray = join(outside, 'stray.tryscript.md');
  writeFileSync(stray, committed);
  assert.equal(main([stray], repo), 2, 'a file outside the checkout is refused');
  assert.match(errors.mock.calls.at(-1).arguments[0], /not in this checkout/);
});
