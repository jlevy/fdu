// Write a file whole, or leave the old one in place.
//
// A reader of a file written here sees the old contents or the new ones, never a torn
// file under the final name, even after a crash or a container restart mid-write. The
// data goes to a temporary beside the target, is synced, and is then renamed over the
// target; on any failure the temporary is removed and the target is untouched. This is
// "Write Every File Whole" in docs/project/architecture/fdu-design-principles.md, and
// scripts/check-atomic-writes.mjs fails on a raw write anywhere outside a helper.
//
// It mirrors writeFileSync, so converting a call changes nothing but atomicity: a new
// file gets the permissions writeFileSync would give it, a replaced file keeps its own,
// and a symbolic link is written through rather than replaced.

import { randomBytes } from "node:crypto";
import {
  closeSync,
  fchmodSync,
  fsyncSync,
  lstatSync,
  openSync,
  readlinkSync,
  realpathSync,
  renameSync,
  statSync,
  unlinkSync,
  writeSync,
} from "node:fs";
import { basename, dirname, join, resolve } from "node:path";

const CREATE_ATTEMPTS = 64;

/**
 * Replace `path` with `data` (a string, Buffer, or typed array), as writeFileSync would.
 * `options` is an encoding name or `{ encoding }`; strings default to UTF-8.
 */
export function writeFileAtomicSync(path, data, options = {}) {
  const encoding = typeof options === "string" ? options : (options.encoding ?? "utf8");
  const bytes =
    typeof data === "string"
      ? Buffer.from(data, encoding)
      : Buffer.from(data.buffer, data.byteOffset, data.byteLength);
  const target = throughLink(path);
  const [descriptor, temporary] = createTemporary(target);
  let open = true;
  try {
    keepPermissions(descriptor, target);
    let offset = 0;
    while (offset < bytes.length) {
      offset += writeSync(descriptor, bytes, offset, bytes.length - offset);
    }
    fsyncSync(descriptor);
    closeSync(descriptor);
    open = false;
    renameSync(temporary, target);
  } catch (error) {
    if (open) closeSync(descriptor);
    try {
      unlinkSync(temporary);
    } catch {
      // Already renamed, or never there; the original error is the one to report.
    }
    throw error;
  }
}

function throughLink(path) {
  let link;
  try {
    link = lstatSync(path).isSymbolicLink();
  } catch (error) {
    if (error.code === "ENOENT") return path;
    throw error;
  }
  if (!link) return path;
  try {
    return realpathSync(path);
  } catch (error) {
    // A dangling link: writeFileSync creates the file it names.
    if (error.code === "ENOENT") return resolve(dirname(path), readlinkSync(path));
    throw error;
  }
}

function createTemporary(target) {
  const directory = dirname(target);
  const name = basename(target);
  for (let attempt = 0; attempt < CREATE_ATTEMPTS; attempt += 1) {
    const temporary = join(
      directory,
      `.${name}.tmp.${process.pid}.${randomBytes(8).toString("hex")}`,
    );
    try {
      // 0o666 under the umask is what writeFileSync gives a new file.
      return [openSync(temporary, "wx", 0o666), temporary];
    } catch (error) {
      if (error.code !== "EEXIST") throw error;
    }
  }
  throw new Error(`could not reserve a unique temporary beside ${target}`);
}

function keepPermissions(descriptor, target) {
  let mode;
  try {
    mode = statSync(target).mode;
  } catch (error) {
    if (error.code === "ENOENT") return;
    throw error;
  }
  fchmodSync(descriptor, mode & 0o7777);
}
