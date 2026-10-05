/**
 * The command line's contract, through the built binary: help groups, exit codes, JSON
 * errors on stderr, and a reader closing stdout early.
 */

import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it } from 'node:test';
import { CLI_MAIN, PACKAGE_ROOT, SKILL_FILE } from '../src/paths.js';

// An empty PATH: these contracts must hold on a host without asciinema, ffmpeg, or Chromium.
const run = (...args: string[]) =>
  spawnSync(process.execPath, [CLI_MAIN, ...args], { encoding: 'utf8', env: { ...process.env, NO_COLOR: '1', PATH: '' } });

describe('command line', () => {
  it('groups the commands in help and hides the driver', () => {
    const { status, stdout } = run('--help');
    assert.equal(status, 0);
    for (const heading of ['Pipeline:', 'Viewing:', 'Setup:']) assert.ok(stdout.includes(heading), heading);
    assert.ok(!stdout.includes('__drive'));
  });

  it('reports the package version', () => {
    const { version } = JSON.parse(readFileSync(join(PACKAGE_ROOT, 'package.json'), 'utf8')) as { version: string };
    assert.equal(run('--version').stdout.trim(), version);
  });

  it('exits 2 on a usage error', () => {
    assert.equal(run('bogus').status, 2);
    const bad = run('render', '--fps', 'x', 'a.cast');
    assert.equal(bad.status, 2);
    assert.match(bad.stderr, /expected a positive integer/);
  });

  it('puts errors on stderr, as JSON with --json, and keeps stdout empty, with no external programs', () => {
    const { status, stdout, stderr } = run('--json', 'verify', 'missing.cast', 'missing.mp4');
    assert.equal(status, 2);
    assert.equal(stdout, '');
    assert.match((JSON.parse(stderr) as { error: string }).error, /missing\.cast/);
  });

  it('prints the skill', () => {
    assert.equal(run('skill').stdout, readFileSync(SKILL_FILE, 'utf8'));
  });

  it('exits 0 when the reader closes stdout early', async () => {
    const child = spawn(process.execPath, [CLI_MAIN, 'skill'], { stdio: ['ignore', 'pipe', 'pipe'] });
    child.stdout.destroy();
    let stderr = '';
    child.stderr.on('data', (chunk: Buffer) => (stderr += chunk.toString()));
    const code = await new Promise<number | null>((done) => child.on('close', done));
    assert.equal(code, 0);
    assert.equal(stderr, '');
  });
});
