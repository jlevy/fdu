/** `cli-animate fonts`: fetch the pinned terminal font and verify it by hash. */

import type { Command } from 'commander';
import { FACES, fetchFonts } from '../../fonts.js';
import { context } from '../lib/context.js';

export function registerFonts(program: Command): void {
  program
    .command('fonts')
    .description('fetch the pinned terminal font and verify it by SHA-256')
    .action(async (_opts: object, command: Command) => {
      const { out } = context(command);
      const result = await fetchFonts();
      const present = FACES.length - result.fetched.length;
      out.result(
        result,
        `${out.colors.success('fonts ready')} in ${out.colors.path(result.dir)} (${result.fetched.length} fetched, ${present} already present)`,
      );
    });
}
