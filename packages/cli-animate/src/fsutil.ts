/**
 * Files this tool writes for something else to read later are written whole: to a
 * temporary in the same directory, synced, then renamed over the target. A reader sees
 * the old file or the new one, never a torn one, even if the process dies mid-write.
 */

import { randomBytes } from 'node:crypto';
import { closeSync, fsyncSync, mkdirSync, openSync, renameSync, rmSync, writeSync } from 'node:fs';
import { basename, dirname, extname, join } from 'node:path';

/** A sibling temporary name for `path` that keeps its extension (ffmpeg picks a muxer by it). */
export function temporarySibling(path: string): string {
  const extension = extname(path);
  const stem = basename(path, extension);
  return join(dirname(path), `.${stem}.${randomBytes(6).toString('hex')}.tmp${extension}`);
}

export function writeFileAtomic(path: string, data: string | Uint8Array): void {
  mkdirSync(dirname(path), { recursive: true });
  const temporary = temporarySibling(path);
  const bytes = typeof data === 'string' ? Buffer.from(data, 'utf8') : data;
  const descriptor = openSync(temporary, 'wx', 0o644);
  try {
    let offset = 0;
    while (offset < bytes.length) offset += writeSync(descriptor, bytes, offset);
    fsyncSync(descriptor);
  } catch (error) {
    closeSync(descriptor);
    rmSync(temporary, { force: true });
    throw error;
  }
  closeSync(descriptor);
  publish(temporary, path);
}

export function writeJsonAtomic(path: string, value: unknown): void {
  writeFileAtomic(path, `${JSON.stringify(value, null, 2)}\n`);
}

/** Rename a finished temporary over its target, removing it if the rename fails. */
export function publish(temporary: string, path: string): void {
  try {
    renameSync(temporary, path);
  } catch (error) {
    rmSync(temporary, { force: true });
    throw error;
  }
}
