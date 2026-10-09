/** `cli-animate make`: fonts, record, capture, verify, and deliver in one step. */

import { join } from 'node:path';
import type { Command } from 'commander';
import { EXIT_CHECK_FAILED } from '../../errors.js';
import { fetchFonts } from '../../fonts.js';
import { PROFILES } from '../../profiles.js';
import { record } from '../../record.js';
import { loadScenario } from '../../scenario.js';
import { capture, deliver, writeRenderReceipt } from '../../render.js';
import { verify } from '../../verify.js';
import { context, integer, nonNegativeInteger, positive, stem } from '../lib/context.js';

interface MakeOptions {
  out: string;
  gif?: boolean;
  fps: number;
  scale: number;
  margin: number;
  chrome?: string;
}

export function registerMake(program: Command): void {
  program
    .command('make')
    .description('fonts, record, capture, verify, and deliver in one step')
    .argument('<scenario>', 'scenario YAML file')
    .option('--out <dir>', 'output directory', 'out')
    .option('--gif', 'also deliver a README-sized GIF')
    .option('--fps <n>', 'frames per second', integer, 60)
    .option('--scale <n>', 'device pixels per CSS pixel', positive, 2)
    .option('--margin <px>', 'background margin around the window', nonNegativeInteger, 32)
    .option('--chrome <path>', 'Chromium executable (default: CHROME_PATH, then playwright-core’s)')
    .action(async (scenario: string, opts: MakeOptions, command: Command) => {
      const { options, out } = context(command);
      const name = stem(scenario);
      const cast = join(opts.out, `${name}.cast`);
      const master = join(opts.out, `${name}.master.mp4`);
      await fetchFonts();
      out.note(`recording ${scenario} (commands run for real)`);
      const recorded = record(scenario, cast, { quiet: options.quiet || options.json });
      out.note(`capturing ${master}`);
      const captured = await capture(cast, master, {
        fps: opts.fps,
        fontSize: loadScenario(scenario).terminal.font_size,
        scale: opts.scale,
        margin: opts.margin,
        ...(opts.chrome === undefined ? {} : { chromePath: opts.chrome }),
        onProgress: (done, total) => out.note(`  frame ${done} / ${total}`),
      });
      const verified = verify(cast, master, opts.fps);
      if (!verified.ok) {
        out.result(
          { record: recorded, capture: captured, verify: verified, deliveries: [] },
          `${out.colors.error('timing mismatch')} in ${out.colors.path(master)}; nothing delivered\n` +
            `  missing ${JSON.stringify(verified.missing)} unexpected ${JSON.stringify(verified.unexpected)}`,
        );
        process.exitCode = EXIT_CHECK_FAILED;
        return;
      }
      const deliveries: string[] = [];
      for (const profileName of opts.gif ? (['web', 'gif'] as const) : (['web'] as const)) {
        const video = join(opts.out, `${name}${PROFILES[profileName].extension}`);
        out.note(`delivering ${video}`);
        writeRenderReceipt(cast, video, profileName, captured, await deliver(captured, video, profileName), true);
        deliveries.push(video);
      }
      out.result(
        { record: recorded, capture: captured, verify: verified, deliveries },
        `${out.colors.success('made')} ${deliveries.map((d) => out.colors.path(d)).join(', ')}\n` +
          recorded.steps.map((s) => `  ${s.label.padEnd(24)} ${s.wall_seconds.toFixed(3)} s`).join('\n'),
      );
    });
}
