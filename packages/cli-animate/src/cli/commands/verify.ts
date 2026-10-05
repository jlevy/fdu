/** `cli-animate verify`: check a lossless master frame-exactly against its cast. */

import type { Command } from 'commander';
import { EXIT_CHECK_FAILED } from '../../errors.js';
import { verify } from '../../verify.js';
import { context, integer } from '../lib/context.js';

export function registerVerify(program: Command): void {
  program
    .command('verify')
    .description('check a lossless master frame-exactly against its cast')
    .argument('<cast>', 'the cast the video was rendered from')
    .argument('<master>', 'the lossless master (`render --master`, or `make`’s *.master.mp4)')
    .option('--fps <n>', 'the render’s frames per second', integer, 60)
    .action((cast: string, master: string, opts: { fps: number }, command: Command) => {
      const { out } = context(command);
      const result = verify(cast, master, opts.fps);
      out.result(
        result,
        result.ok
          ? `${out.colors.success('verified')} ${result.expected_change_frames} change frames, each on its event’s frame`
          : `${out.colors.error('mismatch')} missing ${JSON.stringify(result.missing)} unexpected ${JSON.stringify(result.unexpected)}`,
      );
      if (!result.ok) process.exitCode = EXIT_CHECK_FAILED;
    });
}
