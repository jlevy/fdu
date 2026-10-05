/** `cli-animate deliver`: derive another delivery from a kept master, without capturing again. */

import { type Command, Option } from 'commander';
import { deliver, masterInfo, writeRenderReceipt } from '../../render.js';
import { context, positive } from '../lib/context.js';

export function registerDeliver(program: Command): void {
  program
    .command('deliver')
    .description('derive another delivery (web MP4, README GIF) from a kept master')
    .argument('<master>', 'a lossless master from `render --master` or `make`')
    .requiredOption('-o, --output <video>', 'video to write')
    .addOption(new Option('--profile <name>', 'delivery profile').choices(['web', 'gif']).default('web'))
    .option('--scale <n>', 'the scale the master was captured at', positive, 2)
    .option('--cast <cast>', 'the cast the master came from, for the receipt')
    .action(async (master: string, opts: { output: string; profile: 'web' | 'gif'; scale: number; cast?: string }, command: Command) => {
      const { out } = context(command);
      const info = masterInfo(master, opts.scale);
      const args = await deliver(info, opts.output, opts.profile);
      const receipt = writeRenderReceipt(opts.cast ?? '', opts.output, opts.profile, info, args, true);
      out.result(receipt, `${out.colors.success('delivered')} ${out.colors.path(opts.output)} from ${out.colors.path(master)}`);
    });
}
