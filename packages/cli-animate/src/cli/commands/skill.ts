/** `cli-animate skill`: print the agent skill. */

import { readFileSync } from 'node:fs';
import type { Command } from 'commander';
import { SKILL_FILE } from '../../paths.js';

export function registerSkill(program: Command): void {
  program
    .command('skill')
    .description('print the agent skill (SKILL.md)')
    .action(() => {
      process.stdout.write(readFileSync(SKILL_FILE, 'utf8'));
    });
}
