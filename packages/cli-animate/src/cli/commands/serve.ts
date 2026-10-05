/** `cli-animate serve`: serve the replay page for a cast on localhost. */

import { existsSync } from 'node:fs';
import type { Command } from 'commander';
import { CliError } from '../../errors.js';
import { missingFaces } from '../../fonts.js';
import { startStageServer } from '../../server.js';
import { context, nonNegativeInteger } from '../lib/context.js';

export function registerServe(program: Command): void {
  program
    .command('serve')
    .description('serve the replay page for a cast on localhost')
    .argument('<cast>', 'asciicast file')
    .option('--port <n>', 'port (0 picks a free one)', nonNegativeInteger, 8040)
    .action(async (cast: string, opts: { port: number }, command: Command) => {
      const { out } = context(command);
      if (!existsSync(cast)) throw new CliError(`cast not found: ${cast}`);
      if (missingFaces().length > 0) throw new CliError('fonts are missing: run `cli-animate fonts` first');
      const server = await startStageServer(cast, opts.port);
      const url = `${server.url}/stage.html`;
      out.result({ url }, `serving ${out.colors.path(url)} (Ctrl-C to stop)`);
      await new Promise<void>((resolve) => process.once('SIGINT', () => resolve()));
      await server.close();
    });
}
