/** `cli-animate doctor`: report the external programs and assets the pipeline needs. */

import { existsSync } from 'node:fs';
import type { Command } from 'commander';
import { chromium } from 'playwright-core';
import { EXIT_CHECK_FAILED } from '../../errors.js';
import { missingFaces } from '../../fonts.js';
import { programOutput, programVersion, whichProgram } from '../../tools.js';
import { context } from '../lib/context.js';
import { ICONS } from '../lib/output.js';

interface Check {
  ok: boolean;
  detail: string;
}

function checks(): Record<string, Check> {
  const asciinema = whichProgram('asciinema');
  const asciinemaVersion = asciinema ? programVersion(asciinema, ['--version']) : '';
  const ffmpeg = whichProgram('ffmpeg');
  const chromePath = process.env.CHROME_PATH ?? chromium.executablePath();
  const fontsMissing = missingFaces().length > 0;
  return {
    node: { ok: Number(process.versions.node.split('.')[0]) >= 22, detail: process.version },
    asciinema: { ok: /\b3\.\d+/.test(asciinemaVersion), detail: asciinema ? asciinemaVersion : 'not on PATH (needs 3.x)' },
    ffmpeg: {
      ok: ffmpeg !== undefined && /libx264/.test(programOutput(ffmpeg, ['-hide_banner', '-encoders'])),
      detail: ffmpeg ? programVersion(ffmpeg, ['-version']) : 'not on PATH (needs libx264)',
    },
    chromium: { ok: existsSync(chromePath), detail: chromePath },
    fonts: { ok: !fontsMissing, detail: fontsMissing ? 'run `cli-animate fonts`' : 'Planetaire Mono Text present' },
  };
}

export function registerDoctor(program: Command): void {
  program
    .command('doctor')
    .description('report the external programs and assets the pipeline needs')
    .action((_opts: object, command: Command) => {
      const { out } = context(command);
      const found = checks();
      const ok = Object.values(found).every((c) => c.ok);
      const lines = Object.entries(found).map(
        ([name, c]) => `  ${c.ok ? out.colors.success(ICONS.SUCCESS) : out.colors.error(ICONS.ERROR)} ${name.padEnd(10)} ${c.detail}`,
      );
      out.result({ ok, checks: found }, lines.join('\n'));
      if (!ok) process.exitCode = EXIT_CHECK_FAILED;
    });
}
