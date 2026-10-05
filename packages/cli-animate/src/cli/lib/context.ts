/** What every command handler starts from: the global options and an output for them. */

import { basename, extname } from 'node:path';
import { type Command, InvalidArgumentError } from 'commander';
import { createOutput, type GlobalOptions, type Output } from './output.js';

export interface Context {
  options: GlobalOptions;
  out: Output;
}

export function context(command: Command): Context {
  const o = command.optsWithGlobals<Partial<GlobalOptions>>();
  const options: GlobalOptions = {
    json: o.json ?? false,
    quiet: o.quiet ?? false,
    verbose: o.verbose ?? false,
    color: o.color ?? 'auto',
  };
  return { options, out: createOutput(options) };
}

/** A file's name without its directory or extension. */
export const stem = (path: string): string => basename(path, extname(path));

/** Commander option parsers that refuse what they cannot parse. */
export function integer(value: string): number {
  const n = Number(value);
  if (!Number.isInteger(n) || n <= 0) throw new InvalidArgumentError(`expected a positive integer, got ${value}`);
  return n;
}

export function positive(value: string): number {
  const n = Number(value);
  if (!Number.isFinite(n) || n <= 0) throw new InvalidArgumentError(`expected a positive number, got ${value}`);
  return n;
}

export function nonNegativeInteger(value: string): number {
  const n = Number(value);
  if (!Number.isInteger(n) || n < 0) throw new InvalidArgumentError(`expected a whole number, got ${value}`);
  return n;
}
