/**
 * End to end, through the real external programs: record a tiny scenario with asciinema,
 * capture a lossless master in Chromium, require a frame-exact verify, and check the web
 * delivery's colour tags and level. Opt-in, since it needs asciinema 3, ffmpeg, a
 * Chromium, and the fonts: `npm run e2e` sets CLI_ANIMATE_E2E=1.
 */

import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, it } from 'node:test';
import { parseCast } from '../src/cast.js';
import { record } from '../src/record.js';
import { render } from '../src/render.js';
import { verify } from '../src/verify.js';

const enabled = process.env.CLI_ANIMATE_E2E === '1';

describe('record, render, verify', { skip: !enabled && 'set CLI_ANIMATE_E2E=1 to run' }, () => {
  it('produces a cast whose lossless render changes exactly on its events', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'cli-animate-e2e-'));
    const scenario = join(dir, 'tiny.yaml');
    writeFileSync(
      scenario,
      [
        'title: e2e',
        'terminal: { cols: 60, rows: 8 }',
        'typing: { wpm: 150, seed: 5 }',
        'lead_in: 0.2',
        'tail: 0.5',
        'steps:',
        '  - label: hello',
        "    run: printf 'hello\\n'; sleep 0.3; printf 'world\\n'",
        '    hold: 0.5',
      ].join('\n'),
    );
    const cast = join(dir, 'tiny.cast');
    const receipt = record(scenario, cast, { quiet: true });
    assert.equal(receipt.steps.length, 1);
    assert.equal(receipt.steps[0]!.exit_code, 0);
    assert.ok(receipt.steps[0]!.wall_seconds >= 0.3, 'the command’s own sleep is in its wall time');

    const events = parseCast(readFileSync(cast, 'utf8')).events;
    assert.ok(events.some((e) => e.code === 'm' && e.data === 'hello'));
    const world = events.findIndex((e) => e.code === 'o' && e.data.includes('world'));
    assert.ok(world >= 0);
    // The next prompt follows the command's exit at once; the hold comes after it.
    const prompt = events.findIndex((e, i) => i > world && e.code === 'o' && e.data.includes('❯'));
    const end = events.findIndex((e) => e.code === 'm' && e.data === 'end');
    assert.ok(prompt > world && prompt < end, 'a prompt is printed before the end marker');
    assert.ok(events[prompt]!.time - events[world]!.time < 0.1, 'the prompt appears when the command exits');

    const master = join(dir, 'tiny.master.mp4');
    const video = join(dir, 'tiny.mp4');
    const receipt2 = await render(cast, video, { profile: 'web', fps: 30, holdEnd: 0.2, masterPath: master });
    const result = verify(cast, master, 30);
    assert.ok(result.ok, JSON.stringify(result));
    assert.equal(receipt2.stream.pix_fmt, 'yuv420p');
    assert.equal(receipt2.stream.color_space, 'bt709');
    assert.ok(receipt2.stream.level !== undefined && Number(receipt2.stream.level) <= 5.2);
  });
});
