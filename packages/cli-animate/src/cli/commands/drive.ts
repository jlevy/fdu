/** `cli-animate __drive`: the in-PTY half of `record`; hidden, not for direct use. */

import type { Command } from 'commander';
import { drive } from '../../driver.js';

export function registerDrive(program: Command): void {
  program
    .command('__drive', { hidden: true })
    .argument('<scenario>')
    .requiredOption('--sidecar <path>')
    .action(async (scenario: string, opts: { sidecar: string }) => {
      await drive(scenario, opts.sidecar);
    });
}
