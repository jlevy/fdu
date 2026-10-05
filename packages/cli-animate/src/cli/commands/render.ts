/** `cli-animate render`: capture a cast through the stage page and deliver one profile. */

import { type Command, Option } from 'commander';
import { PROFILE_NAMES, PROFILES, type ProfileName } from '../../profiles.js';
import { render } from '../../render.js';
import { context, integer, nonNegativeInteger, positive } from '../lib/context.js';

interface RenderOptions {
  output?: string;
  profile: ProfileName;
  fps: number;
  scale: number;
  master?: string;
  margin: number;
  fontSize: number;
  chrome?: string;
}

export function registerRender(program: Command): void {
  program
    .command('render')
    .description('capture a cast through the stage page and deliver one profile')
    .argument('<cast>', 'asciicast v2 or v3 file')
    .option('-o, --output <video>', 'video to write (default: the cast name with the profile extension)')
    .addOption(new Option('--profile <name>', 'encoding profile').choices([...PROFILE_NAMES]).default('web'))
    .option('--fps <n>', 'frames per second', integer, 60)
    .option('--scale <n>', 'device pixels per CSS pixel', positive, 2)
    .option('--master <path>', 'also keep the lossless master here (for verify)')
    .option('--margin <px>', 'background margin around the window', nonNegativeInteger, 32)
    .option('--font-size <px>', 'terminal font size', integer, 22)
    .option('--chrome <path>', 'Chromium executable (default: CHROME_PATH, then playwright-core’s)')
    .action(async (cast: string, opts: RenderOptions, command: Command) => {
      const { out } = context(command);
      const chosen = PROFILES[opts.profile];
      const video = opts.output ?? cast.replace(/\.cast$/, '') + chosen.extension;
      out.note(`rendering ${cast} with the ${chosen.name} profile`);
      const receipt = await render(cast, video, {
        profile: opts.profile,
        fps: opts.fps,
        margin: opts.margin,
        fontSize: opts.fontSize,
        scale: opts.scale,
        ...(opts.master === undefined ? {} : { masterPath: opts.master }),
        ...(opts.chrome === undefined ? {} : { chromePath: opts.chrome }),
        onProgress: (done, total) => out.note(`  frame ${done} / ${total}`),
      });
      const c = receipt.capture;
      const level = receipt.stream.level ? `, level ${receipt.stream.level}` : '';
      out.result(
        receipt,
        `${out.colors.success('rendered')} ${out.colors.path(video)} ${c.width}x${c.height}, ${(c.frames / c.fps).toFixed(1)} s, ` +
          `${c.screenshots} screenshots for ${c.frames} frames${level}`,
      );
    });
}
