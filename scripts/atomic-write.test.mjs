import assert from "node:assert/strict";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { writeFileAtomicSync } from "./atomic-write.mjs";

function scratch(t) {
  const directory = mkdtempSync(join(tmpdir(), "fdu-atomic-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  return directory;
}

const leftovers = (directory) => readdirSync(directory).filter((name) => name.includes(".tmp."));

test("writes the bytes writeFileSync would", (t) => {
  const directory = scratch(t);
  writeFileAtomicSync(join(directory, "a.txt"), "café\n");
  writeFileSync(join(directory, "b.txt"), "café\n");
  assert.deepEqual(readFileSync(join(directory, "a.txt")), readFileSync(join(directory, "b.txt")));

  writeFileAtomicSync(join(directory, "c.bin"), Uint8Array.from([0, 1, 2]).subarray(1));
  assert.deepEqual([...readFileSync(join(directory, "c.bin"))], [1, 2]);
  writeFileAtomicSync(join(directory, "d.txt"), "ff", "hex");
  assert.deepEqual([...readFileSync(join(directory, "d.txt"))], [0xff]);
  assert.deepEqual(leftovers(directory), []);
});

test("replaces an existing file whole", (t) => {
  const directory = scratch(t);
  const target = join(directory, "artifact.diff");
  writeFileSync(target, "old");
  writeFileAtomicSync(target, "new");
  assert.equal(readFileSync(target, "utf8"), "new");
  assert.deepEqual(leftovers(directory), []);
});

test("a failed write leaves the old file and no temporary", (t) => {
  const directory = scratch(t);
  // A directory under the target name fails the final rename, after the temporary has
  // been written and synced: the latest point a write can fail.
  const target = join(directory, "artifact.diff");
  mkdirSync(join(target, "occupied"), { recursive: true });
  assert.throws(() => writeFileAtomicSync(target, "new"));
  assert.deepEqual(readdirSync(target), ["occupied"]);
  assert.deepEqual(leftovers(directory), []);

  const missing = join(directory, "absent", "artifact.diff");
  assert.throws(() => writeFileAtomicSync(missing, "new"), { code: "ENOENT" });
});

test("keeps a replaced file's permissions and gives a new one open's", (t) => {
  if (process.platform === "win32") return t.skip("permission bits are POSIX");
  const directory = scratch(t);
  const script = join(directory, "run.sh");
  writeFileSync(script, "#!/bin/sh\n");
  chmodSync(script, 0o750);
  writeFileAtomicSync(script, "#!/bin/sh\necho\n");
  assert.equal(statSync(script).mode & 0o777, 0o750);

  writeFileAtomicSync(join(directory, "fresh.txt"), "x");
  writeFileSync(join(directory, "plain.txt"), "x");
  assert.equal(
    statSync(join(directory, "fresh.txt")).mode & 0o777,
    statSync(join(directory, "plain.txt")).mode & 0o777,
  );
});

test("writes through a symbolic link", (t) => {
  if (process.platform === "win32") return t.skip("symlinks need privileges on Windows");
  const directory = scratch(t);
  writeFileSync(join(directory, "real.txt"), "old");
  symlinkSync("real.txt", join(directory, "link.txt"));
  writeFileAtomicSync(join(directory, "link.txt"), "new");
  assert.equal(readFileSync(join(directory, "real.txt"), "utf8"), "new");

  symlinkSync("created.txt", join(directory, "dangling.txt"));
  writeFileAtomicSync(join(directory, "dangling.txt"), "made");
  assert.equal(readFileSync(join(directory, "created.txt"), "utf8"), "made");
});
