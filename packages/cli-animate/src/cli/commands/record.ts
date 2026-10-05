/** `cli-animate record`: run a scenario for real and write its cast and receipt. */

import { join } from 'node:path';
import type { Command } from 'commander';
import { record } from '../../record.js';
import { context, stem } from '../lib/context.js';

export function registerRecord(program: Command): void {
  program
    .command('record')
    .description('run a scenario for real and write its cast and receipt')
    .argument('<scenario>', 'scenario YAML file')
    .option('-o, --output <cast>', 'cast to write (default: out/<scenario>.cast)')
    .action((scenario: string, opts: { output?: string }, command: Command) => {
      const { options, out } = context(command);
      const cast = opts.output ?? join('out', `${stem(scenario)}.cast`);
      out.note(`recording ${scenario} (commands run for real; this takes as long as the take)`);
      const receipt = record(scenario, cast, { quiet: options.quiet || options.json });
      const lines = receipt.steps.map(
        (s) => `  ${s.label.padEnd(24)} ${s.wall_seconds.toFixed(3)} s  exit ${String(s.exit_code ?? s.signal)}`,
      );
      out.result(
        receipt,
        `${out.colors.success('recorded')} ${out.colors.path(cast)} (${receipt.duration_seconds.toFixed(1)} s)\n${lines.join('\n')}`,
      );
    });
}
