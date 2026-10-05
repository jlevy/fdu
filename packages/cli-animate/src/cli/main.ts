#!/usr/bin/env node
/**
 * cli-animate: record real terminal sessions, replay them in a browser, render them to
 * video, and check the video frame by frame against the recording.
 *
 * Each command lives in `commands/`; shared CLI helpers in `lib/`. The engine modules in
 * `src/` never write to the terminal themselves.
 */

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { Command, CommanderError, Option } from 'commander';
import { CliError, EXIT_USAGE } from '../errors.js';
import { PACKAGE_ROOT } from '../paths.js';
import { registerDeliver } from './commands/deliver.js';
import { registerDoctor } from './commands/doctor.js';
import { registerDrive } from './commands/drive.js';
import { registerFonts } from './commands/fonts.js';
import { registerMake } from './commands/make.js';
import { registerRecord } from './commands/record.js';
import { registerRender } from './commands/render.js';
import { registerServe } from './commands/serve.js';
import { registerSkill } from './commands/skill.js';
import { registerVerify } from './commands/verify.js';
import { createOutput, type GlobalOptions } from './lib/output.js';

const { version } = JSON.parse(readFileSync(join(PACKAGE_ROOT, 'package.json'), 'utf8')) as { version: string };

export function program(): Command {
  const cli = new Command('cli-animate')
    .description('Record real terminal sessions, replay them in a browser, and render them to video')
    .version(version)
    .option('--json', 'print results as JSON on stdout')
    .option('--quiet', 'print only results and errors')
    .option('--verbose', 'print more detail')
    .addOption(new Option('--color <when>', 'colorize output').choices(['auto', 'always', 'never']).default('auto'))
    .exitOverride()
    .helpCommand(false)
    .showHelpAfterError('(add --help for usage)')
    .addHelpText(
      'after',
      '\nPipeline: fonts → record → capture a lossless master → verify → deliver; `make` runs it all.\n' +
        'Agents: `cli-animate skill` prints the workflow.',
    );

  cli.commandsGroup('Pipeline:');
  registerMake(cli);
  registerRecord(cli);
  registerRender(cli);
  registerVerify(cli);
  registerDeliver(cli);
  cli.commandsGroup('Viewing:');
  registerServe(cli);
  cli.commandsGroup('Setup:');
  registerDoctor(cli);
  registerFonts(cli);
  registerSkill(cli);
  registerDrive(cli);
  return cli;
}

function globalsFromArgv(argv: string[]): GlobalOptions {
  const color = argv.find((a) => a.startsWith('--color='))?.slice('--color='.length);
  return {
    json: argv.includes('--json'),
    quiet: argv.includes('--quiet'),
    verbose: argv.includes('--verbose'),
    color: color === 'always' || color === 'never' ? color : 'auto',
  };
}

async function main(argv: string[]): Promise<void> {
  // A reader closing stdout early (`| head`) is not a failure; any other stream error is.
  process.stdout.on('error', (error: NodeJS.ErrnoException) => {
    if (error.code === 'EPIPE') {
      process.exitCode ??= 0;
      return;
    }
    throw error;
  });
  try {
    await program().parseAsync(argv);
  } catch (error) {
    if (error instanceof CommanderError) {
      // Commander has already printed help, the version, or the usage message.
      process.exitCode = error.exitCode === 0 ? 0 : EXIT_USAGE;
      return;
    }
    const globals = globalsFromArgv(argv);
    const out = createOutput(globals);
    if (error instanceof CliError) {
      out.error(error.message);
      process.exitCode = error.exitCode;
    } else {
      // A missing input or a failed program: its message, and the stack only with --verbose.
      const err = error as NodeJS.ErrnoException;
      out.error(globals.verbose ? (err.stack ?? String(err)) : err.message);
      process.exitCode = EXIT_USAGE;
    }
  }
}

await main(process.argv);
